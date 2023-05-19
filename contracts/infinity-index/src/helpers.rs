use crate::state::INFINITY_GLOBAL;
use crate::ContractError;

use cosmwasm_std::{Addr, Deps};
use infinity_global::msg::{GlobalConfigResponse, QueryMsg as InfinityGlobalQueryMsg};

pub fn validate_infinity_pool(deps: Deps, contract: &Addr) -> Result<(), ContractError> {
    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;

    let infinity_global_config = deps
        .querier
        .query_wasm_smart::<GlobalConfigResponse>(
            infinity_global,
            &InfinityGlobalQueryMsg::GlobalConfig {},
        )?
        .config;

    let contract_info = deps.querier.query_wasm_contract_info(contract)?;

    if infinity_global_config.infinity_factory.to_string() != contract_info.creator {
        return Err(ContractError::InvalidInfinityPool(contract.clone()));
    }

    Ok(())
}
