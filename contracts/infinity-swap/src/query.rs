use crate::helpers::{option_bool_to_pair, prep_for_swap};
use crate::msg::{
    ConfigResponse, NftSwap, NftTokenIdsResponse, PairNftSwap, PairQuoteResponse,
    PairsByIdResponse, PairsResponse, QueryMsg, QueryOptions, SwapParams, SwapResponse,
    TransactionType,
};
use crate::state::{
    buy_from_pair_quotes, pairs, sell_to_pair_quotes, PairQuote, CONFIG, NFT_DEPOSITS,
};
use crate::swap_processor::SwapProcessor;
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, Env, Pair, StdError, StdResult, Uint128,
};
use cw_storage_plus::Bound;

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Pairs { query_options } => to_binary(&query_pairs(deps, query_options)?),
        QueryMsg::PairsById { pair_ids } => to_binary(&query_pairs_by_id(deps, pair_ids)?),
        QueryMsg::PairsByOwner {
            owner,
            query_options,
        } => to_binary(&query_pairs_by_owner(
            deps,
            api.addr_validate(&owner)?,
            query_options,
        )?),
        QueryMsg::PairNftTokenIds {
            pair_id,
            query_options,
        } => to_binary(&query_pair_nft_token_ids(deps, pair_id, query_options)?),
        QueryMsg::QuotesBuyFromPair {
            collection,
            query_options,
        } => to_binary(&query_quotes_buy_from_pair(
            deps,
            api.addr_validate(&collection)?,
            query_options,
        )?),
        QueryMsg::QuotesSellToPair {
            collection,
            query_options,
        } => to_binary(&query_quotes_sell_to_pair(
            deps,
            api.addr_validate(&collection)?,
            query_options,
        )?),
        QueryMsg::SimDirectSwapNftsForTokens {
            pair_id,
            nfts_to_swap,
            sender,
            swap_params,
        } => to_binary(&sim_direct_swap_nfts_for_tokens(
            deps,
            env,
            pair_id,
            nfts_to_swap,
            api.addr_validate(&sender)?,
            swap_params,
        )?),
        QueryMsg::SimSwapNftsForTokens {
            collection,
            nfts_to_swap,
            sender,
            swap_params,
        } => to_binary(&sim_swap_nfts_for_tokens(
            deps,
            env,
            api.addr_validate(&collection)?,
            nfts_to_swap,
            api.addr_validate(&sender)?,
            swap_params,
        )?),
        QueryMsg::SimDirectSwapTokensForSpecificNfts {
            pair_id,
            nfts_to_swap_for,
            sender,
            swap_params,
        } => to_binary(&sim_direct_swap_tokens_for_specific_nfts(
            deps,
            env,
            pair_id,
            nfts_to_swap_for,
            api.addr_validate(&sender)?,
            swap_params,
        )?),
        QueryMsg::SimSwapTokensForSpecificNfts {
            collection,
            pair_nfts_to_swap_for,
            sender,
            swap_params,
        } => to_binary(&sim_swap_tokens_for_specific_nfts(
            deps,
            env,
            api.addr_validate(&collection)?,
            pair_nfts_to_swap_for,
            api.addr_validate(&sender)?,
            swap_params,
        )?),
        QueryMsg::SimSwapTokensForAnyNfts {
            collection,
            max_expected_token_input,
            sender,
            swap_params,
        } => to_binary(&sim_swap_tokens_for_any_nfts(
            deps,
            env,
            api.addr_validate(&collection)?,
            max_expected_token_input,
            api.addr_validate(&sender)?,
            swap_params,
        )?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

pub fn query_pairs(deps: Deps, query_options: QueryOptions<u64>) -> StdResult<PairsResponse> {
    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.map(Bound::exclusive);
    let pair = option_bool_to_pair(query_options.descending, Pair::Ascending);

    let pairs = pairs()
        .range(deps.storage, start, None, pair)
        .take(limit)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;

    Ok(PairsResponse { pairs })
}

pub fn query_pairs_by_id(deps: Deps, pair_ids: Vec<u64>) -> StdResult<PairsByIdResponse> {
    let mut resp_vec = vec![];
    for pair_id in pair_ids {
        let pair = pairs().may_load(deps.storage, pair_id)?;
        resp_vec.push((pair_id, pair));
    }
    Ok(PairsByIdResponse { pairs: resp_vec })
}

pub fn query_pairs_by_owner(
    deps: Deps,
    owner: Addr,
    query_options: QueryOptions<u64>,
) -> StdResult<PairsResponse> {
    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.map(Bound::exclusive);
    let pair = option_bool_to_pair(query_options.descending, Pair::Ascending);

    let pairs = pairs()
        .idx
        .owner
        .prefix(owner)
        .range(deps.storage, start, None, pair)
        .take(limit)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;

    Ok(PairsResponse { pairs })
}

pub fn query_pair_nft_token_ids(
    deps: Deps,
    pair_id: u64,
    query_options: QueryOptions<String>,
) -> StdResult<NftTokenIdsResponse> {
    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT) as usize;

    let start = query_options.start_after.as_ref().map(Bound::exclusive);
    let pair = option_bool_to_pair(query_options.descending, Pair::Ascending);

    let nft_token_ids: Vec<String> = NFT_DEPOSITS
        .prefix(pair_id)
        .range(deps.storage, start, None, pair)
        .take(limit)
        .map(|item| item.map(|(nft_token_id, _)| nft_token_id))
        .collect::<StdResult<_>>()?;

    Ok(NftTokenIdsResponse {
        pair_id,
        nft_token_ids,
    })
}

pub fn query_quotes_buy_from_pair(
    deps: Deps,
    collection: Addr,
    query_options: QueryOptions<(Uint128, u64)>,
) -> StdResult<PairQuoteResponse> {
    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT) as usize;
    let start = query_options
        .start_after
        .as_ref()
        .map(|offset| Bound::exclusive((offset.0.u128(), offset.1)));
    let pair = option_bool_to_pair(query_options.descending, Pair::Ascending);

    let pair_quotes: Vec<PairQuote> = buy_from_pair_quotes()
        .idx
        .collection_buy_price
        .sub_prefix(collection)
        .range(deps.storage, start, None, pair)
        .take(limit)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;

    Ok(PairQuoteResponse { pair_quotes })
}

pub fn query_quotes_sell_to_pair(
    deps: Deps,
    collection: Addr,
    query_options: QueryOptions<(Uint128, u64)>,
) -> StdResult<PairQuoteResponse> {
    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT) as usize;
    let start = query_options
        .start_after
        .as_ref()
        .map(|offset| Bound::exclusive((offset.0.u128(), offset.1)));
    let pair = option_bool_to_pair(query_options.descending, Pair::Descending);

    let pair_quotes: Vec<PairQuote> = sell_to_pair_quotes()
        .idx
        .collection_sell_price
        .sub_prefix(collection)
        .range(deps.storage, start, None, pair)
        .take(limit)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;

    Ok(PairQuoteResponse { pair_quotes })
}

pub fn sim_direct_swap_nfts_for_tokens(
    deps: Deps,
    env: Env,
    pair_id: u64,
    nfts_to_swap: Vec<NftSwap>,
    sender: Addr,
    swap_params: SwapParams,
) -> StdResult<SwapResponse> {
    let pair = pairs().load(deps.storage, pair_id)?;

    let swap_prep_result = prep_for_swap(deps, &None, &sender, &pair.collection, &swap_params)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    let mut processor = SwapProcessor::new(
        TransactionType::UserSubmitsNfts,
        env.contract.address,
        pair.collection.clone(),
        sender,
        Uint128::zero(),
        swap_prep_result.asset_recipient,
        swap_prep_result
            .marketplace_params
            .params
            .trading_fee_percent,
        swap_prep_result.marketplace_params.params.min_price,
        swap_prep_result.collection_royalties,
        swap_prep_result.finder,
        swap_prep_result.developer,
    );
    processor
        .direct_swap_nfts_for_tokens(pair, nfts_to_swap, swap_params)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    Ok(SwapResponse {
        swaps: processor.swaps,
    })
}

pub fn sim_swap_nfts_for_tokens(
    deps: Deps,
    env: Env,
    collection: Addr,
    nfts_to_swap: Vec<NftSwap>,
    sender: Addr,
    swap_params: SwapParams,
) -> StdResult<SwapResponse> {
    let swap_prep_result = prep_for_swap(deps, &None, &sender, &collection, &swap_params)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    let mut processor = SwapProcessor::new(
        TransactionType::UserSubmitsNfts,
        env.contract.address,
        collection,
        sender,
        Uint128::zero(),
        swap_prep_result.asset_recipient,
        swap_prep_result
            .marketplace_params
            .params
            .trading_fee_percent,
        swap_prep_result.marketplace_params.params.min_price,
        swap_prep_result.collection_royalties,
        swap_prep_result.finder,
        swap_prep_result.developer,
    );
    processor
        .swap_nfts_for_tokens(deps.storage, nfts_to_swap, swap_params)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    Ok(SwapResponse {
        swaps: processor.swaps,
    })
}

pub fn sim_direct_swap_tokens_for_specific_nfts(
    deps: Deps,
    env: Env,
    pair_id: u64,
    nfts_to_swap_for: Vec<NftSwap>,
    sender: Addr,
    swap_params: SwapParams,
) -> StdResult<SwapResponse> {
    let pair = pairs().load(deps.storage, pair_id)?;

    sim_swap_tokens_for_specific_nfts(
        deps,
        env,
        pair.collection,
        vec![PairNftSwap {
            pair_id,
            nft_swaps: nfts_to_swap_for,
        }],
        sender,
        swap_params,
    )
}

pub fn sim_swap_tokens_for_specific_nfts(
    deps: Deps,
    env: Env,
    collection: Addr,
    pair_nfts_to_swap_for: Vec<PairNftSwap>,
    sender: Addr,
    swap_params: SwapParams,
) -> StdResult<SwapResponse> {
    let swap_prep_result = prep_for_swap(deps, &None, &sender, &collection, &swap_params)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    let mut spend_amount = 0_u128;
    for pair in pair_nfts_to_swap_for.iter() {
        for nft_swap in pair.nft_swaps.iter() {
            spend_amount += nft_swap.token_amount.u128();
        }
    }

    let mut processor = SwapProcessor::new(
        TransactionType::UserSubmitsTokens,
        env.contract.address,
        collection,
        sender,
        spend_amount.into(),
        swap_prep_result.asset_recipient,
        swap_prep_result
            .marketplace_params
            .params
            .trading_fee_percent,
        swap_prep_result.marketplace_params.params.min_price,
        swap_prep_result.collection_royalties,
        swap_prep_result.finder,
        swap_prep_result.developer,
    );
    processor
        .swap_tokens_for_specific_nfts(deps.storage, pair_nfts_to_swap_for, swap_params)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    Ok(SwapResponse {
        swaps: processor.swaps,
    })
}

pub fn sim_swap_tokens_for_any_nfts(
    deps: Deps,
    env: Env,
    collection: Addr,
    max_expected_token_input: Vec<Uint128>,
    sender: Addr,
    swap_params: SwapParams,
) -> StdResult<SwapResponse> {
    let swap_prep_result = prep_for_swap(deps, &None, &sender, &collection, &swap_params)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    let total_tokens: Uint128 = max_expected_token_input.iter().sum();

    let mut processor = SwapProcessor::new(
        TransactionType::UserSubmitsTokens,
        env.contract.address,
        collection,
        sender,
        total_tokens,
        swap_prep_result.asset_recipient,
        swap_prep_result
            .marketplace_params
            .params
            .trading_fee_percent,
        swap_prep_result.marketplace_params.params.min_price,
        swap_prep_result.collection_royalties,
        swap_prep_result.finder,
        swap_prep_result.developer,
    );
    processor
        .swap_tokens_for_any_nfts(deps.storage, max_expected_token_input, swap_params)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    Ok(SwapResponse {
        swaps: processor.swaps,
    })
}
