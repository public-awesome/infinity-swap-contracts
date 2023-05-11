use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Item, Map};
use std::fmt;

/// PoolType refers to the assets held by the pool
/// * Token: A pool that holds fungible tokens
/// * Nft: A pool that holds NFTs
/// * Trade: A pool that holds both fungible tokens and NFTs
#[cw_serde]
pub enum PoolType {
    Token,
    Nft,
    Trade,
}

impl fmt::Display for PoolType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// BondingCurve refers to the curve used to calculate the spot price for the pool
/// * Linear: A linear curve that increments by a constant amount (delta)
/// * Exponential: An exponential curve that increments by a percentage amount (delta)
/// * ConstantProduct: A constant product curve that maintains a constant product of the two assets
#[cw_serde]
pub enum BondingCurve {
    Linear,
    Exponential,
    ConstantProduct,
}

impl fmt::Display for BondingCurve {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// PoolConfig represents the configuration parameters for a pool
#[cw_serde]
pub struct PoolConfig {
    /// The address of the NFT collection contract
    pub collection: Addr,
    /// The address of the pool owner
    pub owner: Addr,
    /// The address of the recipient of assets traded into the pool
    pub asset_recipient: Option<Addr>,
    /// The type of assets held by the pool
    pub pool_type: PoolType,
    /// The bonding curve used to calculate the spot price
    pub bonding_curve: BondingCurve,
    /// A moving value used to derive the price at which the pool will trade assets
    /// Note: this value is not necessarily the final sale price for pool assets
    pub spot_price: Uint128,
    /// The amount by which the spot price will increment/decrement
    /// For linear curves, this is the constant amount
    /// For exponential curves, this is the percentage amount (treated as basis points)
    pub delta: Uint128,
    /// The total number of NFTs held by the pool
    pub total_nfts: u64,
    /// The percentage of the swap that will be paid to the finder of a trade
    pub finders_fee_percent: Decimal,
    /// The percentage of the swap that will be paid to the pool owner
    /// Note: this only applies to Trade pools
    pub swap_fee_percent: Decimal,
    /// Whether or not the pool is accepting trades
    pub is_active: bool,
    /// Whether or not the tokens sold into the pool will be reinvested
    pub reinvest_tokens: bool,
    /// Whether or not the NFTs sold into the pool will be reinvested
    pub reinvest_nfts: bool,
}

pub const POOL_CONFIG: Item<PoolConfig> = Item::new("pc");

// The address of the global gov contract
pub const MARKETPLACE: Item<Addr> = Item::new("mp");

// The address of the infinity index contract
pub const INFINITY_INDEX: Item<Addr> = Item::new("ii");

// A map of all NFT token ids held by the pool
pub const NFT_DEPOSITS: Map<String, bool> = Map::new("nd");
