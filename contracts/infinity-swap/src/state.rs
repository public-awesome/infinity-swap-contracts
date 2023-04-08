use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};
use std::fmt;

/// An incrementing uint used as the primary key for pools
pub const POOL_COUNTER: Item<u64> = Item::new("pool-counter");

/// The global configuration object for the protocol
#[cw_serde]
pub struct Config {
    /// The address of the marketplace contract
    pub marketplace_addr: Addr,
    /// The address of the developer who will receive a portion of the fair burn
    pub developer: Option<Addr>,
}

pub const CONFIG: Item<Config> = Item::new("config");

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

/// Pool represents a pool of assets that can be swapped
#[cw_serde]
pub struct Pool {
    /// The unique id of the pool
    pub id: u64,
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
    /// The total amount of tokens held by the pool
    pub total_tokens: Uint128,
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

/// PoolIndices defines the indices for the Pool type
pub struct PoolIndices<'a> {
    /// Indexes pools by the owner address
    pub owner: MultiIndex<'a, Addr, Pool, u64>,
}

impl<'a> IndexList<Pool> for PoolIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Pool>> + '_> {
        let v: Vec<&dyn Index<Pool>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

pub fn pools<'a>() -> IndexedMap<'a, u64, Pool, PoolIndices<'a>> {
    let indexes = PoolIndices {
        owner: MultiIndex::new(
            |_pk: &[u8], p: &Pool| p.owner.clone(),
            "pools",
            "pools__owner",
        ),
    };
    IndexedMap::new("pools", indexes)
}

/// PoolQuote represents a quote for a pool, at which assets can be bought or sold
#[cw_serde]
pub struct PoolQuote {
    /// The unique id of the pool quote, also corresponds to a pool_id
    pub id: u64,
    /// The address of the NFT collection contract
    pub collection: Addr,
    /// The price at which assets can be bought or sold
    pub quote_price: Uint128,
}

/// BuyPoolQuoteIndices defines the indices for the PoolQuote type
pub struct BuyPoolQuoteIndices<'a> {
    /// Indexes pool quotes by the collection address and buy quote price
    pub collection_buy_price: MultiIndex<'a, (Addr, u128), PoolQuote, u64>,
}

impl<'a> IndexList<PoolQuote> for BuyPoolQuoteIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<PoolQuote>> + '_> {
        let v: Vec<&dyn Index<PoolQuote>> = vec![&self.collection_buy_price];
        Box::new(v.into_iter())
    }
}

pub fn buy_pool_quotes<'a>() -> IndexedMap<'a, u64, PoolQuote, BuyPoolQuoteIndices<'a>> {
    let indexes = BuyPoolQuoteIndices {
        collection_buy_price: MultiIndex::new(
            |_pk: &[u8], p: &PoolQuote| (p.collection.clone(), p.quote_price.u128()),
            "buy_pool_quotes",
            "buy_pool_quotes__collection_buy_price",
        ),
    };
    IndexedMap::new("buy_pool_quotes", indexes)
}

/// SellPoolQuoteIndices defines the indices for the PoolQuote type
pub struct SellPoolQuoteIndices<'a> {
    /// Indexes pool quotes by the collection address and sell quote price
    pub collection_sell_price: MultiIndex<'a, (Addr, u128), PoolQuote, u64>,
}

impl<'a> IndexList<PoolQuote> for SellPoolQuoteIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<PoolQuote>> + '_> {
        let v: Vec<&dyn Index<PoolQuote>> = vec![&self.collection_sell_price];
        Box::new(v.into_iter())
    }
}

pub fn sell_pool_quotes<'a>() -> IndexedMap<'a, u64, PoolQuote, SellPoolQuoteIndices<'a>> {
    let indexes = SellPoolQuoteIndices {
        collection_sell_price: MultiIndex::new(
            |_pk: &[u8], p: &PoolQuote| (p.collection.clone(), p.quote_price.u128()),
            "sell_pool_quotes",
            "sell_pool_quotes__collection_sell_price",
        ),
    };
    IndexedMap::new("sell_pool_quotes", indexes)
}

/// NftDepositKey is comprised of the pool id and the token id
pub type NftDepositKey = (u64, String);

/// Nft deposits are used to track the NFTs that have been deposited into a pool
pub const NFT_DEPOSITS: Map<NftDepositKey, bool> = Map::new("nft_deposits");
