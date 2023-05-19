use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_macro::index_list;
use cw_storage_plus::{IndexedMap, Item, MultiIndex};

pub const INFINITY_GLOBAL: Item<Addr> = Item::new("ig");

#[cw_serde]
pub struct PoolQuote {
    pub pool: Addr,
    pub collection: Addr,
    pub quote_price: Uint128,
}

#[index_list(PoolQuote)]
pub struct BuyPoolQuoteIndices<'a> {
    pub collection_quote_price: MultiIndex<'a, (Addr, u128), PoolQuote, Addr>,
}

pub fn buy_from_pool_quotes<'a>() -> IndexedMap<'a, Addr, PoolQuote, BuyPoolQuoteIndices<'a>> {
    let indexes = BuyPoolQuoteIndices {
        collection_quote_price: MultiIndex::new(
            |_pk: &[u8], p: &PoolQuote| (p.collection.clone(), p.quote_price.u128()),
            "bfpq",
            "bfpq__cqp",
        ),
    };
    IndexedMap::new("bfpq", indexes)
}

#[index_list(PoolQuote)]
pub struct SellPoolQuoteIndices<'a> {
    pub collection_quote_price: MultiIndex<'a, (Addr, u128), PoolQuote, Addr>,
}

pub fn sell_to_pool_quotes<'a>() -> IndexedMap<'a, Addr, PoolQuote, SellPoolQuoteIndices<'a>> {
    let indexes = SellPoolQuoteIndices {
        collection_quote_price: MultiIndex::new(
            |_pk: &[u8], p: &PoolQuote| (p.collection.clone(), p.quote_price.u128()),
            "stpq",
            "stpq__cqp",
        ),
    };
    IndexedMap::new("stpq", indexes)
}
