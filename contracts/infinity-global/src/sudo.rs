use crate::{
    msg::SudoMsg,
    state::{GLOBAL_CONFIG, MIN_PRICES},
};

use cosmwasm_std::{attr, Coin, Decimal, DepsMut, Env, Event, StdError};
use sg_std::Response;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, StdError> {
    match msg {
        SudoMsg::UpdateConfig {
            fair_burn,
            royalty_registry,
            marketplace,
            infinity_factory,
            infinity_index,
            infinity_router,
            infinity_pair_code_id,
            pair_creation_fee,
            fair_burn_fee_percent,
            default_royalty_fee_percent,
            max_royalty_fee_percent,
            max_swap_fee_percent,
        } => sudo_update_config(
            deps,
            fair_burn,
            royalty_registry,
            marketplace,
            infinity_factory,
            infinity_index,
            infinity_router,
            infinity_pair_code_id,
            pair_creation_fee,
            fair_burn_fee_percent,
            default_royalty_fee_percent,
            max_royalty_fee_percent,
            max_swap_fee_percent,
        ),
        SudoMsg::AddMinPrices {
            min_prices,
        } => sudo_add_min_prices(deps, min_prices),
        SudoMsg::RemoveMinPrices {
            denoms,
        } => sudo_remove_min_prices(deps, denoms),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn sudo_update_config(
    deps: DepsMut,
    fair_burn: Option<String>,
    royalty_registry: Option<String>,
    marketplace: Option<String>,
    infinity_factory: Option<String>,
    infinity_index: Option<String>,
    infinity_router: Option<String>,
    infinity_pair_code_id: Option<u64>,
    pair_creation_fee: Option<Coin>,
    fair_burn_fee_percent: Option<Decimal>,
    default_royalty_fee_percent: Option<Decimal>,
    max_royalty_fee_percent: Option<Decimal>,
    max_swap_fee_percent: Option<Decimal>,
) -> Result<Response, StdError> {
    let api = deps.api;

    let mut config = GLOBAL_CONFIG.load(deps.storage)?;

    let mut event = Event::new("sudo-update-config");

    if let Some(fair_burn) = fair_burn {
        event = event.add_attribute("fair_burn", &fair_burn);
        config.fair_burn = api.addr_validate(&fair_burn)?;
    }

    if let Some(royalty_registry) = royalty_registry {
        event = event.add_attribute("royalty_registry", &royalty_registry);
        config.royalty_registry = api.addr_validate(&royalty_registry)?;
    }

    if let Some(marketplace) = marketplace {
        event = event.add_attribute("marketplace", &marketplace);
        config.marketplace = api.addr_validate(&marketplace)?;
    }

    if let Some(infinity_factory) = infinity_factory {
        event = event.add_attribute("infinity_factory", &infinity_factory);
        config.infinity_factory = api.addr_validate(&infinity_factory)?;
    }

    if let Some(infinity_index) = infinity_index {
        event = event.add_attribute("infinity_index", &infinity_index);
        config.infinity_index = api.addr_validate(&infinity_index)?;
    }

    if let Some(infinity_router) = infinity_router {
        event = event.add_attribute("infinity_router", &infinity_router);
        config.infinity_router = api.addr_validate(&infinity_router)?;
    }

    if let Some(infinity_pair_code_id) = infinity_pair_code_id {
        event = event.add_attribute("infinity_pair_code_id", infinity_pair_code_id.to_string());
        config.infinity_pair_code_id = infinity_pair_code_id;
    }

    if let Some(pair_creation_fee) = pair_creation_fee {
        event = event.add_attribute("pair_creation_fee", pair_creation_fee.to_string());
        config.pair_creation_fee = pair_creation_fee;
    }

    if let Some(fair_burn_fee_percent) = fair_burn_fee_percent {
        event = event.add_attribute("fair_burn_fee_percent", fair_burn_fee_percent.to_string());
        config.fair_burn_fee_percent = fair_burn_fee_percent;
    }

    if let Some(default_royalty_fee_percent) = default_royalty_fee_percent {
        event = event
            .add_attribute("default_royalty_fee_percent", default_royalty_fee_percent.to_string());
        config.default_royalty_fee_percent = default_royalty_fee_percent;
    }

    if let Some(max_royalty_fee_percent) = max_royalty_fee_percent {
        event = event.add_attribute("max_royalty_fee_percent", max_royalty_fee_percent.to_string());
        config.max_royalty_fee_percent = max_royalty_fee_percent;
    }

    if let Some(max_swap_fee_percent) = max_swap_fee_percent {
        event = event.add_attribute("max_swap_fee_percent", max_swap_fee_percent.to_string());
        config.max_swap_fee_percent = max_swap_fee_percent;
    }

    GLOBAL_CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_event(event))
}

pub fn sudo_add_min_prices(deps: DepsMut, min_prices: Vec<Coin>) -> Result<Response, StdError> {
    let mut event = Event::new("sudo-add-min-prices");
    for min_price in min_prices {
        MIN_PRICES.save(deps.storage, min_price.denom.clone(), &min_price.amount)?;
        event = event.add_attributes(vec![
            attr("denom", min_price.denom.to_string()),
            attr("amount", min_price.amount.to_string()),
        ]);
    }

    Ok(Response::new().add_event(event))
}

pub fn sudo_remove_min_prices(deps: DepsMut, denoms: Vec<String>) -> Result<Response, StdError> {
    let mut event = Event::new("sudo-remove-min-prices");
    for denom in denoms {
        MIN_PRICES.remove(deps.storage, denom.clone());
        event = event.add_attributes(vec![attr("denom", denom.to_string())]);
    }

    Ok(Response::new().add_event(event))
}
