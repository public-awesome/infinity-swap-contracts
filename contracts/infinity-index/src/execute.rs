use crate::helpers::only_infinity_pair;
use crate::msg::ExecuteMsg;
use crate::state::PairQuote;
use crate::{
    error::ContractError,
    state::{buy_from_pair_quotes, sell_to_pair_quotes},
};

use cosmwasm_std::{coin, Addr, DepsMut, Env, MessageInfo, Uint128};
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
    let api = deps.api;

    match msg {
        ExecuteMsg::UpdatePairIndices {
            collection,
            denom,
            sell_to_pair_quote,
            buy_from_pair_quote,
        } => execute_update_pair_indices(
            deps,
            env,
            info,
            api.addr_validate(&collection)?,
            denom,
            sell_to_pair_quote,
            buy_from_pair_quote,
        ),
    }
}

pub fn execute_update_pair_indices(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection: Addr,
    denom: String,
    sell_to_pair_quote: Option<Uint128>,
    buy_from_pair_quote: Option<Uint128>,
) -> Result<Response, ContractError> {
    only_infinity_pair(deps.as_ref(), &info.sender)?;

    match sell_to_pair_quote {
        Some(amount) => {
            sell_to_pair_quotes().save(
                deps.storage,
                info.sender.clone(),
                &PairQuote {
                    pair: info.sender.clone(),
                    collection: collection.clone(),
                    quote: coin(amount.u128(), denom.clone()),
                },
            )?;
        },
        None => {
            sell_to_pair_quotes().remove(deps.storage, info.sender.clone())?;
        },
    };

    match buy_from_pair_quote {
        Some(amount) => {
            buy_from_pair_quotes().save(
                deps.storage,
                info.sender.clone(),
                &PairQuote {
                    pair: info.sender.clone(),
                    collection: collection.clone(),
                    quote: coin(amount.u128(), denom.clone()),
                },
            )?;
        },
        None => {
            buy_from_pair_quotes().remove(deps.storage, info.sender.clone())?;
        },
    };

    Ok(Response::new())
}
