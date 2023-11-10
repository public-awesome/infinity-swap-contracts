use crate::helpers::generate_salt;
use crate::msg::ExecuteMsg;
use crate::state::{INFINITY_GLOBAL, SENDER_COUNTER, UNRESTRICTED_MIGRATIONS};
use crate::ContractError;

use cosmwasm_std::{attr, ensure_eq, to_binary, DepsMut, Empty, Env, Event, MessageInfo, WasmMsg};
use infinity_global::load_global_config;
use infinity_pair::msg::InstantiateMsg as InfinityPairInstantiateMsg;
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
    match msg {
        ExecuteMsg::CreatePair {
            pair_immutable,
            pair_config,
        } => {
            let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;
            let global_config = load_global_config(&deps.querier, &infinity_global)?;

            let mut response = Response::new();

            response = response.add_message(WasmMsg::Instantiate {
                admin: Some(env.contract.address.into()),
                code_id: global_config.infinity_pair_code_id,
                label: "Infinity Pair".to_string(),
                msg: to_binary(&InfinityPairInstantiateMsg {
                    infinity_global: infinity_global.to_string(),
                    pair_immutable,
                    pair_config,
                })?,
                funds: info.funds,
            });

            // Event used by indexer to track pair creation
            response = response.add_event(
                Event::new("factory-create-pair".to_string()).add_attribute("sender", info.sender),
            );

            Ok(response)
        },
        ExecuteMsg::CreatePair2 {
            pair_immutable,
            pair_config,
        } => {
            let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;
            let global_config = load_global_config(&deps.querier, &infinity_global)?;

            let counter_key = (info.sender.clone(), global_config.infinity_pair_code_id);
            let counter =
                SENDER_COUNTER.may_load(deps.storage, counter_key.clone())?.unwrap_or_default();
            let salt = generate_salt(&info.sender, counter);
            SENDER_COUNTER.save(deps.storage, counter_key, &(counter + 1))?;

            let mut response = Response::new();

            response = response.add_message(WasmMsg::Instantiate2 {
                admin: Some(env.contract.address.into()),
                code_id: global_config.infinity_pair_code_id,
                label: "Infinity Pair".to_string(),
                msg: to_binary(&InfinityPairInstantiateMsg {
                    infinity_global: infinity_global.to_string(),
                    pair_immutable,
                    pair_config,
                })?,
                funds: info.funds,
                salt,
            });

            // Event used by indexer to track pair creation
            response = response.add_event(
                Event::new("factory-create-pair2".to_string()).add_attribute("sender", info.sender),
            );

            Ok(response)
        },
        ExecuteMsg::UnrestrictedMigratePair {
            pair_address,
            target_code_id,
        } => {
            let contract_info_response = deps.querier.query_wasm_contract_info(&pair_address)?;

            let valid_target_code_id =
                UNRESTRICTED_MIGRATIONS.load(deps.storage, contract_info_response.code_id)?;

            ensure_eq!(
                target_code_id,
                valid_target_code_id,
                ContractError::InvalidMigration("Invalid target code id".to_string())
            );

            let mut response = Response::new().add_message(WasmMsg::Migrate {
                contract_addr: pair_address.clone(),
                new_code_id: target_code_id,
                msg: to_binary(&Empty {})?,
            });

            // Event used by indexer to track pair migration
            response = response.add_event(
                Event::new("factory-migrate-pair".to_string()).add_attributes(vec![
                    attr("pair_address", pair_address),
                    attr("target_code_id", target_code_id.to_string()),
                ]),
            );

            Ok(response)
        },
    }
}
