use crate::{
    helpers::load_pool,
    msg::{NftDepositsResponse, PoolConfigResponse, PoolQuoteResponse, QueryMsg},
    state::{INFINITY_GLOBAL, NFT_DEPOSITS, POOL_CONFIG},
};

use cosmwasm_std::{ensure, to_binary, Binary, Deps, Env, StdError, StdResult};
use cw_storage_plus::Bound;
use infinity_shared::{
    global::load_global_config,
    query::{unpack_query_options, QueryOptions},
};

// Query limits
pub const DEFAULT_QUERY_LIMIT: u32 = 10;
pub const MAX_QUERY_LIMIT: u32 = 100;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::PoolConfig {} => to_binary(&query_pool_config(deps)?),
        QueryMsg::NftDeposits {
            query_options,
        } => to_binary(&query_nft_deposits(deps, query_options)?),
        QueryMsg::BuyFromPoolQuote {} => to_binary(&query_buy_from_pool_quote(deps, env)?),
        QueryMsg::SellToPoolQuote {} => to_binary(&query_sell_to_pool_quote(deps, env)?),
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

pub fn query_buy_from_pool_quote(deps: Deps, env: Env) -> StdResult<PoolQuoteResponse> {
    let global_config = load_global_config(&deps.querier, &INFINITY_GLOBAL.load(deps.storage)?)?;

    let pool = load_pool(&env.contract.address, deps.storage, &deps.querier)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    ensure!(pool.can_escrow_nfts(), StdError::generic_err("pool cannot escrow NFTs".to_string()));

    let quote_price = pool.get_buy_from_pool_quote(global_config.min_price).ok();

    Ok(PoolQuoteResponse {
        quote_price,
    })
}

pub fn query_sell_to_pool_quote(deps: Deps, env: Env) -> StdResult<PoolQuoteResponse> {
    let global_config = load_global_config(&deps.querier, &INFINITY_GLOBAL.load(deps.storage)?)?;

    let pool = load_pool(&env.contract.address, deps.storage, &deps.querier)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    ensure!(
        pool.can_escrow_tokens(),
        StdError::generic_err("pool cannot escrow tokens".to_string())
    );

    let quote_price = pool.get_sell_to_pool_quote(global_config.min_price).ok();

    Ok(PoolQuoteResponse {
        quote_price,
    })
}
