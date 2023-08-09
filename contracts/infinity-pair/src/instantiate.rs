use crate::helpers::PayoutContext;
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
use infinity_global::{load_global_config, load_min_price};
use infinity_shared::InfinityError;
use sg_std::Response;
use stargaze_fair_burn::append_fair_burn_msg;
use stargaze_royalty_registry::fetch_or_set_royalties;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let response = Response::new();

    // Store infinity_global address
    let infinity_global = deps.api.addr_validate(&msg.infinity_global)?;
    INFINITY_GLOBAL.save(deps.storage, &infinity_global)?;

    let mut pair = Pair::initialize(
        deps.storage,
        msg.pair_immutable.str_to_addr(deps.api)?,
        msg.pair_config.str_to_addr(deps.api)?,
    )?;

    let global_config = load_global_config(&deps.querier, &infinity_global)?;

    let min_price = load_min_price(&deps.querier, &infinity_global, &pair.immutable.denom)?
        .ok_or(InfinityError::InvalidInput("denom not supported".to_string()))?;

    let (royalty_entry, mut response) = fetch_or_set_royalties(
        deps.as_ref(),
        &global_config.royalty_registry,
        &pair.immutable.collection,
        Some(&infinity_global),
        response,
    )?;

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
        vec![global_config.pair_creation_fee.clone()],
        None,
        response,
    );

    let payout_context = PayoutContext {
        global_config,
        royalty_entry,
        min_price,
        infinity_global,
        denom: pair.immutable.denom.clone(),
    };

    response = pair.save_and_update_indices(deps.storage, &payout_context, response)?;

    response = response
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION);

    Ok(response)
}
