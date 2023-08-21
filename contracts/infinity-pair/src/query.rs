use crate::{
    helpers::load_pair,
    msg::{NftDepositsResponse, QueryMsg},
    pair::Pair,
    state::{NFT_DEPOSITS, PAIR_IMMUTABLE},
};

use cosmwasm_std::{to_binary, Binary, Deps, Env, StdError, StdResult};
use sg_index_query::{QueryOptions, QueryOptionsInternal};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Pair {} => to_binary(&query_pair(deps, env)?),
        QueryMsg::NftDeposits {
            query_options,
        } => to_binary(&query_nft_deposits(deps, query_options.unwrap_or_default())?),
    }
}

pub fn query_pair(deps: Deps, env: Env) -> StdResult<Pair> {
    let pair = load_pair(&env.contract.address, deps.storage, &deps.querier)
        .map_err(|_| StdError::generic_err("failed to load pair".to_string()))?;

    Ok(pair)
}

pub fn query_nft_deposits(
    deps: Deps,
    query_options: QueryOptions<String>,
) -> StdResult<NftDepositsResponse> {
    let collection = PAIR_IMMUTABLE.load(deps.storage)?.collection;

    let QueryOptionsInternal {
        limit,
        order,
        min,
        max,
    } = query_options.unpack(&(|offset| offset.clone()), None, None);

    let token_ids = NFT_DEPOSITS
        .range(deps.storage, min, max, order)
        .take(limit)
        .map(|res| res.map(|(k, _)| k))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(NftDepositsResponse {
        collection,
        token_ids,
    })
}
