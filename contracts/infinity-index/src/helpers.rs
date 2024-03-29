use crate::state::INFINITY_GLOBAL;
use crate::ContractError;

use cosmwasm_std::{ensure_eq, Addr, Deps};
use infinity_global::load_global_config;
use infinity_shared::InfinityError;

/// Only infinity pairs created by the infinity factory can execute this function
/// and update the index.
pub fn only_infinity_pair(deps: Deps, contract: &Addr) -> Result<(), ContractError> {
    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;
    let global_config = load_global_config(&deps.querier, &infinity_global)?;
    let contract_info = deps.querier.query_wasm_contract_info(contract)?;

    ensure_eq!(
        global_config.infinity_factory,
        contract_info.creator,
        InfinityError::Unauthorized(
            "only an infinity pair contract can execute this function".to_string()
        )
    );

    Ok(())
}
