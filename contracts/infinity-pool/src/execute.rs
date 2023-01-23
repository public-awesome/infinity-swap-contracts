use crate::error::ContractError;
use crate::helpers::{
    get_next_pool_counter, get_pool_attributes, only_owner, remove_pool, save_pool, transfer_nft,
    transfer_token,
};
use crate::msg::{ExecuteMsg, PoolNfts};
use crate::state::{pools, BondingCurve, Pool, PoolType, CONFIG};
use crate::swap_processor::SwapProcessor;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{coin, Addr, DepsMut, Env, Event, MessageInfo, Uint128};
use cw_utils::{maybe_addr, must_pay, nonpayable};
use sg_std::Response;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::CreatePool {
            collection,
            asset_recipient,
            pool_type,
            bonding_curve,
            delta,
            spot_price,
            fee_bps,
        } => execute_create_pool(
            deps,
            info,
            api.addr_validate(&collection)?,
            maybe_addr(api, asset_recipient)?,
            pool_type,
            bonding_curve,
            spot_price,
            delta,
            fee_bps,
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
            fee_bps,
        } => execute_update_pool_config(
            deps,
            info,
            pool_id,
            maybe_addr(api, asset_recipient)?,
            delta,
            spot_price,
            fee_bps,
        ),
        ExecuteMsg::SetActivePool { pool_id, is_active } => {
            execute_set_active_pool(deps, info, pool_id, is_active)
        }
        ExecuteMsg::RemovePool {
            pool_id,
            asset_recipient,
        } => execute_remove_pool(deps, info, pool_id, maybe_addr(api, asset_recipient)?),
        ExecuteMsg::SwapNftForTokens {
            collection,
            nft_token_ids,
            min_expected_token_output,
            token_recipient,
        } => execute_swap_nft_for_tokens(
            deps,
            info,
            api.addr_validate(&collection)?,
            nft_token_ids,
            min_expected_token_output,
            maybe_addr(api, token_recipient)?,
        ),
        ExecuteMsg::SwapTokenForSpecificNfts {
            collection,
            pool_nfts,
            max_expected_token_input,
            nft_recipient,
        } => execute_swap_token_for_specific_nfts(
            deps,
            info,
            api.addr_validate(&collection)?,
            pool_nfts,
            max_expected_token_input,
            maybe_addr(api, nft_recipient)?,
        ),
        _ => Ok(Response::default()),
    }
}

pub fn execute_create_pool(
    deps: DepsMut,
    info: MessageInfo,
    collection: Addr,
    asset_recipient: Option<Addr>,
    pool_type: PoolType,
    bonding_curve: BondingCurve,
    spot_price: Uint128,
    delta: Uint128,
    fee_bps: u16,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let response = Response::new();

    let pool_counter = get_next_pool_counter(deps.storage)?;
    let pool = Pool::new(
        pool_counter,
        collection,
        info.sender,
        asset_recipient,
        pool_type,
        bonding_curve,
        spot_price,
        delta,
        fee_bps,
    );
    save_pool(deps.storage, &pool)?;

    let mut event = Event::new("create_token_pool");
    let pool_attributes = get_pool_attributes(&pool);
    for attribute in pool_attributes {
        event = event.add_attribute(attribute.key, attribute.value);
    }

    Ok(response.add_event(event))
}

pub fn execute_deposit_tokens(
    deps: DepsMut,
    info: MessageInfo,
    pool_id: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let received_amount = must_pay(&info, &config.denom)?;

    let mut pool = pools().load(deps.storage, pool_id)?;
    only_owner(&info, &pool)?;

    let response = Response::new();

    pool.deposit_tokens(received_amount)?;
    save_pool(deps.storage, &pool)?;

    let event = Event::new("deposit_tokens")
        .add_attribute("pool_id", pool_id.to_string())
        .add_attribute("tokens_received", received_amount.to_string())
        .add_attribute("total_tokens", pool.total_tokens.to_string());

    Ok(response.add_event(event))
}

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
    only_owner(&info, &pool)?;
    if pool.collection != collection {
        return Err(ContractError::InvalidPool(format!(
            "invalid collection ({}) for pool ({})",
            collection, pool.id
        )));
    }

    let mut response = Response::new();

    for nft_token_id in &nft_token_ids {
        transfer_nft(
            &nft_token_id,
            &env.contract.address,
            &collection,
            &mut response,
        )?;
    }

    pool.deposit_nfts(&nft_token_ids)?;
    save_pool(deps.storage, &pool)?;

    let all_nft_token_ids = pool
        .nft_token_ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(",");
    let event = Event::new("deposit_nfts")
        .add_attribute("nft_token_ids", pool_id.to_string())
        .add_attribute("nfts_received", nft_token_ids.join(","))
        .add_attribute("total_tokens", all_nft_token_ids);

    Ok(response.add_event(event))
}

pub fn execute_withdraw_tokens(
    deps: DepsMut,
    info: MessageInfo,
    pool_id: u64,
    amount: Uint128,
    asset_recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut pool = pools().load(deps.storage, pool_id)?;
    only_owner(&info, &pool)?;

    let mut response = Response::new();

    let config = CONFIG.load(deps.storage)?;
    let recipient = asset_recipient.unwrap_or(info.sender);
    transfer_token(
        coin(amount.u128(), &config.denom),
        recipient.to_string(),
        "withdrawal_by_owner",
        &mut response,
    )?;

    pool.withdraw_tokens(amount)?;
    save_pool(deps.storage, &pool)?;

    let event = Event::new("withdraw_tokens")
        .add_attribute("pool_id", pool_id.to_string())
        .add_attribute("tokens_withdrawn", amount.to_string())
        .add_attribute("total_tokens", pool.total_tokens.to_string());

    Ok(response.add_event(event))
}

pub fn execute_withdraw_all_tokens(
    deps: DepsMut,
    info: MessageInfo,
    pool_id: u64,
    asset_recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let pool = pools().load(deps.storage, pool_id)?;
    execute_withdraw_tokens(deps, info, pool_id, pool.total_tokens, asset_recipient)
}

pub fn execute_withdraw_nfts(
    deps: DepsMut,
    info: MessageInfo,
    pool_id: u64,
    nft_token_ids: Vec<String>,
    asset_recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut pool = pools().load(deps.storage, pool_id)?;
    only_owner(&info, &pool)?;

    let mut response = Response::new();

    let recipient = asset_recipient.unwrap_or(info.sender);
    for nft_token_id in &nft_token_ids {
        transfer_nft(&nft_token_id, &recipient, &pool.collection, &mut response)?;
    }

    pool.withdraw_nfts(&nft_token_ids)?;
    save_pool(deps.storage, &pool)?;

    let all_nft_token_ids = pool
        .nft_token_ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(",");
    let event = Event::new("withdraw_nfts")
        .add_attribute("nft_token_ids", pool_id.to_string())
        .add_attribute("nfts_withdrawn", nft_token_ids.join(","))
        .add_attribute("nft_token_ids", all_nft_token_ids);

    Ok(response.add_event(event))
}

pub fn execute_withdraw_all_nfts(
    deps: DepsMut,
    info: MessageInfo,
    pool_id: u64,
    asset_recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let pool = pools().load(deps.storage, pool_id)?;

    let withdrawal_batch_size: u8 = 10;
    let nft_token_ids = pool
        .nft_token_ids
        .into_iter()
        .take(withdrawal_batch_size as usize)
        .collect();

    execute_withdraw_nfts(deps, info, pool_id, nft_token_ids, asset_recipient)
}

pub fn execute_update_pool_config(
    deps: DepsMut,
    info: MessageInfo,
    pool_id: u64,
    asset_recipient: Option<Addr>,
    delta: Option<Uint128>,
    spot_price: Option<Uint128>,
    fee_bps: Option<u16>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut pool = pools().load(deps.storage, pool_id)?;
    only_owner(&info, &pool)?;

    let response = Response::new();

    if let Some(_asset_recipient) = asset_recipient {
        pool.asset_recipient = Some(_asset_recipient);
    }
    if let Some(_delta) = delta {
        pool.delta = _delta;
    }
    if let Some(_spot_price) = spot_price {
        pool.spot_price = _spot_price;
    }
    if let Some(_fee_bps) = fee_bps {
        pool.fee_bps = _fee_bps;
    }
    save_pool(deps.storage, &pool)?;

    let mut event = Event::new("update_pool_config");
    let pool_attributes = get_pool_attributes(&pool);
    for attribute in pool_attributes {
        event = event.add_attribute(attribute.key, attribute.value);
    }

    Ok(response.add_event(event))
}

pub fn execute_set_active_pool(
    deps: DepsMut,
    info: MessageInfo,
    pool_id: u64,
    is_active: bool,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut pool = pools().load(deps.storage, pool_id)?;
    only_owner(&info, &pool)?;

    let response = Response::new();

    pool.set_active(is_active)?;
    save_pool(deps.storage, &pool)?;

    let event = Event::new("toggle_pool")
        .add_attribute("pool_id", pool_id.to_string())
        .add_attribute("is_active", pool.is_active.to_string());

    Ok(response.add_event(event))
}

pub fn execute_remove_pool(
    deps: DepsMut,
    info: MessageInfo,
    pool_id: u64,
    asset_recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let mut pool = pools().load(deps.storage, pool_id)?;
    only_owner(&info, &pool)?;

    if !pool.nft_token_ids.is_empty() {
        let all_nft_token_ids = pool
            .nft_token_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(",");
        return Err(ContractError::UnableToRemovePool(format!(
            "pool {} still has NFTs: {}",
            pool_id, all_nft_token_ids
        )));
    }

    let mut response = Response::new();

    if pool.total_tokens > Uint128::zero() {
        let config = CONFIG.load(deps.storage)?;
        let recipient = asset_recipient.unwrap_or(info.sender);
        transfer_token(
            coin(pool.total_tokens.u128(), &config.denom),
            recipient.to_string(),
            "pool_removed_by_owner",
            &mut response,
        )?;
    }

    remove_pool(deps.storage, &mut pool)?;

    let event = Event::new("remove_pool").add_attribute("pool_id", pool_id.to_string());

    Ok(response.add_event(event))
}

pub fn execute_swap_token_for_specific_nfts(
    deps: DepsMut,
    info: MessageInfo,
    collection: Addr,
    pool_nfts: Vec<PoolNfts>,
    max_expected_token_input: Uint128,
    nft_recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let received_amount = must_pay(&info, &config.denom)?;
    if received_amount < max_expected_token_input {
        return Err(ContractError::InsufficientFunds(format!(
            "expected {}, received {}",
            received_amount, config.denom
        )));
    }

    let mut response = Response::new();
    let seller_recipient = nft_recipient.unwrap_or(info.sender);

    let mut processor = SwapProcessor::new(
        config.marketplace_addr.clone(),
        collection.clone(),
        seller_recipient,
    );
    processor.swap_token_for_specific_nfts(deps, pool_nfts, max_expected_token_input)?;

    Ok(response)
}

pub fn execute_swap_nft_for_tokens(
    deps: DepsMut,
    info: MessageInfo,
    collection: Addr,
    nft_token_ids: Vec<String>,
    min_expected_token_output: Uint128,
    token_recipient: Option<Addr>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let mut response = Response::new();
    let token_recipient = token_recipient.unwrap_or(info.sender);

    let mut processor = SwapProcessor::new(
        config.marketplace_addr.clone(),
        collection.clone(),
        token_recipient,
    );
    processor.swap_nft_for_tokens(
        deps.storage,
        collection,
        nft_token_ids,
        min_expected_token_output,
    )?;

    Ok(response)
}
