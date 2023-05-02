use crate::helpers::{
    buy_now, match_nfts_against_tokens, match_tokens_against_any_nfts,
    match_tokens_against_specific_nfts, validate_nft_owner, MatchedBid,
};
use crate::state::{CONFIG, FORWARD_NFTS};
use crate::ContractError;
use crate::{helpers::validate_nft_orders, msg::ExecuteMsg};
use cosmwasm_std::{coin, to_binary, Addr, DepsMut, Env, MessageInfo, SubMsg, Uint128, WasmMsg};
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
        ExecuteMsg::SwapTokensForAnyNfts {
            collection,
            orders,
            swap_params,
        } => execute_swap_tokens_for_any_nfts(
            deps,
            env,
            info,
            api.addr_validate(&collection)?,
            orders,
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

    validate_nft_orders(&nft_orders, config.max_batch_size)?;
    validate_nft_owner(&deps.querier, &info.sender, &collection, &nft_orders)?;

    let matches = match_nfts_against_tokens(
        deps.as_ref(),
        &env.block,
        &config,
        &collection,
        nft_orders,
        swap_params.robust,
    )?;

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

    for matched_order in matches {
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

    validate_nft_orders(&nft_orders, config.max_batch_size)?;

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
    let mut remaining_balance = received_amount;

    let matches = match_tokens_against_specific_nfts(
        deps.as_ref(),
        &config,
        &collection,
        nft_orders,
        swap_params.robust,
    )?;

    let finder = swap_params.finder.map(|f| f.to_string());

    let mut response = Response::new();

    for matched_order in matches {
        if matched_order.matched_ask.is_none() {
            continue;
        }
        let matched_ask = matched_order.matched_ask.unwrap();
        remaining_balance -= matched_ask.price;

        response = buy_now(response, &matched_ask, &finder, &config.marketplace)?;

        FORWARD_NFTS.save(
            deps.storage,
            (
                matched_ask.collection.clone(),
                matched_ask.token_id.to_string(),
            ),
            &info.sender,
        )?;
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

/// Execute a SwapTokensForAnyNfts message
pub fn execute_swap_tokens_for_any_nfts(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: Addr,
    orders: Vec<Uint128>,
    swap_params: SwapParamsInternal,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if orders.is_empty() {
        return Err(ContractError::InvalidInput(
            "nft orders must not be empty".to_string(),
        ));
    }
    if orders.len() > config.max_batch_size as usize {
        return Err(ContractError::InvalidInput(
            "nft orders must not exceed max batch size".to_string(),
        ));
    }

    let expected_amount = orders
        .iter()
        .fold(Uint128::zero(), |acc, nft_order| acc + nft_order);
    let received_amount = must_pay(&info, NATIVE_DENOM)?;
    if received_amount != expected_amount {
        return Err(ContractError::InsufficientFunds(format!(
            "expected {} but received {}",
            expected_amount, received_amount
        )));
    }
    let mut remaining_balance = received_amount;

    let matches = match_tokens_against_any_nfts(
        deps.as_ref(),
        &env.block,
        &config,
        &collection,
        orders,
        swap_params.robust,
    )?;

    let finder = swap_params.finder.map(|f| f.to_string());

    let mut response = Response::new();

    for matched_order in matches {
        if matched_order.matched_ask.is_none() {
            continue;
        }
        let matched_ask = matched_order.matched_ask.unwrap();
        remaining_balance -= matched_ask.price;

        response = buy_now(response, &matched_ask, &finder, &config.marketplace)?;

        FORWARD_NFTS.save(
            deps.storage,
            (
                matched_ask.collection.clone(),
                matched_ask.token_id.to_string(),
            ),
            &info.sender,
        )?;
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
