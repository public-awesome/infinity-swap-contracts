use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

pub const INFINITY_GLOBAL: Item<Addr> = Item::new("g");

// (sender, code_id) => counter
pub const SENDER_COUNTER: Map<(Addr, u64), u64> = Map::new("s");

// code_id => code_id
// This is a map of code ids that are allowed to migrate to subsequent code ids.
// This set of migrations can be invoked by anyone.
pub const UNRESTRICTED_MIGRATIONS: Map<u64, u64> = Map::new("um");
