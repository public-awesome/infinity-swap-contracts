use crate::helpers::{load_collection_royalties, load_marketplace_params, option_bool_to_order};
use crate::msg::{
    ConfigResponse, NftSwap, PoolNftSwap, PoolQuoteResponse, PoolsByIdResponse, PoolsResponse,
    QueryMsg, QueryOptions, SwapParams, SwapResponse,
};
use crate::state::{buy_pool_quotes, pools, sell_pool_quotes, PoolQuote, CONFIG};
use crate::swap_processor::{SwapProcessor, TransactionType};
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, Env, Order, StdError, StdResult, Uint128,
};
use cw_storage_plus::Bound;
use cw_utils::maybe_addr;

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Pools { query_options } => to_binary(&query_pools(deps, query_options)?),
        QueryMsg::PoolsById { pool_ids } => to_binary(&query_pools_by_id(deps, pool_ids)?),
        QueryMsg::PoolsByOwner {
            owner,
            query_options,
        } => to_binary(&query_pools_by_owner(
            deps,
            api.addr_validate(&owner)?,
            query_options,
        )?),
        QueryMsg::PoolQuotesBuy {
            collection,
            query_options,
        } => to_binary(&query_pool_quotes_by_buy_price(
            deps,
            api.addr_validate(&collection)?,
            query_options,
        )?),
        QueryMsg::PoolQuotesSell {
            collection,
            query_options,
        } => to_binary(&query_pool_quotes_by_sell_price(
            deps,
            api.addr_validate(&collection)?,
            query_options,
        )?),
        QueryMsg::SimDirectSwapNftsForTokens {
            pool_id,
            nfts_to_swap,
            swap_params,
            token_recipient,
            finder,
        } => to_binary(&sim_direct_swap_nfts_for_tokens(
            deps,
            pool_id,
            nfts_to_swap,
            swap_params,
            api.addr_validate(&token_recipient)?,
            maybe_addr(api, finder)?,
        )?),
        QueryMsg::SimSwapNftsForTokens {
            collection,
            nfts_to_swap,
            swap_params,
            token_recipient,
            finder,
        } => to_binary(&sim_swap_nfts_for_tokens(
            deps,
            api.addr_validate(&collection)?,
            nfts_to_swap,
            swap_params,
            api.addr_validate(&token_recipient)?,
            maybe_addr(api, finder)?,
        )?),
        QueryMsg::SimDirectSwapTokensforSpecificNfts {
            pool_id,
            nfts_to_swap_for,
            swap_params,
            nft_recipient,
            finder,
        } => to_binary(&sim_direct_swap_tokens_for_specific_nfts(
            deps,
            pool_id,
            nfts_to_swap_for,
            swap_params,
            api.addr_validate(&nft_recipient)?,
            maybe_addr(api, finder)?,
        )?),
        QueryMsg::SimSwapTokensForSpecificNfts {
            collection,
            pool_nfts_to_swap_for,
            swap_params,
            nft_recipient,
            finder,
        } => to_binary(&sim_swap_tokens_for_specific_nfts(
            deps,
            api.addr_validate(&collection)?,
            pool_nfts_to_swap_for,
            swap_params,
            api.addr_validate(&nft_recipient)?,
            maybe_addr(api, finder)?,
        )?),
        QueryMsg::SimSwapTokensForAnyNfts {
            collection,
            max_expected_token_input,
            swap_params,
            nft_recipient,
            finder,
        } => to_binary(&sim_swap_tokens_for_any_nfts(
            deps,
            api.addr_validate(&collection)?,
            max_expected_token_input,
            swap_params,
            api.addr_validate(&nft_recipient)?,
            maybe_addr(api, finder)?,
        )?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

pub fn query_pools(deps: Deps, query_options: QueryOptions<u64>) -> StdResult<PoolsResponse> {
    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.map(Bound::exclusive);
    let order = option_bool_to_order(query_options.descending, Order::Ascending);

    let pools = pools()
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;

    Ok(PoolsResponse { pools })
}

pub fn query_pools_by_id(deps: Deps, pool_ids: Vec<u64>) -> StdResult<PoolsByIdResponse> {
    let mut resp_vec = vec![];
    for pool_id in pool_ids {
        let pool = pools().may_load(deps.storage, pool_id)?;
        resp_vec.push((pool_id, pool));
    }
    Ok(PoolsByIdResponse { pools: resp_vec })
}

pub fn query_pools_by_owner(
    deps: Deps,
    owner: Addr,
    query_options: QueryOptions<u64>,
) -> StdResult<PoolsResponse> {
    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.map(Bound::exclusive);
    let order = option_bool_to_order(query_options.descending, Order::Ascending);

    let pools = pools()
        .idx
        .owner
        .prefix(owner)
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;

    Ok(PoolsResponse { pools })
}

pub fn query_pool_quotes_by_buy_price(
    deps: Deps,
    collection: Addr,
    query_options: QueryOptions<(Uint128, u64)>,
) -> StdResult<PoolQuoteResponse> {
    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT) as usize;
    let start = query_options
        .start_after
        .as_ref()
        .map(|offset| Bound::exclusive((offset.0.u128(), offset.1)));
    let order = option_bool_to_order(query_options.descending, Order::Ascending);

    let pool_quotes: Vec<PoolQuote> = buy_pool_quotes()
        .idx
        .collection_buy_price
        .sub_prefix(collection)
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;

    Ok(PoolQuoteResponse { pool_quotes })
}

pub fn query_pool_quotes_by_sell_price(
    deps: Deps,
    collection: Addr,
    query_options: QueryOptions<(Uint128, u64)>,
) -> StdResult<PoolQuoteResponse> {
    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT) as usize;
    let start = query_options
        .start_after
        .as_ref()
        .map(|offset| Bound::exclusive((offset.0.u128(), offset.1)));
    let order = option_bool_to_order(query_options.descending, Order::Descending);

    let pool_quotes: Vec<PoolQuote> = sell_pool_quotes()
        .idx
        .collection_sell_price
        .sub_prefix(collection)
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;

    Ok(PoolQuoteResponse { pool_quotes })
}

pub fn sim_direct_swap_nfts_for_tokens(
    deps: Deps,
    pool_id: u64,
    nfts_to_swap: Vec<NftSwap>,
    swap_params: SwapParams,
    asset_recipient: Addr,
    finder: Option<Addr>,
) -> StdResult<SwapResponse> {
    let config = CONFIG.load(deps.storage)?;

    // convert to StdErr
    let marketplace_params = load_marketplace_params(deps, &config.marketplace_addr)
        .map_err(|_| StdError::generic_err("Marketplace not found"))?;

    let pool = pools().load(deps.storage, pool_id)?;

    let collection_royalties = load_collection_royalties(deps, &pool.collection)
        .map_err(|_| StdError::generic_err("Collection not found"))?;

    let mut processor = SwapProcessor::new(
        TransactionType::Sell,
        pool.collection.clone(),
        asset_recipient.clone(),
        Uint128::zero(),
        asset_recipient,
        marketplace_params.params.trading_fee_percent,
        collection_royalties,
        finder,
        config.developer,
    );
    processor
        .direct_swap_nfts_for_tokens(pool, nfts_to_swap, swap_params)
        .map_err(|_| StdError::generic_err("direct_swap_nft_for_tokens err"))?;

    Ok(SwapResponse {
        swaps: processor.swaps,
    })
}

pub fn sim_swap_nfts_for_tokens(
    deps: Deps,
    collection: Addr,
    nfts_to_swap: Vec<NftSwap>,
    swap_params: SwapParams,
    asset_recipient: Addr,
    finder: Option<Addr>,
) -> StdResult<SwapResponse> {
    let config = CONFIG.load(deps.storage)?;

    let marketplace_params = load_marketplace_params(deps, &config.marketplace_addr)
        .map_err(|_| StdError::generic_err("Marketplace not found"))?;

    let collection_royalties = load_collection_royalties(deps, &collection)
        .map_err(|_| StdError::generic_err("Collection not found"))?;

    let mut processor = SwapProcessor::new(
        TransactionType::Sell,
        collection,
        asset_recipient.clone(),
        Uint128::zero(),
        asset_recipient,
        marketplace_params.params.trading_fee_percent,
        collection_royalties,
        finder,
        config.developer,
    );
    processor
        .swap_nfts_for_tokens(deps.storage, nfts_to_swap, swap_params)
        .map_err(|_| StdError::generic_err("swap_nfts_for_tokens err"))?;

    Ok(SwapResponse {
        swaps: processor.swaps,
    })
}

pub fn sim_direct_swap_tokens_for_specific_nfts(
    deps: Deps,
    pool_id: u64,
    nfts_to_swap_for: Vec<NftSwap>,
    swap_params: SwapParams,
    nft_recipient: Addr,
    finder: Option<Addr>,
) -> StdResult<SwapResponse> {
    let pool = pools().load(deps.storage, pool_id)?;

    sim_swap_tokens_for_specific_nfts(
        deps,
        pool.collection,
        vec![PoolNftSwap {
            pool_id,
            nft_swaps: nfts_to_swap_for,
        }],
        swap_params,
        nft_recipient,
        finder,
    )
}

pub fn sim_swap_tokens_for_specific_nfts(
    deps: Deps,
    collection: Addr,
    pool_nfts_to_swap_for: Vec<PoolNftSwap>,
    swap_params: SwapParams,
    nft_recipient: Addr,
    finder: Option<Addr>,
) -> StdResult<SwapResponse> {
    let config = CONFIG.load(deps.storage)?;
    let marketplace_params = load_marketplace_params(deps, &config.marketplace_addr)
        .map_err(|_| StdError::generic_err("Marketplace not found"))?;

    let collection_royalties = load_collection_royalties(deps, &collection)
        .map_err(|_| StdError::generic_err("Collection not found"))?;

    let mut processor = SwapProcessor::new(
        TransactionType::Buy,
        collection,
        nft_recipient.clone(),
        Uint128::zero(),
        nft_recipient,
        marketplace_params.params.trading_fee_percent,
        collection_royalties,
        finder,
        config.developer,
    );
    processor
        .swap_tokens_for_specific_nfts(deps.storage, pool_nfts_to_swap_for, swap_params)
        .map_err(|_| StdError::generic_err("swap_tokens_for_specific_nfts err"))?;

    Ok(SwapResponse {
        swaps: processor.swaps,
    })
}

pub fn sim_swap_tokens_for_any_nfts(
    deps: Deps,
    collection: Addr,
    max_expected_token_input: Vec<Uint128>,
    swap_params: SwapParams,
    nft_recipient: Addr,
    finder: Option<Addr>,
) -> StdResult<SwapResponse> {
    let config = CONFIG.load(deps.storage)?;
    let marketplace_params = load_marketplace_params(deps, &config.marketplace_addr)
        .map_err(|_| StdError::generic_err("Marketplace not found"))?;

    let collection_royalties = load_collection_royalties(deps, &collection)
        .map_err(|_| StdError::generic_err("Collection not found"))?;

    let mut processor = SwapProcessor::new(
        TransactionType::Buy,
        collection,
        nft_recipient.clone(),
        Uint128::zero(),
        nft_recipient,
        marketplace_params.params.trading_fee_percent,
        collection_royalties,
        finder,
        config.developer,
    );
    processor
        .swap_tokens_for_any_nfts(deps.storage, max_expected_token_input, swap_params)
        .map_err(|_| StdError::generic_err("swap_tokens_for_any_nfts err"))?;

    Ok(SwapResponse {
        swaps: processor.swaps,
    })
}
