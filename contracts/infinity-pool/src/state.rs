use crate::ContractError;

use std::collections::BTreeSet;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex, Item, Map};
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
    pub fee_bps: u16,
    pub is_active: bool,
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

impl Pool {
    pub fn new(
        id: u64,
        collection: Addr,
        owner: Addr,
        asset_recipient: Option<Addr>,
        pool_type: PoolType,
        bonding_curve: BondingCurve,
        spot_price: Uint128,
        delta: Uint128,
        fee_bps: u16,
    ) -> Self {
        Self {
            id,
            collection,
            owner,
            asset_recipient,
            pool_type,
            bonding_curve,
            spot_price,
            delta,
            total_tokens: Uint128::zero(),
            nft_token_ids: BTreeSet::new(),
            fee_bps,
            is_active: false,
        }
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if self.bonding_curve == BondingCurve::ConstantProduct && self.pool_type != PoolType::Trade {
            return Err(ContractError::InvalidPool("constant product bonding curve must be used with trade pool".to_string()));
        }
        match &self.pool_type {
            PoolType::Token => {
                if self.nft_token_ids.len() > 0 {
                    return Err(ContractError::InvalidPool("nft_token_ids must be empty for token pool".to_string()));
                }
                if self.fee_bps > 0 {
                    return Err(ContractError::InvalidPool("fee_bps must be 0 for token pool".to_string()));
                }
            }
            PoolType::Nft => {
                if self.total_tokens > Uint128::zero() {
                    return Err(ContractError::InvalidPool("total_tokens must be zero for nft pool".to_string()));
                }
                if self.fee_bps > 0 {
                    return Err(ContractError::InvalidPool("fee_bps must be 0 for nft pool".to_string()));
                }
            }
            PoolType::Trade => {
                if self.total_tokens == Uint128::zero() {
                    return Err(ContractError::InvalidPool("total_tokens must be greater than zero for trade pool".to_string()));
                }
                if self.nft_token_ids.len() == 0 {
                    return Err(ContractError::InvalidPool("nft_token_ids must be non-empty for trade pool".to_string()));
                }
                if self.fee_bps > 10000 {
                    return Err(ContractError::InvalidPool("fee_bps is greater than 10000".to_string()));
                }
            }
        }

        Ok(())
    }

    pub fn set_active(&mut self, is_active: bool) -> Result<(), ContractError> {
        self.is_active = is_active;
        Ok(())
    }

    pub fn deposit_tokens(&mut self, amount: Uint128) -> Result<(), ContractError> {
        if self.pool_type == PoolType::Nft {
            return Err(ContractError::InvalidPool("cannot deposit tokens into nft pool".to_string()));
        }
        self.total_tokens += amount;
        Ok(())
    }

    pub fn deposit_nfts(&mut self, nft_token_ids: &Vec<String>) -> Result<(), ContractError> {
        if self.pool_type == PoolType::Token {
            return Err(ContractError::InvalidPool("cannot deposit nfts into token pool".to_string()));
        }
        for nft_token_id in nft_token_ids {
            self.nft_token_ids.insert(nft_token_id.clone());
        }
        Ok(())
    }

    pub fn withdraw_tokens(&mut self, amount: Uint128) -> Result<(), ContractError> {
        if self.pool_type == PoolType::Nft {
            return Err(ContractError::InvalidPool("cannot withdraw tokens from nft pool".to_string()));
        }
        if self.total_tokens < amount {
            return Err(ContractError::InvalidPool("insufficient tokens in pool".to_string()));
        }
        self.total_tokens -= amount;
        Ok(())
    }

    pub fn withdraw_nfts(&mut self, nft_token_ids: &Vec<String>) -> Result<(), ContractError> {
        if self.pool_type == PoolType::Token {
            return Err(ContractError::InvalidPool("cannot withdraw nfts from token pool".to_string()));
        }
        for nft_token_id in nft_token_ids {
            if !self.nft_token_ids.contains(nft_token_id) {
                return Err(ContractError::InvalidPool("nft_token_id not found in pool".to_string()));
            }
            self.nft_token_ids.remove(nft_token_id);
        }
        Ok(())
    }

    pub fn can_buy_nfts(&self) -> bool {
        self.pool_type == PoolType::Trade || self.pool_type == PoolType::Token
    }

    pub fn can_sell_nfts(&self) -> bool {
        self.pool_type == PoolType::Trade || self.pool_type == PoolType::Nft
    }

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