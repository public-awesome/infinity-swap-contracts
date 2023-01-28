use crate::msg::NftSwap;
use crate::state::{BondingCurve, Pool, PoolType};
use crate::ContractError;
use core::cmp::Ordering;
use cosmwasm_std::{Addr, Uint128};
use std::collections::BTreeSet;

const MAX_BASIS_POINTS: u128 = 10000u128;

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
        fee_bps: Option<u16>,
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
        match &self.pool_type {
            PoolType::Token => {
                if self.nft_token_ids.len() > 0 {
                    return Err(ContractError::InvalidPool(
                        "nft_token_ids must be empty for token pool".to_string(),
                    ));
                }
                if self.spot_price == Uint128::zero() {
                    return Err(ContractError::InvalidPool(
                        "spot_price must be non-zero for token pool".to_string(),
                    ));
                }
                if let Some(_) = self.fee_bps {
                    return Err(ContractError::InvalidPool(
                        "fee_bps must be 0 for token pool".to_string(),
                    ));
                }
                if self.bonding_curve == BondingCurve::ConstantProduct {
                    return Err(ContractError::InvalidPool(
                        "constant product bonding curve cannot be used with token pools"
                            .to_string(),
                    ));
                }
            }
            PoolType::Nft => {
                if self.total_tokens > Uint128::zero() {
                    return Err(ContractError::InvalidPool(
                        "total_tokens must be zero for nft pool".to_string(),
                    ));
                }
                if self.spot_price == Uint128::zero() {
                    return Err(ContractError::InvalidPool(
                        "spot_price must be non-zero for nft pool".to_string(),
                    ));
                }
                if let Some(_) = self.fee_bps {
                    return Err(ContractError::InvalidPool(
                        "fee_bps must be 0 for nft pool".to_string(),
                    ));
                }
                if self.bonding_curve == BondingCurve::ConstantProduct {
                    return Err(ContractError::InvalidPool(
                        "constant product bonding curve cannot be used with nft pools".to_string(),
                    ));
                }
            }
            PoolType::Trade => {
                if let Some(_fee) = self.fee_bps {
                    if _fee > 9000 {
                        return Err(ContractError::InvalidPool(
                            "fee_bps is greater than 9000".to_string(),
                        ));
                    }
                }
                if self.is_active {
                    if self.total_tokens == Uint128::zero() {
                        return Err(ContractError::InvalidPool(
                            "total_tokens must be greater than zero for trade pool".to_string(),
                        ));
                    }
                    if self.nft_token_ids.len() == 0 {
                        return Err(ContractError::InvalidPool(
                            "nft_token_ids must be non-empty for trade pool".to_string(),
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_recipient(&self) -> Addr {
        match &self.asset_recipient {
            Some(addr) => addr.clone(),
            None => self.owner.clone(),
        }
    }

    pub fn set_active(&mut self, is_active: bool) -> Result<(), ContractError> {
        self.is_active = is_active;
        Ok(())
    }

    pub fn deposit_tokens(&mut self, amount: Uint128) -> Result<(), ContractError> {
        if self.pool_type == PoolType::Nft {
            return Err(ContractError::InvalidPool(
                "cannot deposit tokens into nft pool".to_string(),
            ));
        }
        self.total_tokens += amount;
        Ok(())
    }

    pub fn deposit_nfts(&mut self, nft_token_ids: &Vec<String>) -> Result<(), ContractError> {
        if self.pool_type == PoolType::Token {
            return Err(ContractError::InvalidPool(
                "cannot deposit nfts into token pool".to_string(),
            ));
        }
        for nft_token_id in nft_token_ids {
            self.nft_token_ids.insert(nft_token_id.clone());
        }
        Ok(())
    }

    pub fn withdraw_tokens(&mut self, amount: Uint128) -> Result<(), ContractError> {
        if self.pool_type == PoolType::Nft {
            return Err(ContractError::InvalidPool(
                "cannot withdraw tokens from nft pool".to_string(),
            ));
        }
        if self.total_tokens < amount {
            return Err(ContractError::InvalidPool(
                "insufficient tokens in pool".to_string(),
            ));
        }
        self.total_tokens -= amount;
        Ok(())
    }

    pub fn withdraw_nfts(&mut self, nft_token_ids: &Vec<String>) -> Result<(), ContractError> {
        if self.pool_type == PoolType::Token {
            return Err(ContractError::InvalidPool(
                "cannot withdraw nfts from token pool".to_string(),
            ));
        }
        for nft_token_id in nft_token_ids {
            if !self.nft_token_ids.remove(nft_token_id) {
                return Err(ContractError::InvalidPool(
                    "nft_token_id not found in pool".to_string(),
                ));
            }
        }
        Ok(())
    }

    pub fn can_buy_nfts(&self) -> bool {
        self.pool_type == PoolType::Trade || self.pool_type == PoolType::Token
    }

    pub fn can_sell_nfts(&self) -> bool {
        self.pool_type == PoolType::Trade || self.pool_type == PoolType::Nft
    }

    pub fn get_buy_quote(&self) -> Result<Option<Uint128>, ContractError> {
        let buy_price = match self.pool_type {
            PoolType::Token => Ok(self.spot_price),
            PoolType::Nft => Err(ContractError::InvalidPool(
                "pool cannot buy nfts".to_string(),
            )),
            PoolType::Trade => match self.bonding_curve {
                BondingCurve::Linear => Ok(self.spot_price + self.delta),
                BondingCurve::Exponential => Ok(self.spot_price
                    * (Uint128::from(MAX_BASIS_POINTS) + self.delta)
                    / Uint128::from(MAX_BASIS_POINTS)),
                BondingCurve::ConstantProduct => {
                    Ok(self.total_tokens / Uint128::from(self.nft_token_ids.len() as u128 - 1))
                }
            },
        }?;
        if self.total_tokens < buy_price {
            return Ok(None);
        }
        return Ok(Some(buy_price));
    }

    pub fn get_sell_quote(&self) -> Result<Option<Uint128>, ContractError> {
        if !self.can_sell_nfts() {
            return Err(ContractError::InvalidPool(
                "pool cannot sell nfts".to_string(),
            ));
        }
        if self.nft_token_ids.len() == 0 {
            return Ok(None);
        }
        let sell_price = match self.bonding_curve {
            BondingCurve::Linear | BondingCurve::Exponential => self.spot_price,
            BondingCurve::ConstantProduct => {
                self.total_tokens / Uint128::from(self.nft_token_ids.len() as u128 + 1)
            }
        };
        return Ok(Some(sell_price));
    }

    pub fn buy_nft_from_pool(&mut self, nft_swap: &NftSwap) -> Result<Uint128, ContractError> {
        if !self.can_sell_nfts() {
            return Err(ContractError::InvalidPool(
                "pool does not sell NFTs".to_string(),
            ));
        }
        if !self.is_active {
            return Err(ContractError::InvalidPool("pool is not active".to_string()));
        }
        // if self.id != nft_bid.pool_id {
        //     return Err(ContractError::InvalidPool("incorrect pool".to_string()));
        // }
        let sell_quote = self.get_sell_quote()?;

        let sale_price = sell_quote.ok_or(ContractError::SwapError(
            "pool cannot offer quote".to_string(),
        ))?;

        if sale_price > nft_swap.token_amount {
            return Err(ContractError::SwapError(
                "pool sale price is above max expected".to_string(),
            ));
        }

        if !self.nft_token_ids.remove(&nft_swap.nft_token_id) {
            return Err(ContractError::SwapError(
                "pool does not own NFT".to_string(),
            ));
        }

        self.spot_price = match self.bonding_curve {
            BondingCurve::Linear => self.spot_price + self.delta,
            BondingCurve::Exponential => {
                self.spot_price * (Uint128::from(MAX_BASIS_POINTS) + self.delta)
                    / Uint128::from(MAX_BASIS_POINTS)
            }
            BondingCurve::ConstantProduct => {
                self.total_tokens / Uint128::from(self.nft_token_ids.len() as u128)
            }
        };

        Ok(sale_price)
    }

    pub fn sell_nft_to_pool(&mut self, nft_swap: &NftSwap) -> Result<Uint128, ContractError> {
        if !self.can_buy_nfts() {
            return Err(ContractError::InvalidPool(
                "pool does not buy NFTs".to_string(),
            ));
        }
        if !self.is_active {
            return Err(ContractError::InvalidPool("pool is not active".to_string()));
        }
        let buy_quote = self.get_buy_quote()?;

        let sale_price = buy_quote.ok_or(ContractError::SwapError(
            "pool cannot offer quote".to_string(),
        ))?;

        if sale_price < nft_swap.token_amount {
            return Err(ContractError::SwapError(
                "pool sale price is below min expected".to_string(),
            ));
        }

        self.total_tokens -= sale_price;

        self.spot_price = match self.bonding_curve {
            BondingCurve::Linear => self.spot_price - self.delta,
            BondingCurve::Exponential => {
                self.spot_price * (Uint128::from(MAX_BASIS_POINTS) - self.delta)
                    / Uint128::from(MAX_BASIS_POINTS)
            }
            BondingCurve::ConstantProduct => {
                self.total_tokens / Uint128::from(self.nft_token_ids.len() as u128)
            }
        };

        Ok(sale_price)
    }
}
