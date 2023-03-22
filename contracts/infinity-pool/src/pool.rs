use crate::msg::{NftSwap, TransactionType};
use crate::state::{BondingCurve, Pool, PoolType};
use crate::ContractError;
use cosmwasm_std::{attr, Addr, Attribute, Decimal, Event, OverflowError, StdError, Uint128};
use sg_marketplace::msg::ParamsResponse;

/// 100% represented as basis points
const MAX_BASIS_POINTS: u128 = 10000u128;
/// Maximum swap fee percent that can be set on trade pools
const MAX_SWAP_FEE_PERCENT: u64 = 5000u64;

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
            total_nfts: 0u64,
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
                "finders_fee_percent is above marketplace max_finders_fee_percent".to_string(),
            ));
        }
        if self.bonding_curve == BondingCurve::Exponential && self.delta.u128() > MAX_BASIS_POINTS {
            return Err(ContractError::InvalidPool(
                "delta cannot exceed max basis points on exponential curves".to_string(),
            ));
        }

        match &self.pool_type {
            PoolType::Token => {
                if self.total_nfts != 0 {
                    return Err(ContractError::InvalidPool(
                        "total_nfts must be zero for token pool".to_string(),
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
                if self.swap_fee_percent > Decimal::percent(MAX_SWAP_FEE_PERCENT) {
                    return Err(ContractError::InvalidPool(
                        "swap_fee_percent cannot be greater than 50%".to_string(),
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
        self.total_nfts += nft_token_ids.len() as u64;
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
            return Err(ContractError::Std(StdError::overflow(OverflowError {
                operation: cosmwasm_std::OverflowOperation::Sub,
                operand1: self.total_tokens.to_string(),
                operand2: amount.to_string(),
            })));
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
        if self.total_nfts < nft_token_ids.len() as u64 {
            return Err(ContractError::InternalError(
                "pool NFT overdraw".to_string(),
            ));
        }
        self.total_nfts -= nft_token_ids.len() as u64;
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
    pub fn get_buy_quote(&self, min_quote: Uint128) -> Result<Option<Uint128>, ContractError> {
        // Calculate the buy price with respect to pool types and bonding curves
        let buy_price = match self.pool_type {
            PoolType::Token => Ok(self.spot_price),
            PoolType::Nft => Err(ContractError::InvalidPool(
                "pool cannot buy nfts".to_string(),
            )),
            PoolType::Trade => match self.bonding_curve {
                BondingCurve::Linear => self
                    .spot_price
                    .checked_add(self.delta)
                    .map_err(|e| ContractError::Std(StdError::overflow(e))),
                BondingCurve::Exponential => {
                    let product = self
                        .spot_price
                        .checked_mul(self.delta)
                        .map_err(|e| StdError::Overflow { source: e })?
                        .checked_div(Uint128::from(MAX_BASIS_POINTS))
                        .map_err(|e| ContractError::Std(StdError::divide_by_zero(e)))?;
                    self.spot_price
                        .checked_add(product)
                        .map_err(|e| ContractError::Std(StdError::overflow(e)))
                }
                BondingCurve::ConstantProduct => self
                    .total_tokens
                    .checked_div(Uint128::from(self.total_nfts + 1))
                    .map_err(|e| ContractError::Std(StdError::divide_by_zero(e))),
            },
        }?;
        // If the pool has insufficient tokens to buy the NFT, return None
        if self.total_tokens < buy_price || buy_price < min_quote {
            return Ok(None);
        }
        Ok(Some(buy_price))
    }

    /// Returns the price at which this pool will sell NFTs
    /// Note: the sell quote is indexed by PoolQuote for future discovery
    pub fn get_sell_quote(&self, min_quote: Uint128) -> Result<Option<Uint128>, ContractError> {
        if !self.can_sell_nfts() {
            return Err(ContractError::InvalidPool(
                "pool cannot sell nfts".to_string(),
            ));
        }
        // If the pool has no NFTs to sell, return None
        if self.total_nfts == 0 {
            return Ok(None);
        }
        let sell_price = match self.bonding_curve {
            BondingCurve::Linear | BondingCurve::Exponential => self.spot_price,
            BondingCurve::ConstantProduct => {
                if self.total_nfts < 2 {
                    return Ok(None);
                }
                self.total_tokens
                    .checked_div(Uint128::from(self.total_nfts - 1))
                    .unwrap()
            }
        };
        if sell_price < min_quote {
            return Ok(None);
        }
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
                "pool cannot sell nfts".to_string(),
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

        // Decrement total_nfts on pool
        if self.total_nfts == 0 {
            return Err(ContractError::SwapError(
                "pool does not own any NFTS".to_string(),
            ));
        } else {
            self.total_nfts -= 1;
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
                "pool cannot buy nfts".to_string(),
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
        let result = match tx_type {
            TransactionType::TokensForNfts => match self.bonding_curve {
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
                    .checked_div(Uint128::from(self.total_nfts))
                    .map_err(|e| StdError::DivideByZero { source: e }),
            },
            TransactionType::NftsForTokens => match self.bonding_curve {
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
                    .checked_div(Uint128::from(self.total_nfts))
                    .map_err(|e| StdError::DivideByZero { source: e }),
            },
        };
        match result {
            Ok(_spot_price) => {
                self.spot_price = _spot_price;
                Ok(())
            }
            Err(_err) => {
                self.is_active = false;
                Err(_err)
            }
        }
    }

    /// Create an event with all the Pool properties
    pub fn create_event_all_props(&self, event_type: &str) -> Result<Event, ContractError> {
        self.create_event(
            event_type,
            vec![
                "id",
                "collection",
                "owner",
                "asset_recipient",
                "pool_type",
                "bonding_curve",
                "spot_price",
                "delta",
                "total_tokens",
                "total_nfts",
                "is_active",
                "swap_fee_percent",
                "finders_fee_percent",
                "reinvest_tokens",
                "reinvest_nfts",
            ],
        )
    }

    /// Create an event with certain Pool properties
    pub fn create_event(
        &self,
        event_type: &str,
        attr_keys: Vec<&str>,
    ) -> Result<Event, ContractError> {
        let mut attributes: Vec<Attribute> = vec![];
        for attr_keys in attr_keys {
            let attribute = match attr_keys {
                "id" => attr("id", self.id.to_string()),
                "collection" => attr("collection", self.collection.to_string()),
                "owner" => attr("owner", self.owner.to_string()),
                "asset_recipient" => attr(
                    "asset_recipient",
                    self.asset_recipient
                        .as_ref()
                        .map_or("None".to_string(), |addr| addr.to_string()),
                ),
                "pool_type" => attr("pool_type", self.pool_type.to_string()),
                "bonding_curve" => attr("bonding_curve", self.bonding_curve.to_string()),
                "spot_price" => attr("spot_price", self.spot_price.to_string()),
                "delta" => attr("delta", self.delta.to_string()),
                "total_tokens" => attr("total_tokens", self.total_tokens.to_string()),
                "total_nfts" => attr("total_nfts", self.total_nfts.to_string()),
                "is_active" => attr("is_active", self.is_active.to_string()),
                "swap_fee_percent" => attr("swap_fee_percent", self.swap_fee_percent.to_string()),
                "finders_fee_percent" => {
                    attr("finders_fee_percent", self.finders_fee_percent.to_string())
                }
                "reinvest_tokens" => attr("reinvest_tokens", self.reinvest_tokens.to_string()),
                "reinvest_nfts" => attr("reinvest_nfts", self.reinvest_nfts.to_string()),
                _key => {
                    return Err(ContractError::InvalidPropertyKeyError(format!(
                        "Invalid property key: {}",
                        _key
                    )));
                }
            };
            attributes.push(attribute);
        }
        Ok(Event::new(event_type).add_attributes(attributes))
    }
}
