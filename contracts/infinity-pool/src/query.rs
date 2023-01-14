use crate::msg::{QueryMsg, QueryOptions};
use crate::state::{CONFIG, Config, pools, Pool, PoolQuote, buy_pool_quotes, sell_pool_quotes};
use crate::helpers::{option_bool_to_order};
use cosmwasm_std::{entry_point, to_binary, Binary, Deps, Env, StdResult, Addr};
use cw_storage_plus::Bound;

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Pool { pool_id } => to_binary(&query_pool(deps, pool_id)?),
        QueryMsg::Pools { query_options } => to_binary(&query_pools(deps, query_options)?),
        QueryMsg::PoolsByOwner {
            owner,
            query_options
        } => to_binary(&query_pools_by_owner(
            deps,
            api.addr_validate(&owner)?,
            query_options,
        )?),
        QueryMsg::PoolsByBuyPrice {
            collection,
            query_options,
        } => to_binary(&query_pools_by_buy_price(
            deps,
            api.addr_validate(&collection)?,
            query_options,
        )?),
        QueryMsg::PoolsBySellPrice {
            collection,
            query_options,
        } => to_binary(&query_pools_by_sell_price(
            deps,
            api.addr_validate(&collection)?,
            query_options,
        )?),
    }
}

fn query_config(deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(deps.storage)?;
    Ok(Config {
        denom: config.denom,
        marketplace_addr: config.marketplace_addr,
    })
}

fn query_pool(deps: Deps, pool_id: u64) -> StdResult<Option<Pool>> {
    let pool = pools().may_load(deps.storage, pool_id)?;
    Ok(pool)
}

fn query_pools(
    deps: Deps,
    query_options: QueryOptions<u64>
) -> StdResult<Vec<Pool>> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.map(Bound::exclusive);
    let order = option_bool_to_order(query_options.descending);

    let pools = pools()
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;

    Ok(pools)
}

fn query_pools_by_owner(
    deps: Deps,
    owner: Addr,
    query_options: QueryOptions<u64>,
) -> StdResult<Vec<Pool>> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.map(Bound::exclusive);
    let order = option_bool_to_order(query_options.descending);

    let pools = pools()
        .idx
        .owner
        .prefix(owner)
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;

    Ok(pools)
}

fn query_pools_by_buy_price(
    deps: Deps,
    collection: Addr,
    query_options: QueryOptions<u64>,
) -> StdResult<Vec<Pool>> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.map(Bound::exclusive);
    let order = option_bool_to_order(query_options.descending);

    let pool_quotes: Vec<PoolQuote> = buy_pool_quotes()
        .idx
        .collection_buy_price
        .prefix((collection, 0u128))
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;
    
    let mut return_pools = vec![];
    for pool_quote in pool_quotes {
        let pool = pools().load(deps.storage, pool_quote.id)?;
        return_pools.push(pool);
    }

    Ok(return_pools)
}

fn query_pools_by_sell_price(
    deps: Deps,
    collection: Addr,
    query_options: QueryOptions<u64>,
) -> StdResult<Vec<Pool>> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.map(Bound::exclusive);
    let order = option_bool_to_order(query_options.descending);

    let pool_quotes: Vec<PoolQuote> = sell_pool_quotes()
        .idx
        .collection_sell_price
        .prefix((collection, 0u128))
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;
    
    let mut return_pools = vec![];
    for pool_quote in pool_quotes {
        let pool = pools().load(deps.storage, pool_quote.id)?;
        return_pools.push(pool);
    }

    Ok(return_pools)
}