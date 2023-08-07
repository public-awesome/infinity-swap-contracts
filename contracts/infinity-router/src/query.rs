use crate::nfts_for_tokens_iterators::{NftForTokensQuote, NftForTokensSource, NftsForTokens};
use crate::tokens_for_nfts_iterators::{TokensForNftQuote, TokensForNftSource, TokensForNfts};
use crate::{msg::QueryMsg, state::INFINITY_GLOBAL};

use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, Response, StdResult};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use infinity_global::load_global_config;
use stargaze_royalty_registry::fetch_or_set_royalties;

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
    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;
    let global_config = load_global_config(&deps.querier, &infinity_global)?;
    let (royalty_entry, _) = fetch_or_set_royalties(
        deps,
        &global_config.royalty_registry,
        &collection,
        Some(&infinity_global),
        Response::new(),
    )
    .unwrap();

    let iterator = NftsForTokens::initialize(
        deps,
        global_config,
        collection,
        denom,
        royalty_entry,
        filter_sources,
    );

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
    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;
    let global_config = load_global_config(&deps.querier, &infinity_global)?;
    let (royalty_entry, _) = fetch_or_set_royalties(
        deps,
        &global_config.royalty_registry,
        &collection,
        Some(&infinity_global),
        Response::new(),
    )
    .unwrap();

    let iterator = TokensForNfts::initialize(
        deps,
        global_config,
        collection,
        denom,
        royalty_entry,
        filter_sources,
    );

    let result = iterator.take(limit as usize).collect::<Vec<TokensForNftQuote>>();

    Ok(result)
}
