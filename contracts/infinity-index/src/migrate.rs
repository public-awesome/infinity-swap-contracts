use crate::{
    error::ContractError,
    instantiate::{CONTRACT_NAME, CONTRACT_VERSION},
};

use cosmwasm_std::{ensure, DepsMut, Empty, Env, Event, StdError};
use semver::Version;
use sg_std::Response;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
#[allow(clippy::cmp_owned)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    let prev_contract_version = cw2::get_contract_version(deps.storage)?;

    let valid_contract_names = [CONTRACT_NAME.to_string()];
    ensure!(
        valid_contract_names.contains(&prev_contract_version.contract),
        StdError::generic_err("Invalid contract name for migration")
    );

    ensure!(
        Version::parse(&prev_contract_version.version).unwrap()
            < Version::parse(CONTRACT_VERSION).unwrap(),
        StdError::generic_err("Must upgrade contract version")
    );

    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let response = Response::new().add_event(
        Event::new("migrate")
            .add_attribute("from_name", prev_contract_version.contract)
            .add_attribute("from_version", prev_contract_version.version)
            .add_attribute("to_name", CONTRACT_NAME)
            .add_attribute("to_version", CONTRACT_VERSION),
    );

    Ok(response)
}
