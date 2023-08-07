use crate::{helpers::load_pair, msg::QueryMsg, pair::Pair};

use cosmwasm_std::{to_binary, Binary, Deps, Env, StdError, StdResult};

// Query limits
pub const DEFAULT_QUERY_LIMIT: u32 = 10;
pub const MAX_QUERY_LIMIT: u32 = 100;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Pair {} => to_binary(&query_pair(deps, env)?),
    }
}

pub fn query_pair(deps: Deps, env: Env) -> StdResult<Pair> {
    let pair = load_pair(&env.contract.address, deps.storage, &deps.querier)
        .map_err(|_| StdError::generic_err("failed to load pair".to_string()))?;

    Ok(pair)
}

// pub fn query_nft_deposits(
//     deps: Deps,
//     query_options: Option<QueryOptions<String>>,
// ) -> StdResult<NftDepositsResponse> {
//     let (limit, order, min, max) = unpack_query_options(query_options, |sa| Bound::exclusive(sa));

//     let nft_deposits: Vec<String> = NFT_DEPOSITS
//         .range(deps.storage, min, max, order)
//         .take(limit)
//         .map(|item| item.map(|(v, _)| v))
//         .collect::<StdResult<_>>()?;

//     Ok(NftDepositsResponse {
//         nft_deposits,
//     })
// }

// pub fn query_buy_from_pair_quote(deps: Deps, env: Env) -> StdResult<PairQuoteResponse> {
//     let global_config = load_global_config(&deps.querier, &INFINITY_GLOBAL.load(deps.storage)?)?;

//     let pair = load_pair(&env.contract.address, deps.storage, &deps.querier)
//         .map_err(|err| StdError::generic_err(err.to_string()))?;

//     ensure!(pair.can_escrow_nfts(), StdError::generic_err("pair cannot escrow NFTs".to_string()));

//     let quote_price = pair.get_buy_from_pair_quote(global_config.min_price).ok();

//     Ok(PairQuoteResponse {
//         quote_price,
//     })
// }

// pub fn query_sell_to_pair_quote(deps: Deps, env: Env) -> StdResult<PairQuoteResponse> {
//     let global_config = load_global_config(&deps.querier, &INFINITY_GLOBAL.load(deps.storage)?)?;

//     let pair = load_pair(&env.contract.address, deps.storage, &deps.querier)
//         .map_err(|err| StdError::generic_err(err.to_string()))?;

//     ensure!(
//         pair.can_escrow_tokens(),
//         StdError::generic_err("pair cannot escrow tokens".to_string())
//     );

//     let quote_price = pair.get_sell_to_pair_quote(global_config.min_price).ok();

//     Ok(PairQuoteResponse {
//         quote_price,
//     })
// }
