use crate::msg::{NftSwap, TransactionType};
use crate::state::{BondingCurve, Pool, PoolType};
use crate::ContractError;
use cosmwasm_std::{Addr, Decimal, StdError, Uint128};
use sg_marketplace::msg::ParamsResponse;
use std::collections::BTreeSet;

/// 100% represented as basis points
const MAX_BASIS_POINTS: u128 = 10000u128;

impl Pool {
    /// Create a Pool object
    pub fn new(
        id: u64,
        collection: Addr,
        owner: Addr,
        asset_recipient: Option<Addr>,
        pool_type: PoolType,
        bonding_curve: BondingCurve,
        spot_price: Uint128,
        delta: Uint128,
        finders_fee_percent: Decimal,
        swap_fee_percent: Decimal,
        reinvest_tokens: bool,
        reinvest_nfts: bool,
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
            finders_fee_percent,
            swap_fee_percent,
            is_active: false,
            reinvest_tokens,
            reinvest_nfts,
        }
    }

    /// Verify that the pool is valid by checking invariants before save
    pub fn validate(&self, marketplace_params: &ParamsResponse) -> Result<(), ContractError> {
        if self.finders_fee_percent > marketplace_params.params.max_finders_fee_percent {
            return Err(ContractError::InvalidPool(
                "finders_fee_percent is above max_finders_fee_percent".to_string(),
            ));
        }
        if self.bonding_curve == BondingCurve::Exponential && self.delta.u128() > MAX_BASIS_POINTS {
            return Err(ContractError::InvalidPool(
                "delta cannot exceed max basis points on exponential curves".to_string(),
            ));
        }

        match &self.pool_type {
            PoolType::Token => {
                if !self.nft_token_ids.is_empty() {
                    return Err(ContractError::InvalidPool(
                        "nft_token_ids must be empty for token pool".to_string(),
                    ));
                }
                if self.spot_price == Uint128::zero() {
                    return Err(ContractError::InvalidPool(
                        "spot_price must be non-zero for token pool".to_string(),
                    ));
                }
                if self.swap_fee_percent > Decimal::zero() {
                    return Err(ContractError::InvalidPool(
                        "swap_fee_percent must be 0 for token pool".to_string(),
                    ));
                }
                if self.bonding_curve == BondingCurve::ConstantProduct {
                    return Err(ContractError::InvalidPool(
                        "constant product bonding curve cannot be used with token pools"
                            .to_string(),
                    ));
                }
                if self.reinvest_tokens {
                    return Err(ContractError::InvalidPool(
                        "cannot reinvest buy side on one sided pools".to_string(),
                    ));
                }
                if self.reinvest_nfts {
                    return Err(ContractError::InvalidPool(
                        "cannot reinvest sell side on one sided pools".to_string(),
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
                if self.swap_fee_percent > Decimal::zero() {
                    return Err(ContractError::InvalidPool(
                        "swap_fee_percent must be 0 for nft pool".to_string(),
                    ));
                }
                if self.bonding_curve == BondingCurve::ConstantProduct {
                    return Err(ContractError::InvalidPool(
                        "constant product bonding curve cannot be used with nft pools".to_string(),
                    ));
                }
                if self.reinvest_tokens {
                    return Err(ContractError::InvalidPool(
                        "cannot reinvest buy side on one sided pools".to_string(),
                    ));
                }
                if self.reinvest_nfts {
                    return Err(ContractError::InvalidPool(
                        "cannot reinvest sell side on one sided pools".to_string(),
                    ));
                }
            }
            PoolType::Trade => {
                if self.swap_fee_percent > Decimal::percent(9000u64) {
                    return Err(ContractError::InvalidPool(
                        "swap_fee_percent is greater than 90%".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Get the recipient of assets for trades performed on this pool
    pub fn get_recipient(&self) -> Addr {
        match &self.asset_recipient {
            Some(addr) => addr.clone(),
            None => self.owner.clone(),
        }
    }

    /// Activate the pool so that it may begin accepting trades
    pub fn set_active(&mut self, is_active: bool) -> Result<(), ContractError> {
        self.is_active = is_active;
        Ok(())
    }

    /// Deposit tokens into the pool
    pub fn deposit_tokens(&mut self, amount: Uint128) -> Result<(), ContractError> {
        if self.pool_type == PoolType::Nft {
            return Err(ContractError::InvalidPool(
                "cannot deposit tokens into nft pool".to_string(),
            ));
        }
        self.total_tokens += amount;
        Ok(())
    }

    /// Deposit nfts into the pool
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

    /// Withdraw tokens from the pool
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

    /// Withdraw nfts from the pool
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

    /// Returns whether or not the pool can buy NFTs
    pub fn can_buy_nfts(&self) -> bool {
        self.pool_type == PoolType::Trade || self.pool_type == PoolType::Token
    }

    /// Returns whether or not the pool can sell NFTs
    pub fn can_sell_nfts(&self) -> bool {
        self.pool_type == PoolType::Trade || self.pool_type == PoolType::Nft
    }

    /// Returns the price at which this pool will buy NFTs
    /// Note: the buy quote is indexed by PoolQuote for future discovery
    pub fn get_buy_quote(&self) -> Result<Option<Uint128>, ContractError> {
        // Calculate the buy price with respect to pool types and bonding curves
        let buy_price = match self.pool_type {
            PoolType::Token => Ok(self.spot_price),
            PoolType::Nft => Err(ContractError::InvalidPool(
                "pool cannot buy nfts".to_string(),
            )),
            PoolType::Trade => match self.bonding_curve {
                BondingCurve::Linear => Ok(self.spot_price + self.delta),
                BondingCurve::Exponential => Ok(self.spot_price
                    * Decimal::percent((MAX_BASIS_POINTS + self.delta.u128()) as u64)),
                BondingCurve::ConstantProduct => {
                    Ok(self.total_tokens / Uint128::from(self.nft_token_ids.len() as u128 - 1))
                }
            },
        }?;

        // If the pool has insufficient tokens to buy the NFT, return None
        if self.total_tokens < buy_price {
            return Ok(None);
        }
        Ok(Some(buy_price))
    }

    /// Returns the price at which this pool will sell NFTs
    /// Note: the sell quote is indexed by PoolQuote for future discovery
    pub fn get_sell_quote(&self) -> Result<Option<Uint128>, ContractError> {
        if !self.can_sell_nfts() {
            return Err(ContractError::InvalidPool(
                "pool cannot sell nfts".to_string(),
            ));
        }
        // If the pool has no NFTs to sell, return None
        if self.nft_token_ids.is_empty() {
            return Ok(None);
        }
        let sell_price = match self.bonding_curve {
            BondingCurve::Linear | BondingCurve::Exponential => self.spot_price,
            BondingCurve::ConstantProduct => {
                self.total_tokens / Uint128::from(self.nft_token_ids.len() as u128 + 1)
            }
        };
        Ok(Some(sell_price))
    }

    /// Buy an NFT from the pool
    pub fn buy_nft_from_pool(
        &mut self,
        nft_swap: &NftSwap,
        sale_price: Uint128,
    ) -> Result<(), ContractError> {
        if !self.can_sell_nfts() {
            return Err(ContractError::InvalidPool(
                "pool does not sell NFTs".to_string(),
            ));
        }
        if !self.is_active {
            return Err(ContractError::InvalidPool("pool is not active".to_string()));
        }

        // If sale price exceeds the max expected, return an error
        if sale_price > nft_swap.token_amount {
            return Err(ContractError::SwapError(
                "pool sale price is above max expected".to_string(),
            ));
        }

        // Remove the nft_token_id from the pool
        // Also, if pool does not own the NFT, return an error
        if !self.nft_token_ids.remove(&nft_swap.nft_token_id) {
            return Err(ContractError::SwapError(
                "pool does not own NFT".to_string(),
            ));
        }

        Ok(())
    }

    /// Sell an NFT to the pool
    pub fn sell_nft_to_pool(
        &mut self,
        nft_swap: &NftSwap,
        sale_price: Uint128,
    ) -> Result<(), ContractError> {
        if !self.can_buy_nfts() {
            return Err(ContractError::InvalidPool(
                "pool does not buy NFTs".to_string(),
            ));
        }
        if !self.is_active {
            return Err(ContractError::InvalidPool("pool is not active".to_string()));
        }

        // If sale price is below the min expected, return an error
        if sale_price < nft_swap.token_amount {
            return Err(ContractError::SwapError(
                "pool sale price is below min expected".to_string(),
            ));
        }

        // Deduct the sale price from the pool's token balance
        self.total_tokens -= sale_price;

        Ok(())
    }

    /// Updates the spot price of the pool depending on the transaction type
    pub fn update_spot_price(&mut self, tx_type: &TransactionType) -> Result<(), StdError> {
        self.spot_price = match tx_type {
            TransactionType::Buy => match self.bonding_curve {
                BondingCurve::Linear => self
                    .spot_price
                    .checked_add(self.delta)
                    .map_err(|e| StdError::Overflow { source: e }),
                BondingCurve::Exponential => {
                    let product = self
                        .spot_price
                        .checked_mul(self.delta)
                        .map_err(|e| StdError::Overflow { source: e })?
                        .checked_div(Uint128::from(MAX_BASIS_POINTS))
                        .map_err(|e| StdError::DivideByZero { source: e })?;
                    self.spot_price
                        .checked_add(product)
                        .map_err(|e| StdError::Overflow { source: e })
                }
                BondingCurve::ConstantProduct => self
                    .total_tokens
                    .checked_div(Uint128::from(self.nft_token_ids.len() as u64))
                    .map_err(|e| StdError::DivideByZero { source: e }),
            },
            TransactionType::Sell => match self.bonding_curve {
                BondingCurve::Linear => self
                    .spot_price
                    .checked_sub(self.delta)
                    .map_err(|e| StdError::Overflow { source: e }),
                BondingCurve::Exponential => {
                    let product = self
                        .spot_price
                        .checked_mul(self.delta)
                        .map_err(|e| StdError::Overflow { source: e })?
                        .checked_div(Uint128::from(MAX_BASIS_POINTS))
                        .map_err(|e| StdError::DivideByZero { source: e })?;
                    self.spot_price
                        .checked_sub(product)
                        .map_err(|e| StdError::Overflow { source: e })
                }
                BondingCurve::ConstantProduct => self
                    .total_tokens
                    .checked_div(Uint128::from(self.nft_token_ids.len() as u64))
                    .map_err(|e| StdError::DivideByZero { source: e }),
            },
        }?;
        Ok(())
    }
}
