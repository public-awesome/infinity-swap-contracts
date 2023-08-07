use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Empty, Env, MessageInfo, QuerierWrapper, StdError,
    StdResult,
};
use cosmwasm_std::{Api, Coin, Decimal};
use cw2::set_contract_version;
use cw_address_like::AddressLike;
use cw_storage_plus::Item;
use sg_std::Response;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cw_serde]
pub struct GlobalConfig<T: AddressLike> {
    pub fair_burn: T,
    pub royalty_registry: T,
    pub marketplace: T,
    pub infinity_index: T,
    pub infinity_factory: T,
    pub infinity_pair_code_id: u64,
    pub pair_creation_fee: Coin,
    pub fair_burn_fee_percent: Decimal,
    pub max_royalty_fee_percent: Decimal,
    pub max_swap_fee_percent: Decimal,
}

impl GlobalConfig<String> {
    pub fn str_to_addr(self, api: &dyn Api) -> Result<GlobalConfig<Addr>, StdError> {
        Ok(GlobalConfig {
            fair_burn: api.addr_validate(&self.fair_burn)?,
            royalty_registry: api.addr_validate(&self.royalty_registry)?,
            marketplace: api.addr_validate(&self.marketplace)?,
            infinity_index: api.addr_validate(&self.infinity_index)?,
            infinity_factory: api.addr_validate(&self.infinity_factory)?,
            infinity_pair_code_id: self.infinity_pair_code_id,
            pair_creation_fee: self.pair_creation_fee,
            fair_burn_fee_percent: self.fair_burn_fee_percent,
            max_royalty_fee_percent: self.max_royalty_fee_percent,
            max_swap_fee_percent: self.max_swap_fee_percent,
        })
    }
}

pub const GLOBAL_CONFIG: Item<GlobalConfig<Addr>> = Item::new("g");

#[cw_serde]
pub struct InstantiateMsg {
    pub global_config: GlobalConfig<String>,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, StdError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let global_config = msg.global_config.str_to_addr(deps.api)?;
    GLOBAL_CONFIG.save(deps.storage, &global_config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: Empty,
) -> Result<Response, StdError> {
    unimplemented!("execute not implemented")
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GlobalConfig<Addr>)]
    GlobalConfig {},
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GlobalConfig {} => to_binary(&GLOBAL_CONFIG.load(deps.storage)?),
    }
}

pub fn load_global_config(
    querier: &QuerierWrapper,
    infinity_global: &Addr,
) -> StdResult<GlobalConfig<Addr>> {
    Ok(querier
        .query_wasm_smart::<GlobalConfig<Addr>>(infinity_global, &QueryMsg::GlobalConfig {})?)
}

#[cw_serde]
pub enum SudoMsg {
    UpdateConfig {
        fair_burn: Option<String>,
        royalty_registry: Option<String>,
        marketplace: Option<String>,
        infinity_index: Option<String>,
        infinity_factory: Option<String>,
        infinity_pair_code_id: Option<u64>,
        pair_creation_fee: Option<Coin>,
        fair_burn_fee_percent: Option<Decimal>,
        max_royalty_fee_percent: Option<Decimal>,
        max_swap_fee_percent: Option<Decimal>,
    },
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, StdError> {
    let api = deps.api;

    match msg {
        SudoMsg::UpdateConfig {
            fair_burn,
            royalty_registry,
            marketplace,
            infinity_index,
            infinity_factory,
            infinity_pair_code_id,
            pair_creation_fee,
            fair_burn_fee_percent,
            max_royalty_fee_percent,
            max_swap_fee_percent,
        } => {
            let mut config = GLOBAL_CONFIG.load(deps.storage)?;

            if let Some(fair_burn) = fair_burn {
                config.fair_burn = api.addr_validate(&fair_burn)?;
            }

            if let Some(royalty_registry) = royalty_registry {
                config.royalty_registry = api.addr_validate(&royalty_registry)?;
            }

            if let Some(marketplace) = marketplace {
                config.marketplace = api.addr_validate(&marketplace)?;
            }

            if let Some(infinity_index) = infinity_index {
                config.infinity_index = api.addr_validate(&infinity_index)?;
            }

            if let Some(infinity_factory) = infinity_factory {
                config.infinity_factory = api.addr_validate(&infinity_factory)?;
            }

            if let Some(infinity_pair_code_id) = infinity_pair_code_id {
                config.infinity_pair_code_id = infinity_pair_code_id;
            }

            if let Some(pair_creation_fee) = pair_creation_fee {
                config.pair_creation_fee = pair_creation_fee;
            }

            if let Some(fair_burn_fee_percent) = fair_burn_fee_percent {
                config.fair_burn_fee_percent = fair_burn_fee_percent;
            }

            if let Some(max_royalty_fee_percent) = max_royalty_fee_percent {
                config.max_royalty_fee_percent = max_royalty_fee_percent;
            }

            if let Some(max_swap_fee_percent) = max_swap_fee_percent {
                config.max_swap_fee_percent = max_swap_fee_percent;
            }

            GLOBAL_CONFIG.save(deps.storage, &config)?;

            Ok(Response::new())
        },
    }
}
