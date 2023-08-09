use crate::msg::QueryMsg;
use crate::nfts_for_tokens_iterators::{NftForTokensQuote, NftForTokensSource, NftsForTokens};
use crate::tokens_for_nfts_iterators::{TokensForNftQuote, TokensForNftSource, TokensForNfts};

use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, StdResult};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
        QueryMsg::NftsForTokens {
            collection,
            denom,
            limit,
            filter_sources,
        } => to_binary(&query_nfts_for_tokens(
            deps,
            env,
            api.addr_validate(&collection)?,
            denom,
            limit,
            filter_sources.unwrap_or_default(),
        )?),
        QueryMsg::TokensForNfts {
            collection,
            denom,
            limit,
            filter_sources,
        } => to_binary(&query_tokens_for_nfts(
            deps,
            env,
            api.addr_validate(&collection)?,
            denom,
            limit,
            filter_sources.unwrap_or_default(),
        )?),
    }
}

pub fn query_nfts_for_tokens(
    deps: Deps,
    _env: Env,
    collection: Addr,
    denom: String,
    limit: u32,
    filter_sources: Vec<NftForTokensSource>,
) -> StdResult<Vec<NftForTokensQuote>> {
    let iterator = NftsForTokens::initialize(deps, collection, denom, filter_sources);

    let result = iterator.take(limit as usize).collect::<Vec<NftForTokensQuote>>();

    Ok(result)
}

pub fn query_tokens_for_nfts(
    deps: Deps,
    _env: Env,
    collection: Addr,
    denom: String,
    limit: u32,
    filter_sources: Vec<TokensForNftSource>,
) -> StdResult<Vec<TokensForNftQuote>> {
    let iterator = TokensForNfts::initialize(deps, collection, denom, filter_sources);

    let result = iterator.take(limit as usize).collect::<Vec<TokensForNftQuote>>();

    Ok(result)
}
