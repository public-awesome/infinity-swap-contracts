use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

/// The global configuration object for the contract
#[cw_serde]
pub struct Config {
    /// The address of the marketplace contract
    pub marketplace: Addr,
    /// The max number of NFT swaps that can be processed in a single message
    pub max_batch_size: u32,
}

pub const CONFIG: Item<Config> = Item::new("config");
