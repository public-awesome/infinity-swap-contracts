use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

/// The global configuration object for the contract
#[cw_serde]
pub struct Config {
    /// The address of the marketplace adapter contract
    pub marketplace: Addr,
    /// The address of the infinity swap contract
    pub infinity_swap: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
