use crate::{error::ContractError};
use crate::msg::ExecuteMsg;
use crate::state::{PoolType, BondingCurve, POOL_COUNTER, Pool};
use crate::helpers::save_pool;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Uint128, DepsMut, Env, MessageInfo, Response, Addr};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::CreatePool {
            collection,
            pool_type,
            bonding_curve,
            delta,
            fee,
            asset_recipient,
        } => execute_create_pool(
            deps,
            info,
            api.addr_validate(&collection)?,
            pool_type,
            bonding_curve,
            delta,
            fee,
            api.addr_validate(&asset_recipient)?,
        ),
        _ => Ok(Response::default()),
    }
}

pub fn execute_create_pool(
    deps: DepsMut,
    _info: MessageInfo,
    collection: Addr,
    pool_type: PoolType,
    bonding_curve: BondingCurve,
    delta: Uint128,
    fee: Uint128,
    asset_recipient: Addr,
) -> Result<Response, ContractError> {
    let pool_counter = POOL_COUNTER.load(deps.storage)?;

    save_pool(deps.storage, &Pool {
        id: pool_counter,
        collection,
        pool_type,
        bonding_curve,
        delta,
        fee,
        asset_recipient,
        buy_price_quote: Uint128::zero(),
        sell_price_quote: Uint128::zero(),
    })?;

    POOL_COUNTER.save(deps.storage, &(pool_counter + 1))?;

    Ok(Response::new())
}