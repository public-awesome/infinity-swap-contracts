use crate::error::ContractError;
use crate::helpers::{
    get_next_pool_counter, load_marketplace_params, only_owner, prep_for_swap, remove_nft_deposit,
    remove_pool, save_pool, save_pools, store_nft_deposit, transfer_nft, transfer_token,
    update_nft_deposits, validate_nft_swaps_for_buy, validate_nft_swaps_for_sell,
};
use crate::msg::{ExecuteMsg, NftSwap, PoolNftSwap, QueryOptions, SwapParams, TransactionType};
use crate::query::query_pool_nft_token_ids;
use crate::state::{pools, BondingCurve, Pool, PoolType, CONFIG};
use crate::swap_processor::{Swap, SwapProcessor};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{attr, coin, Addr, Decimal, DepsMut, Env, Event, MessageInfo, Uint128};
use cw_utils::{maybe_addr, must_pay, nonpayable};
use sg1::fair_burn;
use sg_std::{Response, NATIVE_DENOM};

/// A convenience struct for creating Pools
pub struct PoolInfo {
    pub collection: Addr,
    pub asset_recipient: Option<Addr>,
    pub pool_type: PoolType,
    pub bonding_curve: BondingCurve,
    pub spot_price: Uint128,
    pub delta: Uint128,
    pub finders_fee_percent: Decimal,
    pub swap_fee_percent: Decimal,
    pub reinvest_tokens: bool,
    pub reinvest_nfts: bool,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::CreateTokenPool {
            collection,
            asset_recipient,
            bonding_curve,
            spot_price,
            delta,
            finders_fee_bps,
        } => execute_create_pool(
            deps,
            info,
            PoolInfo {
                collection: api.addr_validate(&collection)?,
                asset_recipient: maybe_addr(api, asset_recipient)?,
                pool_type: PoolType::Token,
                bonding_curve,
                spot_price,
                delta,
                finders_fee_percent: Decimal::percent(finders_fee_bps),
                swap_fee_percent: Decimal::percent(0u64),
                reinvest_tokens: false,
                reinvest_nfts: false,
            },
        ),
        ExecuteMsg::CreateNftPool {
            collection,
            asset_recipient,
            bonding_curve,
            spot_price,
            delta,
            finders_fee_bps,
        } => execute_create_pool(
            deps,
            info,
            PoolInfo {
                collection: api.addr_validate(&collection)?,
                asset_recipient: maybe_addr(api, asset_recipient)?,
                pool_type: PoolType::Nft,
                bonding_curve,
                spot_price,
                delta,
                finders_fee_percent: Decimal::percent(finders_fee_bps),
                swap_fee_percent: Decimal::percent(0),
                reinvest_tokens: false,
                reinvest_nfts: false,
            },
        ),
        ExecuteMsg::CreateTradePool {
            collection,
            asset_recipient,
            bonding_curve,
            spot_price,
            delta,
            finders_fee_bps,
            swap_fee_bps,
            reinvest_tokens,
            reinvest_nfts,
        } => execute_create_pool(
            deps,
            info,
            PoolInfo {
                collection: api.addr_validate(&collection)?,
                asset_recipient: maybe_addr(api, asset_recipient)?,
                pool_type: PoolType::Trade,
                bonding_curve,
                spot_price,
                delta,
                finders_fee_percent: Decimal::percent(finders_fee_bps),
                swap_fee_percent: Decimal::percent(swap_fee_bps),
                reinvest_tokens,
                reinvest_nfts,
            },
        ),
        ExecuteMsg::DepositTokens { pool_id } => execute_deposit_tokens(deps, info, pool_id),
        ExecuteMsg::DepositNfts {
            pool_id,
            collection,
            nft_token_ids,
        } => execute_deposit_nfts(
            deps,
            info,
            env,
            pool_id,
            api.addr_validate(&collection)?,
            nft_token_ids,
        ),
        ExecuteMsg::WithdrawTokens {
            pool_id,
            amount,
            asset_recipient,
        } => execute_withdraw_tokens(
            deps,
            info,
            pool_id,
            amount,
            maybe_addr(api, asset_recipient)?,
        ),
        ExecuteMsg::WithdrawAllTokens {
            pool_id,
            asset_recipient,
        } => execute_withdraw_all_tokens(deps, info, pool_id, maybe_addr(api, asset_recipient)?),
        ExecuteMsg::WithdrawNfts {
            pool_id,
            nft_token_ids,
            asset_recipient,
        } => execute_withdraw_nfts(
            deps,
            info,
            pool_id,
            nft_token_ids,
            maybe_addr(api, asset_recipient)?,
        ),
        ExecuteMsg::WithdrawAllNfts {
            pool_id,
            asset_recipient,
        } => execute_withdraw_all_nfts(deps, info, pool_id, maybe_addr(api, asset_recipient)?),
        ExecuteMsg::UpdatePoolConfig {
            pool_id,
            asset_recipient,
            delta,
            spot_price,
            finders_fee_bps,
            swap_fee_bps,
            reinvest_tokens,
            reinvest_nfts,
        } => execute_update_pool_config(
            deps,
            info,
            pool_id,
            maybe_addr(api, asset_recipient)?,
            delta,
            spot_price,
            finders_fee_bps,
            swap_fee_bps,
            reinvest_tokens,
            reinvest_nfts,
        ),
        ExecuteMsg::SetActivePool { pool_id, is_active } => {
            execute_set_active_pool(deps, info, pool_id, is_active)
        }
        ExecuteMsg::RemovePool {
            pool_id,
            asset_recipient,
        } => execute_remove_pool(deps, info, pool_id, maybe_addr(api, asset_recipient)?),
        ExecuteMsg::DirectSwapNftsForTokens {
            pool_id,
            nfts_to_swap,
            swap_params,
        } => {
            execute_direct_swap_nfts_for_tokens(deps, env, info, pool_id, nfts_to_swap, swap_params)
        }
        ExecuteMsg::SwapNftsForTokens {
            collection,
            nfts_to_swap,
            swap_params,
        } => execute_swap_nfts_for_tokens(
            deps,
            env,
            info,
            api.addr_validate(&collection)?,
            nfts_to_swap,
            swap_params,
        ),
        ExecuteMsg::DirectSwapTokensForSpecificNfts {
            pool_id,
            nfts_to_swap_for,
            swap_params,
        } => execute_direct_swap_tokens_for_specific_nfts(
            deps,
            env,
            info,
            pool_id,
            nfts_to_swap_for,
            swap_params,
        ),
        ExecuteMsg::SwapTokensForSpecificNfts {
            collection,
            pool_nfts_to_swap_for,
            swap_params,
        } => execute_swap_tokens_for_specific_nfts(
            deps,
            env,
            info,
            api.addr_validate(&collection)?,
            pool_nfts_to_swap_for,
            swap_params,
        ),
        ExecuteMsg::SwapTokensForAnyNfts {
            collection,
            max_expected_token_input,
            swap_params,
        } => execute_swap_tokens_for_any_nfts(
            deps,
            env,
            info,
            api.addr_validate(&collection)?,
            max_expected_token_input,
            swap_params,
        ),
    }
}

/// Execute a CreatePool message
pub fn execute_create_pool(
    deps: DepsMut,
    info: MessageInfo,
    pool_info: PoolInfo,
) -> Result<Response, ContractError> {
    let pool_counter = get_next_pool_counter(deps.storage)?;
    let mut pool = Pool::new(
        pool_counter,
        pool_info.collection,
        info.sender.clone(),
        pool_info.asset_recipient,
        pool_info.pool_type,
        pool_info.bonding_curve,
        pool_info.spot_price,
        pool_info.delta,
        pool_info.finders_fee_percent,
        pool_info.swap_fee_percent,
        pool_info.reinvest_tokens,
        pool_info.reinvest_nfts,
    );

    let config = CONFIG.load(deps.storage)?;
    let marketplace_params = load_marketplace_params(deps.as_ref(), &config.marketplace_addr)?;

    let mut response = Response::new();
    let event = pool.create_event_all_props("create-pool")?;
    response = response.add_event(event);
    response = save_pool(deps.storage, &mut pool, &marketplace_params, response)?;

    // Burn the listing fee set on the marketplace contract
    let listing_fee = must_pay(&info, NATIVE_DENOM)?;
    if listing_fee != marketplace_params.params.listing_fee {
        return Err(ContractError::UnpaidListingFee(listing_fee));
    }
    if listing_fee > Uint128::zero() {
        fair_burn(listing_fee.u128(), config.developer, &mut response);
    }

    Ok(response)
}

/// Execute a DepositTokens message
pub fn execute_deposit_tokens(
    deps: DepsMut,
    info: MessageInfo,
    pool_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let received_amount = must_pay(&info, &config.denom)?;

    let mut pool = pools().load(deps.storage, pool_id)?;
    // Only the owner of the pool can deposit and withdraw assets
    only_owner(&info, &pool)?;

    // Track the total amount of tokens that have been deposited into the pool
    pool.deposit_tokens(received_amount)?;

    let config = CONFIG.load(deps.storage)?;
    let marketplace_params = load_marketplace_params(deps.as_ref(), &config.marketplace_addr)?;

    let mut response = Response::new();
    let event = pool.create_event("deposit-tokens", vec!["id", "spot_price", "total_tokens"])?;
    response = response.add_event(event);
    response = save_pool(deps.storage, &mut pool, &marketplace_params, response)?;
    Ok(response)
}

/// Execute a DepositNfts message
pub fn execute_deposit_nfts(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    pool_id: u64,
    collection: Addr,
    nft_token_ids: Vec<String>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut pool = pools().load(deps.storage, pool_id)?;
    // Only the owner of the pool can deposit and withdraw assets
    only_owner(&info, &pool)?;
    if pool.collection != collection {
        return Err(ContractError::InvalidInput(format!(
            "invalid collection ({}) for pool ({})",
            collection, pool.id
        )));
    }

    // Push the NFT transfer messages
    let mut response = Response::new();
    for nft_token_id in &nft_token_ids {
        transfer_nft(
            nft_token_id,
            env.contract.address.as_ref(),
            collection.as_ref(),
            &mut response,
        )?;
        store_nft_deposit(deps.storage, pool.id, nft_token_id)?;
    }
    // Track the NFTs that have been deposited into the pool
    pool.deposit_nfts(&nft_token_ids)?;

    let config = CONFIG.load(deps.storage)?;
    let marketplace_params = load_marketplace_params(deps.as_ref(), &config.marketplace_addr)?;

    let event = Event::new("deposit-nfts").add_attributes(vec![
        attr("pool_id", pool.id.to_string()),
        attr("total_nfts", pool.total_nfts.to_string()),
        attr("nft_token_ids", nft_token_ids.join(",")),
    ]);

    response = response.add_event(event);
    response = save_pool(deps.storage, &mut pool, &marketplace_params, response)?;
    Ok(response)
}

/// Execute a WithdrawTokens message
pub fn execute_withdraw_tokens(
    deps: DepsMut,
    info: MessageInfo,
    pool_id: u64,
    amount: Uint128,
    asset_recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut pool = pools().load(deps.storage, pool_id)?;
    // Only the owner of the pool can deposit and withdraw assets
    only_owner(&info, &pool)?;

    let mut response = Response::new();

    let config = CONFIG.load(deps.storage)?;
    // Withdraw tokens to the asset recipient if specified, otherwise to the sender
    let recipient = asset_recipient.unwrap_or(info.sender);
    transfer_token(
        coin(amount.u128(), config.denom),
        recipient.as_ref(),
        &mut response,
    )?;
    // Track total amount owned by the pool
    pool.withdraw_tokens(amount)?;

    let config = CONFIG.load(deps.storage)?;
    let marketplace_params = load_marketplace_params(deps.as_ref(), &config.marketplace_addr)?;

    let event = pool.create_event("withdraw-tokens", vec!["id", "spot_price", "total_tokens"])?;
    response = response.add_event(event);
    response = save_pool(deps.storage, &mut pool, &marketplace_params, response)?;
    Ok(response)
}

/// Execute a WithdrawAllNfts message, a convenvience method for withdrawing all tokens
pub fn execute_withdraw_all_tokens(
    deps: DepsMut,
    info: MessageInfo,
    pool_id: u64,
    asset_recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    let pool = pools().load(deps.storage, pool_id)?;
    execute_withdraw_tokens(deps, info, pool_id, pool.total_tokens, asset_recipient)
}

/// Execute a WithdrawNfts message
pub fn execute_withdraw_nfts(
    deps: DepsMut,
    info: MessageInfo,
    pool_id: u64,
    nft_token_ids: Vec<String>,
    asset_recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut pool = pools().load(deps.storage, pool_id)?;
    // Only the owner of the pool can deposit and withdraw assets
    only_owner(&info, &pool)?;

    let mut response = Response::new();

    // Withdraw NFTs to the asset recipient if specified, otherwise to the sender
    let recipient = asset_recipient.unwrap_or(info.sender);
    for nft_token_id in &nft_token_ids {
        transfer_nft(
            nft_token_id,
            recipient.as_ref(),
            pool.collection.as_ref(),
            &mut response,
        )?;
        remove_nft_deposit(deps.storage, pool_id, nft_token_id);
    }
    // Track the NFTs that have been withdrawn from the pool
    pool.withdraw_nfts(&nft_token_ids)?;

    let config = CONFIG.load(deps.storage)?;
    let marketplace_params = load_marketplace_params(deps.as_ref(), &config.marketplace_addr)?;

    let event = Event::new("withdraw-nfts").add_attributes(vec![
        attr("pool_id", pool.id.to_string()),
        attr("total_nfts", pool.total_nfts.to_string()),
        attr("nft_token_ids", nft_token_ids.join(",")),
    ]);
    response = response.add_event(event);
    response = save_pool(deps.storage, &mut pool, &marketplace_params, response)?;
    Ok(response)
}

/// Execute a WithdrawAllNfts message, a convenvience method for withdrawing all NFTs
pub fn execute_withdraw_all_nfts(
    deps: DepsMut,
    info: MessageInfo,
    pool_id: u64,
    asset_recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    let withdrawal_batch_size: u8 = 10;
    let token_id_response = query_pool_nft_token_ids(
        deps.as_ref(),
        pool_id,
        QueryOptions {
            descending: None,
            start_after: None,
            limit: Some(withdrawal_batch_size as u32),
        },
    )?;

    execute_withdraw_nfts(
        deps,
        info,
        pool_id,
        token_id_response.nft_token_ids,
        asset_recipient,
    )
}

/// Execute an UpdatePoolConfig message
/// Option paramaters that are not specified will not be updated
pub fn execute_update_pool_config(
    deps: DepsMut,
    info: MessageInfo,
    pool_id: u64,
    asset_recipient: Option<Addr>,
    delta: Option<Uint128>,
    spot_price: Option<Uint128>,
    finders_fee_bps: Option<u64>,
    swap_fee_bps: Option<u64>,
    reinvest_tokens: Option<bool>,
    reinvest_nfts: Option<bool>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut pool = pools().load(deps.storage, pool_id)?;
    // Only the owner of the pool can update the pool config
    only_owner(&info, &pool)?;

    let mut attr_keys: Vec<&str> = vec!["id"];
    if let Some(_asset_recipient) = asset_recipient {
        pool.asset_recipient = Some(_asset_recipient);
        attr_keys.push("asset_recipient");
    }
    if let Some(_spot_price) = spot_price {
        pool.spot_price = _spot_price;
        attr_keys.push("spot_price");
    }
    if let Some(_delta) = delta {
        pool.delta = _delta;
        attr_keys.push("delta");
    }
    if let Some(_swap_fee_bps) = swap_fee_bps {
        pool.swap_fee_percent = Decimal::percent(_swap_fee_bps);
        attr_keys.push("swap_fee_percent");
    }
    if let Some(_finders_fee_bps) = finders_fee_bps {
        pool.finders_fee_percent = Decimal::percent(_finders_fee_bps);
        attr_keys.push("finders_fee_percent");
    }
    if let Some(_reinvest_tokens) = reinvest_tokens {
        pool.reinvest_tokens = _reinvest_tokens;
        attr_keys.push("reinvest_tokens");
    }
    if let Some(_reinvest_nfts) = reinvest_nfts {
        pool.reinvest_nfts = _reinvest_nfts;
        attr_keys.push("reinvest_nfts");
    }

    let config = CONFIG.load(deps.storage)?;
    let marketplace_params = load_marketplace_params(deps.as_ref(), &config.marketplace_addr)?;

    let mut response = Response::new();
    let event = pool.create_event("update-pool-config", attr_keys)?;
    response = response.add_event(event);
    response = save_pool(deps.storage, &mut pool, &marketplace_params, response)?;

    Ok(response)
}

/// Execute a SetActivePool message
pub fn execute_set_active_pool(
    deps: DepsMut,
    info: MessageInfo,
    pool_id: u64,
    is_active: bool,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut pool = pools().load(deps.storage, pool_id)?;
    // Only the owner of the pool can update the pool config
    only_owner(&info, &pool)?;

    pool.set_active(is_active)?;

    let config = CONFIG.load(deps.storage)?;
    let marketplace_params = load_marketplace_params(deps.as_ref(), &config.marketplace_addr)?;

    let mut response = Response::new();
    let event = pool.create_event("set-active-pool", vec!["id", "is_active"])?;
    response = response.add_event(event);
    response = save_pool(deps.storage, &mut pool, &marketplace_params, response)?;

    Ok(response)
}

/// Execute a RemovePool message
pub fn execute_remove_pool(
    deps: DepsMut,
    info: MessageInfo,
    pool_id: u64,
    asset_recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut pool = pools().load(deps.storage, pool_id)?;
    // Only the owner of the pool can remove the pool
    only_owner(&info, &pool)?;

    // Pools that hold NFTs cannot be removed
    if pool.total_nfts > 0 {
        return Err(ContractError::UnableToRemovePool(format!(
            "pool {} still has NFTs",
            pool_id
        )));
    }

    let config = CONFIG.load(deps.storage)?;
    let mut response = Response::new();

    // If the pool has tokens, transfer them to the asset recipient
    if pool.total_tokens > Uint128::zero() {
        let recipient = asset_recipient.unwrap_or(info.sender);
        transfer_token(
            coin(pool.total_tokens.u128(), config.denom),
            recipient.as_ref(),
            &mut response,
        )?;
    }

    let marketplace_params = load_marketplace_params(deps.as_ref(), &config.marketplace_addr)?;

    let event = pool.create_event("remove-pool", vec!["id"])?;
    response = response.add_event(event);
    response = remove_pool(deps.storage, &mut pool, &marketplace_params, response)?;

    Ok(response)
}

/// Execute a DirectSwapNftsForTokens message
pub fn execute_direct_swap_nfts_for_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pool_id: u64,
    nfts_to_swap: Vec<NftSwap>,
    swap_params: SwapParams,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let pool = pools().load(deps.storage, pool_id)?;

    let swap_prep_result = prep_for_swap(
        deps.as_ref(),
        &Some(env.block),
        &info.sender,
        &pool.collection,
        &swap_params,
    )?;

    validate_nft_swaps_for_sell(deps.as_ref(), &info, &pool.collection, &nfts_to_swap)?;

    let mut response = Response::new();
    let mut pools_to_save: Vec<Pool>;
    let swaps: Vec<Swap>;

    {
        let mut processor = SwapProcessor::new(
            TransactionType::NftsForTokens,
            env.contract.address.clone(),
            pool.collection.clone(),
            info.sender,
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
        processor.direct_swap_nfts_for_tokens(pool, nfts_to_swap, swap_params)?;
        processor.finalize_transaction(&mut response)?;
        response = response.add_events(processor.get_transaction_events());
        swaps = processor.swaps;
        pools_to_save = processor.pools_to_save.into_values().collect();
    }

    update_nft_deposits(deps.storage, &env.contract.address, &swaps)?;
    response = save_pools(
        deps.storage,
        pools_to_save.iter_mut().collect(),
        &swap_prep_result.marketplace_params,
        response,
    )?;

    Ok(response)
}

/// Execute a SwapNftsForTokens message
pub fn execute_swap_nfts_for_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: Addr,
    nfts_to_swap: Vec<NftSwap>,
    swap_params: SwapParams,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let swap_prep_result = prep_for_swap(
        deps.as_ref(),
        &Some(env.block),
        &info.sender,
        &collection,
        &swap_params,
    )?;

    validate_nft_swaps_for_sell(deps.as_ref(), &info, &collection, &nfts_to_swap)?;

    let mut response = Response::new();
    let mut pools_to_save: Vec<Pool>;
    let swaps: Vec<Swap>;

    {
        let mut processor = SwapProcessor::new(
            TransactionType::NftsForTokens,
            env.contract.address.clone(),
            collection,
            info.sender,
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
        processor.swap_nfts_for_tokens(deps.as_ref().storage, nfts_to_swap, swap_params)?;
        processor.finalize_transaction(&mut response)?;
        response = response.add_events(processor.get_transaction_events());
        swaps = processor.swaps;
        pools_to_save = processor.pools_to_save.into_values().collect();
    }

    update_nft_deposits(deps.storage, &env.contract.address, &swaps)?;
    response = save_pools(
        deps.storage,
        pools_to_save.iter_mut().collect(),
        &swap_prep_result.marketplace_params,
        response,
    )?;

    Ok(response)
}

/// Execute a DirectSwapTokensForSpecificNfts message
pub fn execute_direct_swap_tokens_for_specific_nfts(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pool_id: u64,
    nfts_to_swap_for: Vec<NftSwap>,
    swap_params: SwapParams,
) -> Result<Response, ContractError> {
    let pool = pools().load(deps.storage, pool_id)?;
    execute_swap_tokens_for_specific_nfts(
        deps,
        env,
        info,
        pool.collection,
        vec![PoolNftSwap {
            pool_id,
            nft_swaps: nfts_to_swap_for,
        }],
        swap_params,
    )
}

/// Execute a SwapTokensForSpecificNfts message
pub fn execute_swap_tokens_for_specific_nfts(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: Addr,
    nfts_to_swap_for: Vec<PoolNftSwap>,
    swap_params: SwapParams,
) -> Result<Response, ContractError> {
    let swap_prep_result = prep_for_swap(
        deps.as_ref(),
        &Some(env.block),
        &info.sender,
        &collection,
        &swap_params,
    )?;

    let received_amount =
        validate_nft_swaps_for_buy(&info, &swap_prep_result.denom, &nfts_to_swap_for)?;

    let mut response = Response::new();
    let mut pools_to_save: Vec<Pool>;
    let swaps: Vec<Swap>;

    {
        let mut processor = SwapProcessor::new(
            TransactionType::TokensForNfts,
            env.contract.address.clone(),
            collection,
            info.sender,
            received_amount,
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
        processor.swap_tokens_for_specific_nfts(deps.storage, nfts_to_swap_for, swap_params)?;
        processor.finalize_transaction(&mut response)?;
        response = response.add_events(processor.get_transaction_events());
        swaps = processor.swaps;
        pools_to_save = processor.pools_to_save.into_values().collect();
    }

    update_nft_deposits(deps.storage, &env.contract.address, &swaps)?;
    response = save_pools(
        deps.storage,
        pools_to_save.iter_mut().collect(),
        &swap_prep_result.marketplace_params,
        response,
    )?;

    Ok(response)
}

/// Execute a SwapTokensForAnyNfts message
pub fn execute_swap_tokens_for_any_nfts(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: Addr,
    max_expected_token_input: Vec<Uint128>,
    swap_params: SwapParams,
) -> Result<Response, ContractError> {
    let swap_prep_result = prep_for_swap(
        deps.as_ref(),
        &Some(env.block),
        &info.sender,
        &collection,
        &swap_params,
    )?;

    if max_expected_token_input.is_empty() {
        return Err(ContractError::InvalidInput(
            "max expected token input must not be empty".to_string(),
        ));
    }

    // User must send enough tokens to cover the swap
    // Should be the sum of all the token amounts in max_expected_token_input
    let received_amount = must_pay(&info, NATIVE_DENOM)?;
    let expected_amount = max_expected_token_input
        .iter()
        .fold(Uint128::zero(), |acc, amount| acc + amount);
    if received_amount < expected_amount {
        return Err(ContractError::InsufficientFunds(format!(
            "expected {} but received {}",
            expected_amount, received_amount
        )));
    }

    let mut response = Response::new();
    let mut pools_to_save: Vec<Pool>;
    let swaps: Vec<Swap>;

    {
        let mut processor = SwapProcessor::new(
            TransactionType::TokensForNfts,
            env.contract.address.clone(),
            collection,
            info.sender,
            received_amount,
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
        processor.swap_tokens_for_any_nfts(deps.storage, max_expected_token_input, swap_params)?;
        processor.finalize_transaction(&mut response)?;
        response = response.add_events(processor.get_transaction_events());
        swaps = processor.swaps;
        pools_to_save = processor.pools_to_save.into_values().collect();
    }

    update_nft_deposits(deps.storage, &env.contract.address, &swaps)?;
    response = save_pools(
        deps.storage,
        pools_to_save.iter_mut().collect(),
        &swap_prep_result.marketplace_params,
        response,
    )?;

    Ok(response)
}
