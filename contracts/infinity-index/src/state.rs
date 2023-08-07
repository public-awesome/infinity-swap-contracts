use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};
use cw_storage_macro::index_list;
use cw_storage_plus::{IndexedMap, Item, MultiIndex};

pub const INFINITY_GLOBAL: Item<Addr> = Item::new("g");

#[cw_serde]
pub struct PairQuote {
    pub pair: Addr,
    pub collection: Addr,
    pub quote: Coin,
}

#[index_list(PairQuote)]
pub struct BuyPairQuoteIndices<'a> {
    pub collection_quote: MultiIndex<'a, (Addr, String, u128), PairQuote, Addr>,
}

pub fn buy_from_pair_quotes<'a>() -> IndexedMap<'a, Addr, PairQuote, BuyPairQuoteIndices<'a>> {
    let indexes = BuyPairQuoteIndices {
        collection_quote: MultiIndex::new(
            |_pk: &[u8], p: &PairQuote| {
                (p.collection.clone(), p.quote.denom.clone(), p.quote.amount.u128())
            },
            "b",
            "bq",
        ),
    };
    IndexedMap::new("b", indexes)
}

#[index_list(PairQuote)]
pub struct SellPairQuoteIndices<'a> {
    pub collection_quote: MultiIndex<'a, (Addr, String, u128), PairQuote, Addr>,
}

pub fn sell_to_pair_quotes<'a>() -> IndexedMap<'a, Addr, PairQuote, SellPairQuoteIndices<'a>> {
    let indexes = SellPairQuoteIndices {
        collection_quote: MultiIndex::new(
            |_pk: &[u8], p: &PairQuote| {
                (p.collection.clone(), p.quote.denom.clone(), p.quote.amount.u128())
            },
            "s",
            "sq",
        ),
    };
    IndexedMap::new("s", indexes)
}
