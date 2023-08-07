use crate::msg::InstantiateMsg;
use crate::pair::Pair;
use crate::state::INFINITY_GLOBAL;
use crate::{
    constants::{CONTRACT_NAME, CONTRACT_VERSION},
    error::ContractError,
};

use cosmwasm_std::{ensure_eq, DepsMut, Env, MessageInfo};
use cw2::set_contract_version;
use cw_utils::must_pay;
use infinity_global::load_global_config;
use infinity_shared::InfinityError;
use sg_std::Response;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use stargaze_fair_burn::append_fair_burn_msg;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let mut response = Response::new();
    let global_config = load_global_config(&deps.querier, &msg.infinity_global)?;

    // Pay pair creation fee
    let received_amount = must_pay(&info, &global_config.pair_creation_fee.denom)?;
    ensure_eq!(
        received_amount,
        global_config.pair_creation_fee.amount,
        InfinityError::InsufficientFunds {
            expected: global_config.pair_creation_fee
        }
    );
    response = append_fair_burn_msg(
        &global_config.fair_burn,
        vec![global_config.pair_creation_fee],
        None,
        response,
    );

    INFINITY_GLOBAL.save(deps.storage, &msg.infinity_global)?;

    let mut pair = Pair::initialize(msg.pair_immutable, msg.pair_config);
    response =
        pair.save_and_update_indices(deps.storage, &global_config.infinity_index, response)?;

    response = response
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION);

    Ok(response)
}
