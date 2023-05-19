use crate::msg::ExecuteMsg;
use crate::{error::ContractError, state::INFINITY_GLOBAL};

use cosmwasm_std::{to_binary, Addr, DepsMut, Env, MessageInfo, Uint128, WasmMsg};
use cw_utils::maybe_addr;
use infinity_pool::msg::{InstantiateMsg as InfinityPoolInstantiateMsg, PoolInfo};
use infinity_pool::state::{BondingCurve, PoolType};
use infinity_shared::global::load_global_config;
use sg_std::Response;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

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
        } => execute_create_token_pool(
            deps,
            env,
            info,
            api.addr_validate(&collection)?,
            maybe_addr(api, asset_recipient)?,
            bonding_curve,
            spot_price,
            delta,
            finders_fee_bps,
        ),
        ExecuteMsg::CreateNftPool {
            collection,
            asset_recipient,
            bonding_curve,
            spot_price,
            delta,
            finders_fee_bps,
        } => execute_create_nft_pool(
            deps,
            env,
            info,
            api.addr_validate(&collection)?,
            maybe_addr(api, asset_recipient)?,
            bonding_curve,
            spot_price,
            delta,
            finders_fee_bps,
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
        } => execute_create_trade_pool(
            deps,
            env,
            info,
            api.addr_validate(&collection)?,
            maybe_addr(api, asset_recipient)?,
            bonding_curve,
            spot_price,
            delta,
            finders_fee_bps,
            swap_fee_bps,
            reinvest_tokens,
            reinvest_nfts,
        ),
    }
}

pub fn execute_create_token_pool(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: Addr,
    asset_recipient: Option<Addr>,
    bonding_curve: BondingCurve,
    spot_price: Uint128,
    delta: Uint128,
    finders_fee_bps: u64,
) -> Result<Response, ContractError> {
    let mut response = Response::new();

    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;

    let global_config = load_global_config(&deps.querier, &infinity_global)?;

    response = response.add_message(WasmMsg::Instantiate {
        admin: Some(env.contract.address.into()),
        code_id: global_config.infinity_pool_code_id,
        label: "InfinityTokenPool".to_string(),
        msg: to_binary(&InfinityPoolInstantiateMsg {
            infinity_global: infinity_global.to_string(),
            pool_info: PoolInfo {
                collection: collection.to_string(),
                owner: info.sender.to_string(),
                asset_recipient: asset_recipient.map(|ar| ar.to_string()),
                pool_type: PoolType::Token,
                bonding_curve,
                spot_price,
                delta,
                finders_fee_bps,
                swap_fee_bps: 0,
                reinvest_tokens: false,
                reinvest_nfts: false,
            },
        })?,
        funds: vec![],
    });

    Ok(response)
}

pub fn execute_create_nft_pool(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: Addr,
    asset_recipient: Option<Addr>,
    bonding_curve: BondingCurve,
    spot_price: Uint128,
    delta: Uint128,
    finders_fee_bps: u64,
) -> Result<Response, ContractError> {
    let mut response = Response::new();

    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;

    let global_config = load_global_config(&deps.querier, &infinity_global)?;

    response = response.add_message(WasmMsg::Instantiate {
        admin: Some(env.contract.address.into()),
        code_id: global_config.infinity_pool_code_id,
        label: "InfinityNftPool".to_string(),
        msg: to_binary(&InfinityPoolInstantiateMsg {
            infinity_global: infinity_global.to_string(),
            pool_info: PoolInfo {
                collection: collection.to_string(),
                owner: info.sender.to_string(),
                asset_recipient: asset_recipient.map(|ar| ar.to_string()),
                pool_type: PoolType::Nft,
                bonding_curve,
                spot_price,
                delta,
                finders_fee_bps,
                swap_fee_bps: 0,
                reinvest_tokens: false,
                reinvest_nfts: false,
            },
        })?,
        funds: vec![],
    });

    Ok(response)
}

pub fn execute_create_trade_pool(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: Addr,
    asset_recipient: Option<Addr>,
    bonding_curve: BondingCurve,
    spot_price: Uint128,
    delta: Uint128,
    finders_fee_bps: u64,
    swap_fee_bps: u64,
    reinvest_tokens: bool,
    reinvest_nfts: bool,
) -> Result<Response, ContractError> {
    let mut response = Response::new();

    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;

    let global_config = load_global_config(&deps.querier, &infinity_global)?;

    response = response.add_message(WasmMsg::Instantiate {
        admin: Some(env.contract.address.into()),
        code_id: global_config.infinity_pool_code_id,
        label: "InfinityTradePool".to_string(),
        msg: to_binary(&InfinityPoolInstantiateMsg {
            infinity_global: infinity_global.to_string(),
            pool_info: PoolInfo {
                collection: collection.to_string(),
                owner: info.sender.to_string(),
                asset_recipient: asset_recipient.map(|ar| ar.to_string()),
                pool_type: PoolType::Trade,
                bonding_curve,
                spot_price,
                delta,
                finders_fee_bps,
                swap_fee_bps,
                reinvest_tokens,
                reinvest_nfts,
            },
        })?,
        funds: vec![],
    });

    Ok(response)
}
