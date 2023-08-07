use cosmwasm_std::Addr;
use cw_storage_plus::Item;

// The address of the infinity global contract
pub const INFINITY_GLOBAL: Item<Addr> = Item::new("g");
