use std::collections::BTreeMap;

use crate::helpers::{load_collection_royalties, load_marketplace_params, option_bool_to_order};
use crate::msg::{
    ConfigResponse, PoolQuoteResponse, PoolsByIdResponse, PoolsResponse, QueryMsg, QueryOptions,
    SwapNft, SwapParams,
};
use crate::state::{buy_pool_quotes, pools, sell_pool_quotes, Config, Pool, PoolQuote, CONFIG};
use crate::swap_processor::{Swap, SwapProcessor};
use crate::ContractError;
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, Env, Order, StdError, StdResult, Uint128,
};
use cw_storage_plus::Bound;

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
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
        } => to_binary(&query_pools_by_buy_price(
            deps,
            api.addr_validate(&collection)?,
            query_options,
        )?),
        QueryMsg::PoolQuotesSell {
            collection,
            query_options,
        } => to_binary(&query_pools_by_sell_price(
            deps,
            api.addr_validate(&collection)?,
            query_options,
        )?),
        QueryMsg::SimDirectSwapNftForTokens {
            pool_id,
            swap_nfts,
            swap_params,
            token_recipient,
        } => to_binary(&query_direct_swap_nft_for_tokens(
            deps,
            env,
            pool_id,
            swap_nfts,
            swap_params,
            api.addr_validate(&token_recipient)?,
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

pub fn query_pools_by_buy_price(
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

pub fn query_pools_by_sell_price(
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

pub fn query_direct_swap_nft_for_tokens(
    deps: Deps,
    env: Env,
    pool_id: u64,
    swap_nfts: Vec<SwapNft>,
    swap_params: SwapParams,
    asset_recipient: Addr,
) -> StdResult<Vec<Swap>> {
    let config = CONFIG.load(deps.storage)?;

    // convert to StdErr
    let marketplace_params = load_marketplace_params(deps, &config.marketplace_addr)
        .map_err(|_| StdError::generic_err("Marketplace not found"))?;

    let pool = pools().load(deps.storage, pool_id)?;

    let collection_royalties = load_collection_royalties(deps, &pool.collection)
        .map_err(|_| StdError::generic_err("Collection not found"))?;

    let mut processor = SwapProcessor::new(
        pool.collection.clone(),
        asset_recipient,
        marketplace_params.params.trading_fee_percent,
        collection_royalties,
    );
    processor
        .direct_swap_nft_for_tokens(deps, env, pool_id, swap_nfts, swap_params)
        .map_err(|_| StdError::generic_err("direct_swap_nft_for_tokens err"))?;

    Ok(processor.swaps)
}
