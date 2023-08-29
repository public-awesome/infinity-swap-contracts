use crate::msg::{QueryMsg, QuotesResponse};
use crate::nfts_for_tokens_iterators::{
    iter::NftsForTokens,
    types::{NftForTokensQuote, NftForTokensSource},
};
use crate::state::INFINITY_GLOBAL;
use crate::tokens_for_nfts_iterators::{
    iter::TokensForNfts,
    types::{TokensForNftQuote, TokensForNftSource},
};

use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, StdError, StdResult, Uint128};
use infinity_pair::helpers::load_payout_context;
use infinity_pair::pair::Pair;

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
        QueryMsg::SimSellToPairQuotes {
            pair,
            limit,
        } => to_binary(&query_sell_to_pair_quotes(deps, pair, limit)?),
        QueryMsg::SimBuyFromPairQuotes {
            pair,
            limit,
        } => to_binary(&query_buy_from_pair_quotes(deps, pair, limit)?),
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
    let iterator =
        NftsForTokens::initialize(deps, &infinity_global, &collection, &denom, filter_sources)
            .map_err(|e| StdError::generic_err(e.to_string()))?;

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
    let iterator =
        TokensForNfts::initialize(deps, &infinity_global, &collection, &denom, filter_sources);

    let result = iterator.take(limit as usize).collect::<Vec<TokensForNftQuote>>();

    Ok(result)
}

pub fn query_sell_to_pair_quotes(
    deps: Deps,
    mut pair: Pair,
    limit: u32,
) -> StdResult<QuotesResponse> {
    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;
    let payout_context = load_payout_context(
        deps,
        &infinity_global,
        &pair.immutable.collection,
        &pair.immutable.denom,
    )
    .map_err(|_| StdError::generic_err("failed to load payout context".to_string()))?;

    let mut quotes: Vec<Uint128> = vec![];

    let mut idx = 0u32;
    while idx < limit {
        if let Some(quote_summary) = &pair.internal.sell_to_pair_quote_summary {
            quotes.push(quote_summary.seller_amount);
        } else {
            break;
        }

        pair.sim_swap_nft_for_tokens(&payout_context);

        idx += 1;
    }

    Ok(QuotesResponse {
        denom: pair.immutable.denom,
        quotes,
    })
}

pub fn query_buy_from_pair_quotes(
    deps: Deps,
    mut pair: Pair,
    limit: u32,
) -> StdResult<QuotesResponse> {
    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;
    let payout_context = load_payout_context(
        deps,
        &infinity_global,
        &pair.immutable.collection,
        &pair.immutable.denom,
    )
    .map_err(|_| StdError::generic_err("failed to load payout context".to_string()))?;

    let mut quotes: Vec<Uint128> = vec![];

    let mut idx = 0u32;
    while idx < limit {
        if let Some(quote_summary) = &pair.internal.buy_from_pair_quote_summary {
            quotes.push(quote_summary.total());
        } else {
            break;
        }

        pair.sim_swap_tokens_for_nft(&payout_context);

        idx += 1;
    }

    Ok(QuotesResponse {
        denom: pair.immutable.denom,
        quotes,
    })
}
