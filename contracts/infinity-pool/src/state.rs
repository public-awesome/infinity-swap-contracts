use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex, Item};

pub const POOL_COUNTER: Item<u64> = Item::new("pool-counter");

#[cw_serde]
pub struct Config {
    /// The fungible token used in the child pools
    pub denom: String,
    /// The address of the marketplace contract
    pub marketplace_addr: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub type PoolKey = (Addr, u64);

pub fn pool_key(collection: &Addr, id: u64) -> PoolKey {
    (collection.clone(), id)
}

#[cw_serde]
pub enum PoolType {
    Token,
    Nft,
    Trade,
}

#[cw_serde]
pub enum BondingCurve {
    Linear,
    Exponential,
    ConstantProduct,
}

#[cw_serde]
pub struct Pool {
    pub id: u64,
    pub collection: Addr,
    pub pool_type: PoolType,
    pub bonding_curve: BondingCurve,
    pub delta: Uint128,
    pub fee: Uint128,
    pub asset_recipient: Addr,
    pub buy_price_quote: Uint128,
    pub sell_price_quote: Uint128,
}

/// Defines indices for accessing Pools
pub struct PoolIndices<'a> {
	pub collection_buy_price: MultiIndex<'a, (Addr, u128), Pool, PoolKey>,
	pub collection_sell_price: MultiIndex<'a, (Addr, u128), Pool, PoolKey>,
}

impl<'a> IndexList<Pool> for PoolIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Pool>> + '_> {
        let v: Vec<&dyn Index<Pool>> = vec![&self.collection_buy_price, &self.collection_sell_price];
        Box::new(v.into_iter())
    }
}

pub fn pools<'a>() -> IndexedMap<'a, PoolKey, Pool, PoolIndices<'a>> {
    let indexes = PoolIndices {
        collection_buy_price: MultiIndex::new(
            |_pk: &[u8], p: &Pool| (p.collection.clone(), p.buy_price_quote.u128()),
            "pools",
            "pools__collection_buy_price",
        ),
        collection_sell_price: MultiIndex::new(
            |_pk: &[u8], p: &Pool| (p.collection.clone(), p.sell_price_quote.u128()),
            "pools",
            "pools__collection_sell_price",
        ),
    };
    IndexedMap::new("pools", indexes)
}