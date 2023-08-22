use crate::{msg::PairOptions, ContractError};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ensure, Addr, Api, Coin, Decimal, Storage, Uint128};
use cw_address_like::AddressLike;
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};
use std::fmt;

pub type Denom = String;
pub type TokenId = String;

/// An incrementing uint used as the identifier for Pairs
pub const PAIR_COUNTER: Item<u64> = Item::new("c");

/// The global configuration object for the protocol
#[cw_serde]
pub struct SudoParams<T: AddressLike> {
    /// The address of the fair burn contract
    pub fair_burn: T,
    /// The address of the royalty registry contract
    pub royalty_registry: T,
    /// Fee to reduce spam
    pub create_pool_fee: Coin,
    /// Fair Burn fee
    pub trading_fee_percent: Decimal,
    /// Max value for the royalty fee
    pub max_royalty_fee_percent: Decimal,
    /// Max value for the finders fee
    pub max_finders_fee_percent: Decimal,
}

pub const SUDO_PARAMS: Item<SudoParams<Addr>> = Item::new("p");

impl SudoParams<String> {
    pub fn str_to_addr(self, api: &dyn Api) -> Result<SudoParams<Addr>, ContractError> {
        Ok(SudoParams {
            fair_burn: api.addr_validate(&self.fair_burn)?,
            royalty_registry: api.addr_validate(&self.royalty_registry)?,
            create_pool_fee: self.create_pool_fee,
            trading_fee_percent: self.trading_fee_percent,
            max_royalty_fee_percent: self.max_royalty_fee_percent,
            max_finders_fee_percent: self.max_finders_fee_percent,
        })
    }
}

impl SudoParams<Addr> {
    pub fn save(&self, storage: &mut dyn Storage) -> Result<(), ContractError> {
        SUDO_PARAMS.save(storage, self)?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        ensure!(
            self.trading_fee_percent < Decimal::one(),
            ContractError::InvalidInput("trade_fee_percent must be less than 1".to_string())
        );
        ensure!(
            self.max_finders_fee_percent < Decimal::one(),
            ContractError::InvalidInput("max_finders_fee_percent must be less than 1".to_string())
        );

        Ok(())
    }
}

/// NftDepositKey is comprised of the pair id and the token id
pub type NftDepositKey = (u64, String);

/// Nft deposits are used to track the NFTs that have been deposited into a pair
pub const NFT_DEPOSITS: Map<NftDepositKey, bool> = Map::new("n");

/// Pair refers to the assets held in the pair
/// * Token: An pair that holds fungible tokens
/// * Nft: An pair that holds NFTs
/// * Trade: An pair that holds both fungible tokens and NFTs
#[cw_serde]
pub enum PairType {
    Token {
        /// The total amount of tokens held by the pair
        total_tokens: Uint128,
    },
    Nft {
        /// The total number of NFTs held by the pair
        total_nfts: u64,
    },
    Trade {
        /// The total amount of tokens held by the pair
        total_tokens: Uint128,
        /// The total number of NFTs held by the pair
        total_nfts: u64,
        /// The percentage of the swap that will be paid to the pair owner
        /// Note: this only applies to Trade pairs
        swap_fee_percent: Decimal,
        /// Whether or not the tokens sold into the pair will be reinvested
        reinvest_tokens: bool,
        /// Whether or not the NFTs sold into the pair will be reinvested
        reinvest_nfts: bool,
    },
}

impl fmt::Display for Pair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// BondingCurve refers to the curve used to calculate the current spot price for the pair
/// * Linear: A linear curve that increments by a constant amount (delta)
/// * Exponential: An exponential curve that increments by a percentage amount (delta)
/// * ConstantProduct: A constant product curve that maintains a constant product of the two assets
#[cw_serde]
pub enum BondingCurve {
    Linear {
        /// A moving value used to derive the price at which the pair will trade assets
        /// Note: this value is not necessarily the final sale price for pair assets
        spot_price: Uint128,
        /// The amount by which the spot price will increment/decrement
        /// For linear curves, this is the constant amount
        /// For exponential curves, this is the percentage amount (treated as basis points)
        delta: Uint128,
    },
    Exponential {
        /// A moving value used to derive the price at which the pair will trade assets
        /// Note: this value is not necessarily the final sale price for pair assets
        spot_price: Uint128,
        /// The amount by which the spot price will increment/decrement
        /// For linear curves, this is the constant amount
        /// For exponential curves, this is the percentage amount (treated as basis points)
        delta: Uint128,
    },
    ConstantProduct,
}

impl fmt::Display for BondingCurve {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Pair represents a set of assets that can be swapped
#[cw_serde]
pub struct Pair {
    /// The unique id of the pair
    pub id: u64,
    /// The address of the NFT collection contract
    pub collection: Addr,
    /// The address of the pair owner
    pub owner: Addr,
    /// The denom of the fungible token in the pair
    pub denom: String,
    /// Whether or not the pair is accepting trades
    pub is_active: bool,
    /// The type of assets held by the pair
    pub pair_type: PairType,
    /// The bonding curve used to calculate the spot price
    pub bonding_curve: BondingCurve,
    /// The options for the pair
    pub pair_options: PairOptions<Addr>,
}

/// PairIndices defines the indices for the Pair type
pub struct PairIndices<'a> {
    /// Indexes pairs by the owner address
    pub owner: MultiIndex<'a, Addr, Pair, u64>,
}

impl<'a> IndexList<Pair> for PairIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Pair>> + '_> {
        let v: Vec<&dyn Index<Pair>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

pub fn pairs<'a>() -> IndexedMap<'a, u64, Pair, PairIndices<'a>> {
    let indexes = PairIndices {
        owner: MultiIndex::new(|_pk: &[u8], p: &Pair| p.owner.clone(), "o", "oo"),
    };
    IndexedMap::new("o", indexes)
}

// /// PairQuote represents a quote at which assets can be bought or sold from a pair
// #[cw_serde]
// pub struct PairQuote {
//     /// The unique id of the pair quote, also corresponds to a pair_id
//     pub id: u64,
//     /// The address of the NFT collection contract
//     pub collection: Addr,
//     /// The price at which assets can be bought or sold
//     pub quote: Coin,
// }

// /// BuyPairQuoteIndices defines the indices for the PairQuote type
// pub struct BuyPairQuoteIndices<'a> {
//     /// Indexes pair quotes by the collection address and buy quote price
//     pub collection_buy_price: MultiIndex<'a, (Addr, Denom, u128), PairQuote, u64>,
// }

// impl<'a> IndexList<PairQuote> for BuyPairQuoteIndices<'a> {
//     fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<PairQuote>> + '_> {
//         let v: Vec<&dyn Index<PairQuote>> = vec![&self.collection_buy_price];
//         Box::new(v.into_iter())
//     }
// }

// pub fn buy_from_pair_quotes<'a>() -> IndexedMap<'a, u64, PairQuote, BuyPairQuoteIndices<'a>> {
//     let indexes = BuyPairQuoteIndices {
//         collection_buy_price: MultiIndex::new(
//             |_pk: &[u8], p: &PairQuote| {
//                 (p.collection.clone(), p.quote.denom, p.quote.amount.u128())
//             },
//             "b",
//             "bp",
//         ),
//     };
//     IndexedMap::new("b", indexes)
// }

// /// SellPairQuoteIndices defines the indices for the PairQuote type
// pub struct SellPairQuoteIndices<'a> {
//     /// Indexes pair quotes by the collection address and sell quote price
//     pub collection_sell_price: MultiIndex<'a, (Addr, u128), PairQuote, u64>,
// }

// impl<'a> IndexList<PairQuote> for SellPairQuoteIndices<'a> {
//     fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<PairQuote>> + '_> {
//         let v: Vec<&dyn Index<PairQuote>> = vec![&self.collection_sell_price];
//         Box::new(v.into_iter())
//     }
// }

// pub fn sell_to_pair_quotes<'a>() -> IndexedMap<'a, u64, PairQuote, SellPairQuoteIndices<'a>> {
//     let indexes = SellPairQuoteIndices {
//         collection_sell_price: MultiIndex::new(
//             (p.collection.clone(), p.quote.denom, p.quote.amount.u128())
//             "s",
//             "sp",
//         ),
//     };
//     IndexedMap::new("s", indexes)
// }
