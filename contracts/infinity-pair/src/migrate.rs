use crate::{
    constants::{CONTRACT_NAME, CONTRACT_VERSION},
    error::ContractError,
    events::{PairInternalEvent, UpdatePairEvent},
    helpers::{load_pair, load_payout_context},
    state::INFINITY_GLOBAL,
};

use cosmwasm_std::{ensure, DepsMut, Empty, Env, Event, StdError};
use semver::Version;
use sg_std::Response;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
#[allow(clippy::cmp_owned)]
pub fn migrate(deps: DepsMut, env: Env, _msg: Empty) -> Result<Response, ContractError> {
    let prev_contract_version = cw2::get_contract_version(deps.storage)?;

    let valid_contract_names = vec![CONTRACT_NAME.to_string()];
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

    let mut pair = load_pair(&env.contract.address, deps.storage, &deps.querier)?;

    let infinity_global = INFINITY_GLOBAL.load(deps.storage)?;

    let payout_context = load_payout_context(
        deps.as_ref(),
        &infinity_global,
        &pair.immutable.collection,
        &pair.immutable.denom,
    )?;

    let mut response =
        pair.save_and_update_indices(deps.storage, &payout_context, Response::new())?;

    response = response
        .add_event(
            Event::new("migrate")
                .add_attribute("from_name", prev_contract_version.contract)
                .add_attribute("from_version", prev_contract_version.version)
                .add_attribute("to_name", CONTRACT_NAME)
                .add_attribute("to_version", CONTRACT_VERSION),
        )
        .add_event(
            UpdatePairEvent {
                ty: "migrate-pair",
                pair: &pair,
            }
            .into(),
        )
        .add_event(
            PairInternalEvent {
                pair: &pair,
            }
            .into(),
        );

    Ok(response)
}
