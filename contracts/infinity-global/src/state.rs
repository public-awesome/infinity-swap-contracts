use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::Item;

#[cw_serde]
pub struct GlobalConfig {
    pub infinity_index: Addr,
    pub infinity_factory: Addr,
    pub infinity_pool_code_id: u64,
    pub marketplace: Addr,
    pub min_price: Uint128,
    pub pool_creation_fee: Uint128,
    pub trading_fee_percent: Decimal,
}

pub const GLOBAL_CONFIG: Item<GlobalConfig> = Item::new("gc");
