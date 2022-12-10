use crate::{error::ContractError, state::pools};
use crate::msg::ExecuteMsg;
use crate::state::{PoolType, POOL_COUNTER, Pool};

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
            delta,
            fee,
            asset_recipient,
        } => execute_create_pool(
            deps,
            info,
            api.addr_validate(&collection)?,
            pool_type,
            delta,
            fee,
            api.addr_validate(&asset_recipient)?,
        ),
    }
}

pub fn execute_create_pool(
    deps: DepsMut,
    _info: MessageInfo,
    collection: Addr,
    pool_type: PoolType,
    delta: Uint128,
    fee: Uint128,
    asset_recipient: Addr,
) -> Result<Response, ContractError> {
    let pool_counter = POOL_COUNTER.load(deps.storage)?;

    pools().save(deps.storage, pool_counter, &Pool {
        key: pool_counter,
        collection,
        pool_type,
        delta,
        fee,
        asset_recipient,
    })?;

    POOL_COUNTER.save(deps.storage, &(pool_counter + 1))?;

    Ok(Response::new())
}