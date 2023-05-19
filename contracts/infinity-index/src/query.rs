use crate::msg::{PoolQuotesResponse, QueryMsg};
use crate::state::{sell_to_pool_quotes, PoolQuote};

use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, StdResult};
use cw_storage_plus::Bound;
use infinity_shared::query::{unpack_query_options, QueryOptions};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::BuyFromPoolQuotes {
            collection,
            query_options,
        } => to_binary(&query_buy_from_pool_quotes(
            deps,
            deps.api.addr_validate(&collection)?,
            query_options,
        )?),
        QueryMsg::SellToPoolQuotes {
            collection,
            query_options,
        } => to_binary(&query_sell_to_pool_quotes(
            deps,
            deps.api.addr_validate(&collection)?,
            query_options,
        )?),
    }
}

pub fn query_buy_from_pool_quotes(
    deps: Deps,
    collection: Addr,
    query_options: Option<QueryOptions<(u128, String)>>,
) -> StdResult<PoolQuotesResponse> {
    let (limit, order, min, max) = unpack_query_options(query_options, |sa| {
        Bound::exclusive((sa.0, deps.api.addr_validate(&sa.1).unwrap()))
    });

    let pool_quotes: Vec<PoolQuote> = sell_to_pool_quotes()
        .idx
        .collection_quote_price
        .sub_prefix(collection)
        .range(deps.storage, min, max, order)
        .take(limit as usize)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;

    Ok(PoolQuotesResponse {
        pool_quotes,
    })
}

pub fn query_sell_to_pool_quotes(
    deps: Deps,
    collection: Addr,
    query_options: Option<QueryOptions<(u128, String)>>,
) -> StdResult<PoolQuotesResponse> {
    let (limit, order, min, max) = unpack_query_options(query_options, |sa| {
        Bound::exclusive((sa.0, deps.api.addr_validate(&sa.1).unwrap()))
    });

    let pool_quotes: Vec<PoolQuote> = sell_to_pool_quotes()
        .idx
        .collection_quote_price
        .sub_prefix(collection)
        .range(deps.storage, min, max, order)
        .take(limit as usize)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;

    Ok(PoolQuotesResponse {
        pool_quotes,
    })
}
