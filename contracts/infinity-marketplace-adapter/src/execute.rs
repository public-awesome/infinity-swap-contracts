use crate::helpers::{match_user_submitted_nfts, MatchedBid};
use crate::state::CONFIG;
use crate::ContractError;
use crate::{helpers::validate_user_submitted_nfts, msg::ExecuteMsg};
use cosmwasm_std::{to_binary, Addr, DepsMut, Env, MessageInfo, SubMsg, WasmMsg};
use cw721::Cw721ExecuteMsg;
use infinity_shared::interface::{transform_swap_params, NftOrder, SwapParamsInternal};
use sg_marketplace::msg::ExecuteMsg as MarketplaceExecuteMsg;
use sg_marketplace_common::transfer_nft;
use sg_std::Response;

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

    validate_user_submitted_nfts(deps.as_ref(), &info.sender, &collection, &nft_orders)?;

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
