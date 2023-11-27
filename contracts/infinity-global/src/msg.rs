use crate::state::GlobalConfig;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Decimal};

#[cw_serde]
pub struct InstantiateMsg {
    pub global_config: GlobalConfig<String>,
    pub min_prices: Vec<Coin>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GlobalConfig<Addr>)]
    GlobalConfig {},
    #[returns(Option<Coin>)]
    MinPrice {
        denom: String,
    },
}

#[cw_serde]
#[allow(clippy::large_enum_variant)]
pub enum SudoMsg {
    UpdateConfig {
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
    },
    AddMinPrices {
        min_prices: Vec<Coin>,
    },
    RemoveMinPrices {
        denoms: Vec<String>,
    },
}
