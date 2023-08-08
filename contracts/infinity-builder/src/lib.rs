use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    instantiate2_address, to_binary, Addr, Binary, Coin, Decimal, Deps, DepsMut, Empty, Env,
    Instantiate2AddressError, MessageInfo, StdError, StdResult, WasmMsg,
};
use cw2::set_contract_version;
use infinity_factory::InstantiateMsg as InfinityFactoryInstantiateMsg;
use infinity_global::{GlobalConfig, InstantiateMsg as InfinityGlobalInstantiateMsg};
use infinity_index::msg::InstantiateMsg as InfinityIndexInstantiateMsg;
use infinity_router::msg::InstantiateMsg as InfinityRouterInstantiateMsg;
use sg_std::Response;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Instantiate2AddressError(#[from] Instantiate2AddressError),
}

#[cw_serde]
pub struct CodeIds {
    pub infinity_global: u64,
    pub infinity_factory: u64,
    pub infinity_index: u64,
    pub infinity_pair: u64,
    pub infinity_router: u64,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub fair_burn: String,
    pub royalty_registry: String,
    pub marketplace: String,
    pub pair_creation_fee: Coin,
    pub fair_burn_fee_percent: Decimal,
    pub max_royalty_fee_percent: Decimal,
    pub max_swap_fee_percent: Decimal,
    pub code_ids: CodeIds,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let (infinity_global, infinity_global_salt) = generate_instantiate_2_addr(
        deps.as_ref(),
        &env,
        "infinity_global",
        msg.code_ids.infinity_global,
    )?;

    let (infinity_factory, infinity_factory_salt) = generate_instantiate_2_addr(
        deps.as_ref(),
        &env,
        "infinity_factory",
        msg.code_ids.infinity_factory,
    )?;

    let (infinity_index, infinity_index_salt) = generate_instantiate_2_addr(
        deps.as_ref(),
        &env,
        "infinity_index",
        msg.code_ids.infinity_index,
    )?;

    let (infinity_router, infinity_router_salt) = generate_instantiate_2_addr(
        deps.as_ref(),
        &env,
        "infinity_router",
        msg.code_ids.infinity_router,
    )?;

    let mut response = Response::new();

    response = response.add_message(WasmMsg::Instantiate2 {
        admin: Some(env.contract.address.to_string()),
        code_id: msg.code_ids.infinity_global,
        label: "Infinity Global".to_string(),
        msg: to_binary(&InfinityGlobalInstantiateMsg {
            global_config: GlobalConfig {
                fair_burn: msg.fair_burn,
                royalty_registry: msg.royalty_registry,
                marketplace: msg.marketplace,
                infinity_factory: infinity_factory.to_string(),
                infinity_index: infinity_index.to_string(),
                infinity_router: infinity_router.to_string(),
                infinity_pair_code_id: msg.code_ids.infinity_pair,
                pair_creation_fee: msg.pair_creation_fee,
                fair_burn_fee_percent: msg.fair_burn_fee_percent,
                max_royalty_fee_percent: msg.max_royalty_fee_percent,
                max_swap_fee_percent: msg.max_swap_fee_percent,
            },
        })?,
        funds: vec![],
        salt: infinity_global_salt,
    });

    response = response.add_message(WasmMsg::Instantiate2 {
        admin: Some(env.contract.address.to_string()),
        code_id: msg.code_ids.infinity_factory,
        label: "Infinity Factory".to_string(),
        msg: to_binary(&InfinityFactoryInstantiateMsg {
            infinity_global: infinity_global.to_string(),
        })?,
        funds: vec![],
        salt: infinity_factory_salt,
    });

    response = response.add_message(WasmMsg::Instantiate2 {
        admin: Some(env.contract.address.to_string()),
        code_id: msg.code_ids.infinity_factory,
        label: "Infinity Index".to_string(),
        msg: to_binary(&InfinityIndexInstantiateMsg {
            infinity_global: infinity_global.to_string(),
        })?,
        funds: vec![],
        salt: infinity_index_salt,
    });

    response = response.add_message(WasmMsg::Instantiate2 {
        admin: Some(env.contract.address.to_string()),
        code_id: msg.code_ids.infinity_factory,
        label: "Infinity Router".to_string(),
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: Empty) -> StdResult<Binary> {
    unimplemented!()
}
