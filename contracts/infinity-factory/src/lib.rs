use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, DepsMut, Empty, Env, MessageInfo, StdResult};
use cosmwasm_std::{StdError, WasmMsg};
use cw2::set_contract_version;
use cw_storage_plus::Item;
use infinity_global::load_global_config;
use infinity_pair::msg::InstantiateMsg as InfinityPairInstantiateMsg;
use infinity_pair::state::{PairConfig, PairImmutable};
use sg_std::Response;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const INFINITY_GLOBAL: Item<Addr> = Item::new("g");

#[cw_serde]
pub struct InstantiateMsg {
    /// The address of the infinity global contract
    pub infinity_global: String,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, StdError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    INFINITY_GLOBAL.save(deps.storage, &deps.api.addr_validate(&msg.infinity_global)?)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION))
}

#[cw_serde]
pub enum ExecuteMsg {
    CreatePair {
        pair_immutable: PairImmutable,
        pair_config: PairConfig,
    },
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, StdError> {
    match msg {
        ExecuteMsg::CreatePair {
            pair_immutable,
            pair_config,
        } => {
            let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;
            let global_config = load_global_config(&deps.querier, &infinity_global)?;

            let response = Response::new().add_message(WasmMsg::Instantiate {
                admin: Some(env.contract.address.into()),
                code_id: global_config.infinity_pair_code_id,
                label: "infinity-pair".to_string(),
                msg: to_binary(&InfinityPairInstantiateMsg {
                    infinity_global,
                    pair_immutable,
                    pair_config,
                })?,
                funds: info.funds,
            });

            Ok(response)
        },
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: Empty) -> StdResult<Binary> {
    unimplemented!("not implemented")
}
