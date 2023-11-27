use crate::{
    msg::QueryMsg,
    state::{GLOBAL_CONFIG, MIN_PRICES},
};

use cosmwasm_std::{coin, to_binary, Binary, Deps, Env, StdResult};

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GlobalConfig {} => to_binary(&GLOBAL_CONFIG.load(deps.storage)?),
        QueryMsg::MinPrice {
            denom,
        } => {
            let min_amount = MIN_PRICES.may_load(deps.storage, denom.clone())?;
            to_binary(&Some(min_amount.map(|a| coin(a.u128(), denom))))
        },
    }
}
