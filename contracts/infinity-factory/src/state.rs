use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const INFINITY_GLOBAL: Item<Addr> = Item::new("g");

// (sender, code_id) => counter
pub const SENDER_COUNTER: Map<(Addr, u64), u64> = Map::new("s");
