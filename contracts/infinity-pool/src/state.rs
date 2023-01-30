use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};
use std::collections::BTreeSet;
use std::fmt;

pub const POOL_COUNTER: Item<u64> = Item::new("pool-counter");

#[cw_serde]
pub struct Config {
    /// The fungible token used in the child pools
    pub denom: String,
    /// The address of the marketplace contract
    pub marketplace_addr: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

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

#[cw_serde]
pub struct Pool {
    pub id: u64,
    pub collection: Addr,
    pub owner: Addr,
    pub asset_recipient: Option<Addr>,
    pub pool_type: PoolType,
    pub bonding_curve: BondingCurve,
    pub spot_price: Uint128,
    pub delta: Uint128,
    pub total_tokens: Uint128,
    pub nft_token_ids: BTreeSet<String>,
    pub is_active: bool,
    pub finders_fee_bps: u16,
    pub swap_fee_bps: u16,
}

pub struct PoolIndices<'a> {
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

#[cw_serde]
pub struct PoolQuote {
    pub id: u64,
    pub collection: Addr,
    pub quote_price: Uint128,
}

pub struct BuyPoolQuoteIndices<'a> {
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

pub struct SellPoolQuoteIndices<'a> {
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
