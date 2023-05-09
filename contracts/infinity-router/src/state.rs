use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_macro::index_list;
use cw_storage_plus::{IndexedMap, Item, Map, MultiIndex};

#[cw_serde]
pub struct Config {
    pub infinity_index: Addr,
}

pub const CONFIG: Item<Config> = Item::new("cfg");
