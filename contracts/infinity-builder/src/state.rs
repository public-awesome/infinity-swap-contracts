use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const INFINITY_GLOBAL: Item<Addr> = Item::new("ig");
