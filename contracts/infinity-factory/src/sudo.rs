use crate::msg::SudoMsg;
use crate::state::UNRESTRICTED_MIGRATIONS;
use crate::ContractError;

use cosmwasm_std::{attr, ensure, ensure_eq, DepsMut, Env, Event};
use infinity_shared::InfinityError;
use sg_std::Response;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::AddUnrestrictedMigration {
            starting_code_id,
            target_code_id,
        } => sudo_add_unrestricted_migration(deps, starting_code_id, target_code_id),
        SudoMsg::RemoveUnrestrictedMigration {
            starting_code_id,
        } => sudo_remove_unrestricted_migration(deps, starting_code_id),
    }
}

pub fn sudo_add_unrestricted_migration(
    deps: DepsMut,
    starting_code_id: u64,
    target_code_id: u64,
) -> Result<Response, ContractError> {
    let existing_migration = UNRESTRICTED_MIGRATIONS.may_load(deps.storage, starting_code_id)?;
    ensure_eq!(
        existing_migration,
        None,
        InfinityError::InvalidInput("Migration already exists".to_string())
    );

    UNRESTRICTED_MIGRATIONS.save(deps.storage, starting_code_id, &target_code_id)?;

    let response =
        Response::new().add_event(Event::new("add-unrestricted-migration").add_attributes(vec![
            attr("starting_code_id", starting_code_id.to_string()),
            attr("target_code_id", target_code_id.to_string()),
        ]));

    Ok(response)
}

pub fn sudo_remove_unrestricted_migration(
    deps: DepsMut,
    starting_code_id: u64,
) -> Result<Response, ContractError> {
    let existing_migration = UNRESTRICTED_MIGRATIONS.may_load(deps.storage, starting_code_id)?;
    ensure!(
        existing_migration.is_some(),
        InfinityError::InvalidInput("Migration does not exist".to_string())
    );

    let response = Response::new().add_event(
        Event::new("remove-unrestricted-migration")
            .add_attributes(vec![attr("starting_code_id", starting_code_id.to_string())]),
    );

    Ok(response)
}
