use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const INFINITY_GLOBAL: Item<Addr> = Item::new("g");
pub const SENDER_COUNTER: Map<Addr, u64> = Map::new("s");
