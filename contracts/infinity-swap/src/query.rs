use crate::helpers::{option_bool_to_order, prep_for_swap};
use crate::msg::{
    ConfigResponse, NftTokenIdsResponse, PoolQuoteResponse, PoolsByIdResponse, PoolsResponse,
    QueryMsg, QueryOptions,
};
use crate::state::{
    buy_from_pool_quotes, nft_deposits, pools, sell_to_pool_quotes, NftDeposit, PoolQuote, CONFIG,
};
use crate::swap_processor::SwapProcessor;
use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, Env, Order, StdError, StdResult, Uint128,
};
use cw_storage_plus::Bound;
use infinity_shared::interface::{
    transform_swap_params, NftOrder, SwapParamsInternal, SwapResponse, TransactionType,
};

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 100;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Pools { query_options } => to_binary(&query_pools(deps, query_options)?),
        QueryMsg::PoolsById { pool_ids } => to_binary(&query_pools_by_id(deps, pool_ids)?),
        QueryMsg::PoolsByOwner {
            owner,
            query_options,
        } => to_binary(&query_pools_by_owner(
            deps,
            api.addr_validate(&owner)?,
            query_options,
        )?),
        QueryMsg::PoolNftTokenIds {
            pool_id,
            query_options,
        } => to_binary(&query_pool_nft_token_ids(deps, pool_id, query_options)?),
        QueryMsg::QuotesBuyFromPool {
            collection,
            query_options,
        } => to_binary(&query_quotes_buy_from_pool(
            deps,
            api.addr_validate(&collection)?,
            query_options,
        )?),
        QueryMsg::QuotesSellToPool {
            collection,
            query_options,
        } => to_binary(&query_quotes_sell_to_pool(
            deps,
            api.addr_validate(&collection)?,
            query_options,
        )?),
        QueryMsg::SimDirectSwapNftsForTokens {
            sender,
            pool_id,
            nft_orders,
            swap_params,
        } => to_binary(&sim_direct_swap_nfts_for_tokens(
            deps,
            env,
            api.addr_validate(&sender)?,
            pool_id,
            nft_orders,
            transform_swap_params(api, swap_params)?,
        )?),
        QueryMsg::SimSwapNftsForTokens {
            sender,
            collection,
            nft_orders,
            swap_params,
        } => to_binary(&sim_swap_nfts_for_tokens(
            deps,
            env,
            api.addr_validate(&sender)?,
            api.addr_validate(&collection)?,
            nft_orders,
            transform_swap_params(api, swap_params)?,
        )?),
        QueryMsg::SimSwapTokensForSpecificNfts {
            sender,
            collection,
            nft_orders,
            swap_params,
        } => to_binary(&sim_swap_tokens_for_specific_nfts(
            deps,
            env,
            api.addr_validate(&sender)?,
            api.addr_validate(&collection)?,
            nft_orders,
            transform_swap_params(api, swap_params)?,
        )?),
        QueryMsg::SimSwapTokensForAnyNfts {
            sender,
            collection,
            orders,
            swap_params,
        } => to_binary(&sim_swap_tokens_for_any_nfts(
            deps,
            env,
            api.addr_validate(&collection)?,
            orders,
            api.addr_validate(&sender)?,
            transform_swap_params(api, swap_params)?,
        )?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse { config })
}

pub fn query_pools(deps: Deps, query_options: QueryOptions<u64>) -> StdResult<PoolsResponse> {
    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.map(Bound::exclusive);
    let order = option_bool_to_order(query_options.descending, Order::Ascending);

    let pools = pools()
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;

    Ok(PoolsResponse { pools })
}

pub fn query_pools_by_id(deps: Deps, pool_ids: Vec<u64>) -> StdResult<PoolsByIdResponse> {
    let mut resp_vec = vec![];
    for pool_id in pool_ids {
        let pool = pools().may_load(deps.storage, pool_id)?;
        resp_vec.push((pool_id, pool));
    }
    Ok(PoolsByIdResponse { pools: resp_vec })
}

pub fn query_pools_by_owner(
    deps: Deps,
    owner: Addr,
    query_options: QueryOptions<u64>,
) -> StdResult<PoolsResponse> {
    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.map(Bound::exclusive);
    let order = option_bool_to_order(query_options.descending, Order::Ascending);

    let pools = pools()
        .idx
        .owner
        .prefix(owner)
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;

    Ok(PoolsResponse { pools })
}

pub fn query_pool_nft_token_ids(
    deps: Deps,
    pool_id: u64,
    query_options: QueryOptions<String>,
) -> StdResult<NftTokenIdsResponse> {
    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT) as usize;

    let pool = pools().load(deps.storage, pool_id)?;

    let start = query_options
        .start_after
        .as_ref()
        .map(|token_id| Bound::exclusive((pool.collection.clone(), token_id.to_string())));
    let order = option_bool_to_order(query_options.descending, Order::Ascending);

    let nft_deposits: Vec<NftDeposit> = nft_deposits()
        .idx
        .pool_deposits
        .prefix(pool_id)
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|item| item.map(|(_, nft_deposit)| nft_deposit))
        .collect::<StdResult<_>>()?;

    let nft_token_ids: Vec<String> = nft_deposits
        .iter()
        .map(|nd| nd.token_id.to_string())
        .collect::<Vec<_>>();

    Ok(NftTokenIdsResponse {
        pool_id,
        collection: pool.collection.to_string(),
        nft_token_ids,
    })
}

pub fn query_quotes_buy_from_pool(
    deps: Deps,
    collection: Addr,
    query_options: QueryOptions<(Uint128, u64)>,
) -> StdResult<PoolQuoteResponse> {
    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT) as usize;
    let start = query_options
        .start_after
        .as_ref()
        .map(|offset| Bound::exclusive((offset.0.u128(), offset.1)));
    let order = option_bool_to_order(query_options.descending, Order::Ascending);

    let pool_quotes: Vec<PoolQuote> = buy_from_pool_quotes()
        .idx
        .collection_buy_price
        .sub_prefix(collection)
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;

    Ok(PoolQuoteResponse { pool_quotes })
}

pub fn query_quotes_sell_to_pool(
    deps: Deps,
    collection: Addr,
    query_options: QueryOptions<(Uint128, u64)>,
) -> StdResult<PoolQuoteResponse> {
    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT) as usize;
    let start = query_options
        .start_after
        .as_ref()
        .map(|offset| Bound::exclusive((offset.0.u128(), offset.1)));
    let order = option_bool_to_order(query_options.descending, Order::Descending);

    let pool_quotes: Vec<PoolQuote> = sell_to_pool_quotes()
        .idx
        .collection_sell_price
        .sub_prefix(collection)
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|item| item.map(|(_, v)| v))
        .collect::<StdResult<_>>()?;

    Ok(PoolQuoteResponse { pool_quotes })
}

pub fn sim_direct_swap_nfts_for_tokens(
    deps: Deps,
    env: Env,
    sender: Addr,
    pool_id: u64,
    nft_orders: Vec<NftOrder>,
    swap_params: SwapParamsInternal,
) -> StdResult<SwapResponse> {
    let pool = pools().load(deps.storage, pool_id)?;

    let swap_prep_result = prep_for_swap(deps, &None, &sender, &pool.collection, &swap_params)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    let mut processor = SwapProcessor::new(
        TransactionType::UserSubmitsNfts,
        env.contract.address,
        pool.collection.clone(),
        sender,
        Uint128::zero(),
        swap_prep_result.asset_recipient,
        swap_prep_result.marketplace_params.trading_fee_percent,
        swap_prep_result.marketplace_params.min_price,
        swap_prep_result.collection_royalties,
        swap_prep_result.finder,
        swap_prep_result.developer,
    );
    processor
        .direct_swap_nfts_for_tokens(pool, nft_orders, swap_params)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    Ok(SwapResponse {
        swaps: processor.swaps,
    })
}

pub fn sim_swap_nfts_for_tokens(
    deps: Deps,
    env: Env,
    sender: Addr,
    collection: Addr,
    nft_orders: Vec<NftOrder>,
    swap_params: SwapParamsInternal,
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
        swap_prep_result.marketplace_params.trading_fee_percent,
        swap_prep_result.marketplace_params.min_price,
        swap_prep_result.collection_royalties,
        swap_prep_result.finder,
        swap_prep_result.developer,
    );
    processor
        .swap_nfts_for_tokens(deps.storage, nft_orders, swap_params)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    Ok(SwapResponse {
        swaps: processor.swaps,
    })
}

pub fn sim_swap_tokens_for_specific_nfts(
    deps: Deps,
    env: Env,
    sender: Addr,
    collection: Addr,
    nft_orders: Vec<NftOrder>,
    swap_params: SwapParamsInternal,
) -> StdResult<SwapResponse> {
    let swap_prep_result = prep_for_swap(deps, &None, &sender, &collection, &swap_params)
        .map_err(|err| StdError::generic_err(err.to_string()))?;

    let spend_amount = nft_orders
        .iter()
        .fold(0u128, |acc, order| acc + order.amount.u128());

    let mut processor = SwapProcessor::new(
        TransactionType::UserSubmitsTokens,
        env.contract.address,
        collection,
        sender,
        spend_amount.into(),
        swap_prep_result.asset_recipient,
        swap_prep_result.marketplace_params.trading_fee_percent,
        swap_prep_result.marketplace_params.min_price,
        swap_prep_result.collection_royalties,
        swap_prep_result.finder,
        swap_prep_result.developer,
    );
    processor
        .swap_tokens_for_specific_nfts(deps.storage, nft_orders, swap_params)
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
    swap_params: SwapParamsInternal,
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
        swap_prep_result.marketplace_params.trading_fee_percent,
        swap_prep_result.marketplace_params.min_price,
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
