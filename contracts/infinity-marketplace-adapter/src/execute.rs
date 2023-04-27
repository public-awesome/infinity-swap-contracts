use crate::helpers::{match_user_submitted_nfts, match_user_submitted_tokens, MatchedBid};
use crate::state::CONFIG;
use crate::ContractError;
use crate::{helpers::validate_nft_orders, msg::ExecuteMsg};
use cosmwasm_std::{
    coin, coins, to_binary, Addr, DepsMut, Env, MessageInfo, SubMsg, Uint128, WasmMsg,
};
use cw721::Cw721ExecuteMsg;
use cw_utils::must_pay;
use infinity_shared::interface::{transform_swap_params, NftOrder, SwapParamsInternal};
use sg_marketplace::msg::ExecuteMsg as MarketplaceExecuteMsg;
use sg_marketplace_common::{bank_send, transfer_nft};
use sg_std::{Response, NATIVE_DENOM};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::SwapNftsForTokens {
            collection,
            nft_orders,
            swap_params,
        } => execute_swap_nfts_for_tokens(
            deps,
            env,
            info,
            api.addr_validate(&collection)?,
            nft_orders,
            transform_swap_params(api, swap_params)?,
        ),
        ExecuteMsg::SwapTokensForSpecificNfts {
            collection,
            nft_orders,
            swap_params,
        } => execute_swap_tokens_for_specific_nfts(
            deps,
            env,
            info,
            api.addr_validate(&collection)?,
            nft_orders,
            transform_swap_params(api, swap_params)?,
        ),
    }
}

/// Execute a SwapNftsForTokens message
pub fn execute_swap_nfts_for_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: Addr,
    nft_orders: Vec<NftOrder>,
    swap_params: SwapParamsInternal,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    validate_nft_orders(
        deps.as_ref(),
        &info.sender,
        &collection,
        &nft_orders,
        config.max_batch_size,
    )?;

    let matched_user_submitted_nfts =
        match_user_submitted_nfts(deps.as_ref(), &env.block, &config, &collection, nft_orders)?;

    if !swap_params.robust {
        let missing_match = matched_user_submitted_nfts
            .iter()
            .any(|m| m.matched_bid.is_none());
        if missing_match {
            return Err(ContractError::MatchError(
                "all nfts not matched with a bid".to_string(),
            ));
        }
    }

    let sender_recipient = swap_params.asset_recipient.unwrap_or(info.sender);
    let finder = swap_params.finder.map(|f| f.to_string());

    let mut response = Response::new();

    let approve_all_msg = Cw721ExecuteMsg::ApproveAll {
        operator: config.marketplace.to_string(),
        expires: None,
    };
    response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_binary(&approve_all_msg)?,
        funds: vec![],
    }));

    for matched_order in matched_user_submitted_nfts {
        if matched_order.matched_bid.is_none() {
            continue;
        }
        let matched_bid = matched_order.matched_bid.unwrap();

        let transfer_nft_msg = transfer_nft(
            &collection,
            &matched_order.nft_order.token_id,
            &env.contract.address,
        );
        response = response.add_submessage(transfer_nft_msg);

        match matched_bid {
            MatchedBid::Bid(bid) => {
                let accept_bid_msg = MarketplaceExecuteMsg::AcceptBid {
                    collection: collection.to_string(),
                    token_id: bid.token_id,
                    bidder: bid.bidder.to_string(),
                    finder: finder.clone(),
                    funds_recipient: Some(sender_recipient.to_string()),
                };
                response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
                    contract_addr: config.marketplace.to_string(),
                    msg: to_binary(&accept_bid_msg)?,
                    funds: vec![],
                }));
            }
            MatchedBid::CollectionBid(collection_bid) => {
                let token_id_num = matched_order.nft_order.token_id.parse::<u32>().unwrap();
                let accept_collection_bid_msg = MarketplaceExecuteMsg::AcceptCollectionBid {
                    collection: collection.to_string(),
                    token_id: token_id_num,
                    bidder: collection_bid.bidder.to_string(),
                    finder: finder.clone(),
                    funds_recipient: Some(sender_recipient.to_string()),
                };
                response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
                    contract_addr: config.marketplace.to_string(),
                    msg: to_binary(&accept_collection_bid_msg)?,
                    funds: vec![],
                }));
            }
        }
    }

    Ok(response)
}

/// Execute a SwapTokensForSpecificNfts message
pub fn execute_swap_tokens_for_specific_nfts(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: Addr,
    nft_orders: Vec<NftOrder>,
    swap_params: SwapParamsInternal,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    validate_nft_orders(
        deps.as_ref(),
        &info.sender,
        &collection,
        &nft_orders,
        config.max_batch_size,
    )?;

    let expected_amount = nft_orders
        .iter()
        .fold(Uint128::zero(), |acc, nft_order| acc + nft_order.amount);
    let received_amount = must_pay(&info, NATIVE_DENOM)?;
    if received_amount != expected_amount {
        return Err(ContractError::InsufficientFunds(format!(
            "expected {} but received {}",
            expected_amount, received_amount
        )));
    }
    let mut remaining_balance = received_amount.clone();

    let matched_user_submitted_tokens =
        match_user_submitted_tokens(deps.as_ref(), &config, &collection, nft_orders)?;

    if !swap_params.robust {
        let missing_match = matched_user_submitted_tokens
            .iter()
            .any(|m| m.matched_ask.is_none());
        if missing_match {
            return Err(ContractError::MatchError(
                "all nfts not matched with a bid".to_string(),
            ));
        }
    }

    let finder = swap_params.finder.map(|f| f.to_string());

    let mut response = Response::new();

    for matched_order in matched_user_submitted_tokens {
        if matched_order.matched_ask.is_none() {
            continue;
        }
        let matched_ask = matched_order.matched_ask.unwrap();

        let buy_now_msg = MarketplaceExecuteMsg::BuyNow {
            collection: collection.to_string(),
            token_id: matched_ask.token_id,
            expires: matched_ask.expires_at,
            finder: finder.clone(),
            finders_fee_bps: None,
        };

        remaining_balance -= matched_ask.price;

        response = response.add_submessage(SubMsg::new(WasmMsg::Execute {
            contract_addr: config.marketplace.to_string(),
            msg: to_binary(&buy_now_msg)?,
            funds: coins(matched_ask.price.u128(), NATIVE_DENOM),
        }));
    }

    // Refund remaining balance
    if remaining_balance.u128() > 0 {
        response = response.add_submessage(bank_send(
            coin(remaining_balance.u128(), NATIVE_DENOM),
            &info.sender,
        ));
    }

    Ok(response)
}
