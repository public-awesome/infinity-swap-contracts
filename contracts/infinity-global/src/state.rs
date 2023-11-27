use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, StdError, Uint128};
use cosmwasm_std::{Api, Coin, Decimal};
use cw_address_like::AddressLike;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct GlobalConfig<T: AddressLike> {
    /// The address of the FairBurn contract
    pub fair_burn: T,
    /// The address of the RoyaltyRegistry contract
    pub royalty_registry: T,
    /// The address of the Marketplace contract
    pub marketplace: T,
    /// The address of the InfinityFactory contract
    pub infinity_factory: T,
    /// The address of the InfinityIndex contract
    pub infinity_index: T,
    /// The address of the InfinityRouter contract
    pub infinity_router: T,
    /// The code ID of the InfinityPair code
    pub infinity_pair_code_id: u64,
    /// The fee to create a pair
    pub pair_creation_fee: Coin,
    /// The percentage amount of a sale that is paid to the FairBurn contract
    pub fair_burn_fee_percent: Decimal,
    /// The royalty percentage amount to be paid when no royalty is specified for the protocol
    pub default_royalty_fee_percent: Decimal,
    /// The maximum percentage amount of a sale that can be paid in royalties
    pub max_royalty_fee_percent: Decimal,
    /// The maximum percentage amount of a sale that can be paid to LPs
    pub max_swap_fee_percent: Decimal,
}

impl GlobalConfig<String> {
    pub fn str_to_addr(self, api: &dyn Api) -> Result<GlobalConfig<Addr>, StdError> {
        Ok(GlobalConfig {
            fair_burn: api.addr_validate(&self.fair_burn)?,
            royalty_registry: api.addr_validate(&self.royalty_registry)?,
            marketplace: api.addr_validate(&self.marketplace)?,
            infinity_factory: api.addr_validate(&self.infinity_factory)?,
            infinity_index: api.addr_validate(&self.infinity_index)?,
            infinity_router: api.addr_validate(&self.infinity_router)?,
            infinity_pair_code_id: self.infinity_pair_code_id,
            pair_creation_fee: self.pair_creation_fee,
            fair_burn_fee_percent: self.fair_burn_fee_percent,
            default_royalty_fee_percent: self.default_royalty_fee_percent,
            max_royalty_fee_percent: self.max_royalty_fee_percent,
            max_swap_fee_percent: self.max_swap_fee_percent,
        })
    }
}

pub const GLOBAL_CONFIG: Item<GlobalConfig<Addr>> = Item::new("g");

pub const MIN_PRICES: Map<String, Uint128> = Map::new("m");
