use crate::{
    msg::{NftDepositsResponse, PoolConfigResponse, QueryMsg, QueryOptions},
    state::{NFT_DEPOSITS, POOL_CONFIG},
};

use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdResult};
use cw_storage_plus::{Bound, PrimaryKey};

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 100;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::PoolConfig {} => to_binary(&query_pool_config(deps)?),
        QueryMsg::NftDeposits {
            query_options,
        } => to_binary(&query_nft_deposits(deps, query_options)?),
    }
}

pub fn query_pool_config(deps: Deps) -> StdResult<PoolConfigResponse> {
    let config = POOL_CONFIG.load(deps.storage)?;
    Ok(PoolConfigResponse {
        config,
    })
}

pub fn query_nft_deposits(
    deps: Deps,
    query_options: Option<QueryOptions<String>>,
) -> StdResult<NftDepositsResponse> {
    let (limit, order, min, max) = unpack_query_options(query_options, |sa| Bound::exclusive(sa));

    let nft_deposits: Vec<String> = NFT_DEPOSITS
        .range(deps.storage, min, max, order)
        .take(limit)
        .map(|item| item.map(|(v, _)| v))
        .collect::<StdResult<_>>()?;

    Ok(NftDepositsResponse {
        nft_deposits,
    })
}

pub fn unpack_query_options<'a, T: PrimaryKey<'a>, U>(
    query_options: Option<QueryOptions<U>>,
    start_after_fn: fn(U) -> Bound<'a, T>,
) -> (usize, Order, Option<Bound<'a, T>>, Option<Bound<'a, T>>) {
    if query_options.is_none() {
        return (DEFAULT_QUERY_LIMIT as usize, Order::Ascending, None, None);
    }
    let query_options = query_options.unwrap();

    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

    let mut order = Order::Ascending;
    if let Some(descending) = query_options.descending {
        if descending {
            order = Order::Descending;
        }
    };

    let (mut min, mut max) = (None, None);
    let mut bound = None;
    if let Some(start_after) = query_options.start_after {
        bound = Some(start_after_fn(start_after));
    };
    match order {
        Order::Ascending => min = bound,
        Order::Descending => max = bound,
    };

    (limit, order, min, max)
}
