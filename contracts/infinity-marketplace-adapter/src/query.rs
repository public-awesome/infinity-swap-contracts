use crate::helpers::{
    match_nfts_against_tokens, match_tokens_against_any_nfts, match_tokens_against_specific_nfts,
    set_ask_swap_data, set_bid_swap_data, tx_fees_to_swap, validate_nft_orders, validate_nft_owner,
    MatchedBid,
};
use crate::msg::QueryMsg;
use crate::state::CONFIG;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, StdError, StdResult, Uint128};
use infinity_shared::interface::{
    transform_swap_params, NftOrder, Swap, SwapParamsInternal, SwapResponse, TransactionType,
};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use infinity_shared::shared::load_marketplace_params;
use sg_marketplace_common::{calculate_nft_sale_fees, load_collection_royalties};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
        QueryMsg::SimSwapNftsForTokens {
            sender,
            collection,
            nft_orders,
            swap_params,
        } => to_binary(&query_sim_swap_nfts_for_tokens(
            deps,
            env,
            api.addr_validate(&sender)?,
            api.addr_validate(&collection)?,
            nft_orders,
            transform_swap_params(api, swap_params)?,
        )?),
        QueryMsg::SimSwapTokensForSpecificNfts {
            sender,
            collection,
            nft_orders,
            swap_params,
        } => to_binary(&query_sim_swap_tokens_for_specific_nfts(
            deps,
            env,
            api.addr_validate(&sender)?,
            api.addr_validate(&collection)?,
            nft_orders,
            transform_swap_params(api, swap_params)?,
        )?),
        QueryMsg::SimSwapTokensForAnyNfts {
            sender,
            collection,
            orders,
            swap_params,
        } => to_binary(&query_sim_swap_tokens_for_any_nfts(
            deps,
            env,
            api.addr_validate(&sender)?,
            api.addr_validate(&collection)?,
            orders,
            transform_swap_params(api, swap_params)?,
        )?),
    }
}

pub fn query_sim_swap_nfts_for_tokens(
    deps: Deps,
    env: Env,
    sender: Addr,
    collection: Addr,
    nft_orders: Vec<NftOrder>,
    swap_params: SwapParamsInternal,
) -> StdResult<SwapResponse> {
    let config = CONFIG.load(deps.storage)?;

    validate_nft_orders(&nft_orders).map_err(|err| StdError::generic_err(err.to_string()))?;
    validate_nft_owner(&deps.querier, &sender, &collection, &nft_orders)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    let matches = match_nfts_against_tokens(
        deps,
        &env.block,
        &config,
        &collection,
        nft_orders,
        swap_params.robust,
    )
    .map_err(|err| StdError::generic_err(err.to_string()))?;

    let sender_recipient = swap_params.asset_recipient.unwrap_or(sender);

    let mut swaps: Vec<Swap> = vec![];

    let marketplace_params = load_marketplace_params(&deps.querier, &config.marketplace)?;
    let royalty_info = load_collection_royalties(&deps.querier, deps.api, &collection)?;

    for matched_order in matches {
        if matched_order.matched_bid.is_none() {
            continue;
        }
        let matched_bid = matched_order.matched_bid.unwrap();

        let (sale_price, bidder, finders_fee_bps): (Uint128, Addr, Option<u64>) = match &matched_bid
        {
            MatchedBid::Bid(bid) => (bid.price, bid.bidder.clone(), bid.finders_fee_bps),
            MatchedBid::CollectionBid(collection_bid) => (
                collection_bid.price,
                collection_bid.bidder.clone(),
                collection_bid.finders_fee_bps,
            ),
        };

        let token_id = matched_order.nft_order.token_id;

        let tx_fees = calculate_nft_sale_fees(
            sale_price,
            marketplace_params.trading_fee_percent,
            &sender_recipient,
            swap_params.finder.as_ref(),
            finders_fee_bps,
            royalty_info.as_ref(),
        )?;

        let mut swap = tx_fees_to_swap(
            tx_fees,
            TransactionType::UserSubmitsNfts,
            &token_id,
            sale_price,
            &bidder,
            &config.marketplace,
        );
        set_bid_swap_data(&mut swap, &matched_bid);
        swaps.push(swap);
    }

    Ok(SwapResponse { swaps })
}

pub fn query_sim_swap_tokens_for_specific_nfts(
    deps: Deps,
    _env: Env,
    sender: Addr,
    collection: Addr,
    nft_orders: Vec<NftOrder>,
    swap_params: SwapParamsInternal,
) -> StdResult<SwapResponse> {
    let config = CONFIG.load(deps.storage)?;

    validate_nft_orders(&nft_orders).map_err(|err| StdError::generic_err(err.to_string()))?;

    let matches = match_tokens_against_specific_nfts(
        deps,
        &config,
        &collection,
        nft_orders,
        swap_params.robust,
    )
    .map_err(|err| StdError::generic_err(err.to_string()))?;

    let sender_recipient = swap_params.asset_recipient.unwrap_or(sender);

    let mut swaps: Vec<Swap> = vec![];

    let marketplace_params = load_marketplace_params(&deps.querier, &config.marketplace)?;
    let royalty_info = load_collection_royalties(&deps.querier, deps.api, &collection)?;

    for matched_order in matches {
        if matched_order.matched_ask.is_none() {
            continue;
        }
        let token_id = matched_order.nft_order.token_id;
        let matched_ask = matched_order.matched_ask.unwrap();

        let ask_recipient = matched_ask
            .funds_recipient
            .as_ref()
            .unwrap_or(&matched_ask.seller)
            .clone();

        let tx_fees = calculate_nft_sale_fees(
            matched_ask.price,
            marketplace_params.trading_fee_percent,
            &ask_recipient,
            swap_params.finder.as_ref(),
            matched_ask.finders_fee_bps,
            royalty_info.as_ref(),
        )?;

        let mut swap = tx_fees_to_swap(
            tx_fees,
            TransactionType::UserSubmitsTokens,
            &token_id,
            matched_ask.price,
            &sender_recipient,
            &config.marketplace,
        );
        set_ask_swap_data(&mut swap, &matched_ask);
        swaps.push(swap);
    }

    Ok(SwapResponse { swaps })
}

pub fn query_sim_swap_tokens_for_any_nfts(
    deps: Deps,
    env: Env,
    sender: Addr,
    collection: Addr,
    orders: Vec<Uint128>,
    swap_params: SwapParamsInternal,
) -> StdResult<SwapResponse> {
    let config = CONFIG.load(deps.storage)?;

    if orders.is_empty() {
        return Err(StdError::generic_err(
            "nft orders must not be empty".to_string(),
        ));
    }

    let matches = match_tokens_against_any_nfts(
        deps,
        &env.block,
        &config,
        &collection,
        orders,
        swap_params.robust,
    )
    .map_err(|err| StdError::generic_err(err.to_string()))?;

    let sender_recipient = swap_params.asset_recipient.unwrap_or(sender);

    let mut swaps: Vec<Swap> = vec![];

    let marketplace_params = load_marketplace_params(&deps.querier, &config.marketplace)?;
    let royalty_info = load_collection_royalties(&deps.querier, deps.api, &collection)?;

    for matched_order in matches {
        if matched_order.matched_ask.is_none() {
            continue;
        }
        let matched_ask = matched_order.matched_ask.unwrap();
        let token_id = matched_ask.token_id.to_string();

        let ask_recipient = matched_ask
            .funds_recipient
            .as_ref()
            .unwrap_or(&matched_ask.seller)
            .clone();

        let tx_fees = calculate_nft_sale_fees(
            matched_ask.price,
            marketplace_params.trading_fee_percent,
            &ask_recipient,
            swap_params.finder.as_ref(),
            matched_ask.finders_fee_bps,
            royalty_info.as_ref(),
        )?;

        let mut swap = tx_fees_to_swap(
            tx_fees,
            TransactionType::UserSubmitsTokens,
            &token_id,
            matched_ask.price,
            &sender_recipient,
            &config.marketplace,
        );
        set_ask_swap_data(&mut swap, &matched_ask);
        swaps.push(swap);
    }

    Ok(SwapResponse { swaps })
}
