use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex, Item};

pub const POOL_COUNTER: Item<u64> = Item::new("pool-counter");

/// The `PoolType` refers to what the pool holds
#[cw_serde]
pub enum PoolType {
    /// A `Token` pool has tokens that it is willing to give to traders in exchange for NFTs
    Token,
    /// An `Nft` pool has NFTs that it is willing to give to traders in exchange for tokens
    Nft,
    /// A `Trade` pool allows for both Token-->NFT and NFT-->Token swaps
    Trade,
}

pub type PoolKey = u64;

/// An Infinity Pool that allows for trading of a specific NFT collection
#[cw_serde]
pub struct Pool {
    pub key: PoolKey,
    pub collection: Addr,
    pub pool_type: PoolType,
    pub delta: Uint128,
    pub fee: Uint128,
    pub asset_recipient: Addr,
}

/// Defines indices for accessing Pools
pub struct PoolIndices<'a> {
    pub collection: MultiIndex<'a, Addr, Pool, PoolKey>,
}

impl<'a> IndexList<Pool> for PoolIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Pool>> + '_> {
        let v: Vec<&dyn Index<Pool>> = vec![&self.collection];
        Box::new(v.into_iter())
    }
}

pub fn pools<'a>() -> IndexedMap<'a, PoolKey, Pool, PoolIndices<'a>> {
    let indexes = PoolIndices {
        collection: MultiIndex::new(
            |_pk: &[u8], p: &Pool| p.collection.clone(),
            "pools",
            "pools__collection",
        ),
    };
    IndexedMap::new("pools", indexes)
}