use crate::helpers::{
    build_swap, fetch_asks, fetch_collection_bids, load_collection_royalties,
    load_marketplace_params, validate_ask,
};
use crate::msg::{QueryMsg, MAX_QUERY_LIMIT};
use crate::state::CONFIG;
use crate::ContractError;
use cosmwasm_std::{entry_point, to_binary, Addr, Binary, Deps, Env, StdError, StdResult, Uint128};
use infinity_interface::{transform_swap_params, NftOrder, Swap, SwapParamsInternal, SwapResponse};
use sg_marketplace::msg::{AskResponse, BidResponse, QueryMsg as MarketplaceQueryMsg};
use sg_marketplace::state::Order;
use std::cmp::min;
use std::iter::zip;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
        QueryMsg::SimSwapTokensForSpecificNfts {
            sender,
            collection,
            nft_orders,
            swap_params,
        } => to_binary(&sim_swap_tokens_for_specific_nfts(
            deps,
            env,
            api.addr_validate(&sender)?,
            api.addr_validate(&collection)?,
            nft_orders,
            transform_swap_params(deps, swap_params)?,
        )?),
        QueryMsg::SimSwapTokensForAnyNfts {
            sender,
            collection,
            orders,
            swap_params,
        } => to_binary(&sim_swap_tokens_for_any_nfts(
            deps,
            env,
            api.addr_validate(&sender)?,
            api.addr_validate(&collection)?,
            orders,
            transform_swap_params(deps, swap_params)?,
        )?),
        QueryMsg::SimSwapNftsForTokens {
            sender,
            collection,
            nft_orders,
            swap_params,
        } => to_binary(&sim_swap_nfts_for_tokens(
            deps,
            env,
            api.addr_validate(&sender)?,
            api.addr_validate(&collection)?,
            nft_orders,
            transform_swap_params(deps, swap_params)?,
        )?),
    }
}

pub fn sim_swap_tokens_for_specific_nfts(
    deps: Deps,
    env: Env,
    sender: Addr,
    collection: Addr,
    nft_orders: Vec<NftOrder>,
    swap_params: SwapParamsInternal,
) -> StdResult<SwapResponse> {
    let config = CONFIG.load(deps.storage)?;

    let marketplace_params = load_marketplace_params(deps, &config.marketplace)
        .map_err(|err| StdError::generic_err(err.to_string()))?;
    let collection_royalties = load_collection_royalties(deps, &collection)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    let mut swaps: Vec<Swap> = vec![];
    for nft_order in nft_orders {
        let ask_response: AskResponse = deps.querier.query_wasm_smart(
            &config.marketplace,
            &MarketplaceQueryMsg::Ask {
                collection: collection.to_string(),
                token_id: nft_order.token_id.parse::<u32>().unwrap(),
            },
        )?;
        let validate_ask_result = if ask_response.ask.is_none() {
            Err(ContractError::InvalidAsk("not found".to_string()))
        } else {
            validate_ask(
                &env.block,
                &ask_response.ask.as_ref().unwrap(),
                &sender,
                &Some(nft_order),
            )
        };

        if validate_ask_result.is_err() {
            if swap_params.robust {
                break;
            } else {
                return Err(StdError::generic_err(
                    validate_ask_result.err().unwrap().to_string(),
                ));
            }
        }

        let ask = ask_response.ask.unwrap();
        let swap = build_swap(
            ask.token_id.to_string(),
            ask.price,
            ask.seller.to_string(),
            sender.to_string(),
            ask.finders_fee_bps.unwrap_or(0),
            swap_params.finder.as_ref().map(|finder| finder.to_string()),
            &marketplace_params.params,
            &collection_royalties,
        );
        swaps.push(swap);
    }

    Ok(SwapResponse { swaps })
}

pub fn sim_swap_tokens_for_any_nfts(
    deps: Deps,
    env: Env,
    sender: Addr,
    collection: Addr,
    orders: Vec<Uint128>,
    swap_params: SwapParamsInternal,
) -> StdResult<SwapResponse> {
    let config = CONFIG.load(deps.storage)?;

    let marketplace_params = load_marketplace_params(deps, &config.marketplace)
        .map_err(|err| StdError::generic_err(err.to_string()))?;
    let collection_royalties = load_collection_royalties(deps, &collection)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    let query_limit = min(MAX_QUERY_LIMIT, orders.len() as u32);
    let asks = fetch_asks(
        deps,
        &config.marketplace,
        &collection,
        &env.block,
        &sender,
        query_limit,
    )
    .map_err(|err| StdError::generic_err(err.to_string()))?;

    let mut swaps: Vec<Swap> = vec![];
    for (order, ask) in zip(orders, asks) {
        if ask.price > order {
            if swap_params.robust {
                break;
            } else {
                return Err(StdError::generic_err(
                    ContractError::PriceMismatch("price too high".to_string()).to_string(),
                ));
            }
        }
        let swap = build_swap(
            ask.token_id.to_string(),
            ask.price,
            ask.seller.to_string(),
            sender.to_string(),
            ask.finders_fee_bps.unwrap_or(0),
            swap_params.finder.as_ref().map(|finder| finder.to_string()),
            &marketplace_params.params,
            &collection_royalties,
        );
        swaps.push(swap);
    }

    Ok(SwapResponse { swaps })
}

pub fn sim_swap_nfts_for_tokens(
    deps: Deps,
    env: Env,
    sender: Addr,
    collection: Addr,
    nft_orders: Vec<NftOrder>,
    swap_params: SwapParamsInternal,
) -> StdResult<SwapResponse> {
    let config = CONFIG.load(deps.storage)?;

    let marketplace_params = load_marketplace_params(deps, &config.marketplace)
        .map_err(|err| StdError::generic_err(err.to_string()))?;
    let collection_royalties = load_collection_royalties(deps, &collection)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    let query_limit = min(MAX_QUERY_LIMIT, nft_orders.len() as u32);
    let mut collection_bids = fetch_collection_bids(
        deps,
        &config.marketplace,
        &collection,
        &env.block,
        query_limit,
    )
    .map_err(|err| StdError::generic_err(err.to_string()))?;

    let mut swaps: Vec<Swap> = vec![];
    for nft_order in nft_orders {
        let bid_response: BidResponse = deps.querier.query_wasm_smart(
            &config.marketplace,
            &MarketplaceQueryMsg::Bid {
                collection: collection.to_string(),
                token_id: nft_order.token_id.parse::<u32>().unwrap(),
                bidder: sender.to_string(),
            },
        )?;

        let validate_bid_result = if bid_response.bid.is_none() {
            Err(ContractError::InvalidBid("not found".to_string()))
        } else {
            if bid_response.bid.as_ref().unwrap().is_expired(&env.block) {
                Err(ContractError::InvalidBid("expired".to_string()))
            } else {
                Ok(())
            }
        };

        if validate_bid_result.is_err() {
            if swap_params.robust {
                break;
            } else {
                return Err(StdError::generic_err(
                    validate_bid_result.err().unwrap().to_string(),
                ));
            }
        }

        let bid = bid_response.bid.unwrap();
        let swap = if bid.price
            < collection_bids
                .first()
                .map_or(Uint128::zero(), |cb| cb.price)
        {
            let bid = collection_bids.pop().unwrap();
            build_swap(
                nft_order.token_id.to_string(),
                bid.price,
                sender.to_string(),
                bid.bidder.to_string(),
                bid.finders_fee_bps.unwrap_or(0),
                swap_params.finder.as_ref().map(|finder| finder.to_string()),
                &marketplace_params.params,
                &collection_royalties,
            )
        } else {
            build_swap(
                nft_order.token_id.to_string(),
                bid.price,
                sender.to_string(),
                bid.bidder.to_string(),
                bid.finders_fee_bps.unwrap_or(0),
                swap_params.finder.as_ref().map(|finder| finder.to_string()),
                &marketplace_params.params,
                &collection_royalties,
            )
        };
        swaps.push(swap);
    }

    Ok(SwapResponse { swaps })
}
