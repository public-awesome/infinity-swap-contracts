use crate::error::ContractError;
use crate::msg::InstantiateMsg;
use crate::pool::Pool;
use crate::state::{PoolConfig, INFINITY_INDEX, MARKETPLACE};
use cosmwasm_std::{Decimal, DepsMut, Env, MessageInfo, Uint128};
use cw2::set_contract_version;
use cw_utils::maybe_addr;
use sg_std::Response;

pub const CONTRACT_NAME: &str = "crates.io:infinity-pool";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let marketplace = deps.api.addr_validate(&msg.marketplace)?;
    MARKETPLACE.save(deps.storage, &marketplace)?;

    let infinity_index = deps.api.addr_validate(&msg.infinity_index)?;
    INFINITY_INDEX.save(deps.storage, &infinity_index)?;

    let pool_config = PoolConfig {
        collection: deps.api.addr_validate(&msg.pool_info.collection)?,
        owner: deps.api.addr_validate(&msg.pool_info.owner)?,
        asset_recipient: maybe_addr(deps.api, msg.pool_info.asset_recipient)?,
        pool_type: msg.pool_info.pool_type.clone(),
        bonding_curve: msg.pool_info.bonding_curve,
        spot_price: msg.pool_info.spot_price,
        delta: msg.pool_info.delta,
        total_nfts: 0u64,
        finders_fee_percent: Decimal::percent(msg.pool_info.finders_fee_bps),
        swap_fee_percent: Decimal::percent(msg.pool_info.swap_fee_bps),
        is_active: false,
        reinvest_tokens: msg.pool_info.reinvest_tokens,
        reinvest_nfts: msg.pool_info.reinvest_nfts,
    };

    let mut pool = Pool::new(pool_config, Uint128::zero());
    pool.save(deps.storage)?;
    let mut response = Response::new().add_event(pool.create_event_all_props("create-pool")?);

    response = response
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("marketplace", msg.marketplace.to_string())
        .add_attribute("infinity_controller", msg.infinity_index.to_string());

    Ok(response)
}
