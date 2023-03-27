use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

/// The global configuration object for the protocol
#[cw_serde]
pub struct Config {
    /// The address of the marketplace contract
    pub marketplace: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");
