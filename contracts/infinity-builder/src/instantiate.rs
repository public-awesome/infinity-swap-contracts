use crate::error::ContractError;
use crate::msg::InstantiateMsg;

use cosmwasm_std::{
    instantiate2_address, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, WasmMsg,
};
use cw2::set_contract_version;
use infinity_factory::msg::InstantiateMsg as InfinityFactoryInstantiateMsg;
use infinity_global::msg::InstantiateMsg as InfinityGlobalInstantiateMsg;
use infinity_index::msg::InstantiateMsg as InfinityIndexInstantiateMsg;
use infinity_router::msg::InstantiateMsg as InfinityRouterInstantiateMsg;
use sg_std::Response;
use sha2::{Digest, Sha256};

pub const CONTRACT_NAME: &str = "crates.io:infinity-builder";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let marketplace = deps.api.addr_validate(&msg.marketplace)?;

    let (infinity_global, infinity_global_salt) = generate_instantiate_2_addr(
        deps.as_ref(),
        &env,
        "infinity_global",
        msg.infinity_global_code_id,
    )?;

    let (infinity_factory, infinity_factory_salt) = generate_instantiate_2_addr(
        deps.as_ref(),
        &env,
        "infinity_factory",
        msg.infinity_factory_code_id,
    )?;

    let (infinity_index, infinity_index_salt) = generate_instantiate_2_addr(
        deps.as_ref(),
        &env,
        "infinity_index",
        msg.infinity_index_code_id,
    )?;

    let (_infinity_router, infinity_router_salt) = generate_instantiate_2_addr(
        deps.as_ref(),
        &env,
        "infinity_router",
        msg.infinity_router_code_id,
    )?;

    let mut response = Response::new();

    // Instantiate InfinityGlobal
    response = response.add_message(WasmMsg::Instantiate2 {
        admin: Some(env.contract.address.to_string()),
        code_id: msg.infinity_global_code_id,
        label: "InfinityGlobal".to_string(),
        msg: to_binary(&InfinityGlobalInstantiateMsg {
            infinity_index: infinity_index.to_string(),
            infinity_factory: infinity_factory.to_string(),
            marketplace: marketplace.to_string(),
            infinity_pool_code_id: msg.infinity_pool_code_id,
            min_price: msg.min_price,
            pool_creation_fee: msg.pool_creation_fee,
            trading_fee_bps: msg.trading_fee_bps,
        })?,
        funds: vec![],
        salt: infinity_global_salt,
    });

    // Instantiate InfinityFactory
    response = response.add_message(WasmMsg::Instantiate2 {
        admin: Some(env.contract.address.to_string()),
        code_id: msg.infinity_factory_code_id,
        label: "InfinityFactory".to_string(),
        msg: to_binary(&InfinityFactoryInstantiateMsg {
            infinity_global: infinity_global.to_string(),
        })?,
        funds: vec![],
        salt: infinity_factory_salt,
    });

    // Instantiate InfinityIndex
    response = response.add_message(WasmMsg::Instantiate2 {
        admin: Some(env.contract.address.to_string()),
        code_id: msg.infinity_factory_code_id,
        label: "InfinityIndex".to_string(),
        msg: to_binary(&InfinityIndexInstantiateMsg {
            infinity_global: infinity_global.to_string(),
        })?,
        funds: vec![],
        salt: infinity_index_salt,
    });

    // Instantiate InfinityRouter
    response = response.add_message(WasmMsg::Instantiate2 {
        admin: Some(env.contract.address.to_string()),
        code_id: msg.infinity_factory_code_id,
        label: "InfinityRouter".to_string(),
        msg: to_binary(&InfinityRouterInstantiateMsg {
            infinity_global: infinity_global.to_string(),
        })?,
        funds: vec![],
        salt: infinity_router_salt,
    });

    Ok(response
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION))
}

pub fn generate_salt(label: &str) -> Binary {
    let mut hasher = Sha256::new();
    hasher.update(label.as_bytes());
    hasher.finalize().to_vec().into()
}

pub fn generate_instantiate_2_addr(
    deps: Deps,
    env: &Env,
    label: &str,
    code_id: u64,
) -> Result<(Addr, Binary), ContractError> {
    let code_res = deps.querier.query_wasm_code_info(code_id)?;

    let salt = generate_salt(label);

    // predict the contract address
    let addr_raw = instantiate2_address(
        &code_res.checksum,
        &deps.api.addr_canonicalize(env.contract.address.as_str())?,
        &salt,
    )?;

    let addr = deps.api.addr_humanize(&addr_raw)?;

    Ok((addr, salt))
}
