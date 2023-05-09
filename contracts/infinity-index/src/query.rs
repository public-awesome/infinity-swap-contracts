use crate::msg::{PoolQuoteResponse, QueryMsg};
use crate::state::{sell_to_pool_quotes, PoolQuote};

use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, Order, StdResult};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::QuoteSellToPool {
            collection,
            limit,
        } => to_binary(&query_quotes_sell_to_pool(
            deps,
            deps.api.addr_validate(&collection)?,
            limit,
        )?),
    }
}

pub fn query_quotes_sell_to_pool(
    deps: Deps,
    collection: Addr,
    limit: u64,
) -> StdResult<PoolQuoteResponse> {
    let pool_quotes: Vec<PoolQuote> = sell_to_pool_quotes()
        .idx
        .collection_quote_price
        .sub_prefix(collection)
        .range(deps.storage, None, None, Order::Descending)
        .take(limit as usize)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;

    Ok(PoolQuoteResponse {
        pool_quotes,
    })
}
