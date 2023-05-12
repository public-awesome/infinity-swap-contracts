use crate::error::ContractError;
use crate::state::{BondingCurve, PoolConfig, PoolType, POOL_CONFIG};

use cosmwasm_std::{attr, ensure, Addr, Attribute, Decimal, Event, Storage, Uint128};

/// 100% represented as basis points
const MAX_BASIS_POINTS: u128 = 10000u128;

pub struct TokenPool(PoolConfig, Uint128);

impl Pool for TokenPool {
    fn new(pool_config: PoolConfig, total_tokens: Uint128) -> Self {
        TokenPool(pool_config, total_tokens)
    }

    fn inner(&self) -> (&PoolConfig, &Uint128) {
        (&self.0, &self.1)
    }

    fn inner_mut(&mut self) -> (&mut PoolConfig, &mut Uint128) {
        (&mut self.0, &mut self.1)
    }
}

impl EscrowToken for TokenPool {}

pub struct NftPool(PoolConfig, Uint128);

impl Pool for NftPool {
    fn new(pool_config: PoolConfig, total_tokens: Uint128) -> Self {
        NftPool(pool_config, total_tokens)
    }

    fn inner(&self) -> (&PoolConfig, &Uint128) {
        (&self.0, &self.1)
    }

    fn inner_mut(&mut self) -> (&mut PoolConfig, &mut Uint128) {
        (&mut self.0, &mut self.1)
    }
}

impl EscrowNft for NftPool {}

pub struct TradePool(PoolConfig, Uint128);

impl Pool for TradePool {
    fn new(pool_config: PoolConfig, total_tokens: Uint128) -> Self {
        TradePool(pool_config, total_tokens)
    }

    fn inner(&self) -> (&PoolConfig, &Uint128) {
        (&self.0, &self.1)
    }

    fn inner_mut(&mut self) -> (&mut PoolConfig, &mut Uint128) {
        (&mut self.0, &mut self.1)
    }
}

impl EscrowNft for TradePool {}

impl EscrowToken for TradePool {}

pub trait EscrowToken: Pool {}

pub trait EscrowNft: Pool {}

pub trait Pool {
    /// ---------------------------------------
    /// Implement
    /// ---------------------------------------

    fn new(pool_config: PoolConfig, total_tokens: Uint128) -> Self
    where
        Self: Sized;

    fn inner(&self) -> (&PoolConfig, &Uint128);

    fn inner_mut(&mut self) -> (&mut PoolConfig, &mut Uint128);

    /// ---------------------------------------
    /// Getters
    /// ---------------------------------------

    fn config(&self) -> &PoolConfig {
        self.inner().0
    }

    fn config_mut(&mut self) -> &mut PoolConfig {
        self.inner_mut().0
    }

    fn total_tokens(&self) -> &Uint128 {
        self.inner().1
    }

    fn owner(&self) -> &Addr {
        &self.config().owner
    }

    fn collection(&self) -> &Addr {
        &self.config().collection
    }

    fn is_active(&self) -> bool {
        self.config().is_active
    }

    /// Get the recipient of assets for trades performed on this pool
    fn pool_recipient(&self) -> &Addr {
        let config = self.config();
        match &config.asset_recipient {
            Some(addr) => addr,
            None => &config.owner,
        }
    }

    fn total_nfts(&self) -> u64 {
        self.config().total_nfts
    }

    fn finders_fee_percent(&self) -> Decimal {
        self.config().finders_fee_percent
    }

    fn reinvest_tokens(&self) -> bool {
        self.config().reinvest_tokens
    }

    fn reinvest_nfts(&self) -> bool {
        self.config().reinvest_nfts
    }

    /// Returns whether or not the pool can hold NFTs
    fn can_hold_nfts(&self) -> bool {
        let config = &self.config();
        config.pool_type == PoolType::Trade || config.pool_type == PoolType::Nft
    }

    /// Returns whether or not the pool can hold tokens
    fn can_hold_tokens(&self) -> bool {
        let config = &self.config();
        config.pool_type == PoolType::Trade || config.pool_type == PoolType::Token
    }

    /// ---------------------------------------
    /// Setters
    /// ---------------------------------------

    fn save(&mut self, storage: &mut dyn Storage) -> Result<(), ContractError> {
        self.force_property_values();
        self.validate()?;
        POOL_CONFIG.save(storage, &self.config())?;
        Ok(())
    }

    // Forces spot_price and delta to be correct for the constant product bonding curve
    fn force_property_values(&mut self) {
        let total_tokens = self.total_tokens().clone();
        let config = self.config_mut();
        if config.bonding_curve == BondingCurve::ConstantProduct {
            config.delta = Uint128::zero();
            if config.total_nfts == 0u64 {
                config.spot_price = Uint128::zero();
            } else {
                config.spot_price =
                    total_tokens.checked_div(Uint128::from(config.total_nfts)).unwrap();
            }
        };
    }

    fn set_asset_recipient(&mut self, asset_recipient: Addr) {
        self.inner_mut().0.asset_recipient = Some(asset_recipient);
    }

    fn set_spot_price(&mut self, spot_price: Uint128) {
        self.inner_mut().0.spot_price = spot_price;
    }

    fn set_delta(&mut self, delta: Uint128) {
        self.inner_mut().0.delta = delta;
    }

    fn set_swap_fee_percent(&mut self, swap_fee_bps: u64) {
        self.inner_mut().0.swap_fee_percent = Decimal::percent(swap_fee_bps);
    }

    fn set_finders_fee_percent(&mut self, finders_fee_bps: u64) {
        self.inner_mut().0.finders_fee_percent = Decimal::percent(finders_fee_bps);
    }

    fn set_is_active(&mut self, is_active: bool) {
        self.inner_mut().0.is_active = is_active;
    }

    fn set_reinvest_tokens(&mut self, reinvest_tokens: bool) {
        self.inner_mut().0.reinvest_tokens = reinvest_tokens;
    }

    fn set_reinvest_nfts(&mut self, reinvest_nfts: bool) {
        self.inner_mut().0.reinvest_nfts = reinvest_nfts;
    }

    fn set_total_tokens(&mut self, amount: Uint128) {
        *self.inner_mut().1 = amount;
    }

    fn set_total_nfts(&mut self, total_nfts: u64) {
        self.inner_mut().0.total_nfts = total_nfts;
    }

    /// ---------------------------------------

    /// Verify that the pool is valid by checking invariants before save
    fn validate(&self) -> Result<(), ContractError> {
        let config = &self.config();
        ensure!(
            !(config.bonding_curve == BondingCurve::Exponential
                && config.delta.u128() >= MAX_BASIS_POINTS),
            ContractError::InvalidPool(
                "delta cannot exceed max basis points on exponential curves".to_string(),
            )
        );

        match config.pool_type {
            PoolType::Token => {
                ensure!(
                    config.total_nfts == 0,
                    ContractError::InvalidPool(
                        "total_nfts must be zero for token pool".to_string(),
                    )
                );
                ensure!(
                    config.spot_price != Uint128::zero(),
                    ContractError::InvalidPool(
                        "spot_price must be non-zero for token pool".to_string(),
                    )
                );
                ensure!(
                    config.swap_fee_percent == Decimal::zero(),
                    ContractError::InvalidPool(
                        "swap_fee_percent must be 0 for token pool".to_string(),
                    )
                );
                ensure!(
                    config.bonding_curve != BondingCurve::ConstantProduct,
                    ContractError::InvalidPool(
                        "constant product bonding curve cannot be used with token pools"
                            .to_string(),
                    )
                );
                ensure!(
                    !config.reinvest_tokens,
                    ContractError::InvalidPool(
                        "cannot reinvest tokens on one sided pools".to_string(),
                    )
                );
                ensure!(
                    !config.reinvest_nfts,
                    ContractError::InvalidPool(
                        "cannot reinvest nfts on one sided pools".to_string(),
                    )
                );
            },
            PoolType::Nft => {
                ensure!(
                    config.spot_price != Uint128::zero(),
                    ContractError::InvalidPool(
                        "spot_price must be non-zero for nft pool".to_string(),
                    )
                );
                ensure!(
                    config.swap_fee_percent == Decimal::zero(),
                    ContractError::InvalidPool(
                        "swap_fee_percent must be 0 for nft pool".to_string(),
                    )
                );
                ensure!(
                    config.bonding_curve != BondingCurve::ConstantProduct,
                    ContractError::InvalidPool(
                        "constant product bonding curve cannot be used with nft pools".to_string(),
                    )
                );
                ensure!(
                    !config.reinvest_tokens,
                    ContractError::InvalidPool(
                        "cannot reinvest tokens on one sided pools".to_string(),
                    )
                );
                ensure!(
                    !config.reinvest_nfts,
                    ContractError::InvalidPool(
                        "cannot reinvest nfts on one sided pools".to_string(),
                    )
                );
            },
            PoolType::Trade => {},
        }

        Ok(())
    }

    /// ----------------------------
    /// Events
    /// ----------------------------

    /// Create an event with all the Pool properties
    fn create_event_all_props(&self, event_type: &str) -> Result<Event, ContractError> {
        self.create_event(
            event_type,
            vec![
                "collection",
                "owner",
                "asset_recipient",
                "pool_type",
                "bonding_curve",
                "spot_price",
                "delta",
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
    fn create_event(&self, event_type: &str, attr_keys: Vec<&str>) -> Result<Event, ContractError> {
        let config = &self.config();
        let mut attributes: Vec<Attribute> = vec![];
        for attr_keys in attr_keys {
            let attribute = match attr_keys {
                "collection" => attr("collection", config.collection.to_string()),
                "owner" => attr("owner", config.owner.to_string()),
                "asset_recipient" => attr(
                    "asset_recipient",
                    config
                        .asset_recipient
                        .as_ref()
                        .map_or("None".to_string(), |addr| addr.to_string()),
                ),
                "pool_type" => attr("pool_type", config.pool_type.to_string()),
                "bonding_curve" => attr("bonding_curve", config.bonding_curve.to_string()),
                "spot_price" => attr("spot_price", config.spot_price.to_string()),
                "delta" => attr("delta", config.delta.to_string()),
                "total_tokens" => attr("total_tokens", self.total_tokens().to_string()),
                "total_nfts" => attr("total_nfts", config.total_nfts.to_string()),
                "is_active" => attr("is_active", config.is_active.to_string()),
                "swap_fee_percent" => attr("swap_fee_percent", config.swap_fee_percent.to_string()),
                "finders_fee_percent" => {
                    attr("finders_fee_percent", config.finders_fee_percent.to_string())
                },
                "reinvest_tokens" => attr("reinvest_tokens", config.reinvest_tokens.to_string()),
                "reinvest_nfts" => attr("reinvest_nfts", config.reinvest_nfts.to_string()),
                _key => {
                    unreachable!("{} is not a valid attribute key", _key)
                },
            };
            attributes.push(attribute);
        }
        Ok(Event::new(event_type).add_attributes(attributes))
    }
}

// impl Pool {
//     /// Create a Pool object
//     pub fn new(config: PoolConfig) -> Self {
//         Self(config)
//     }

//     // Save a Pool object to storage
//     pub fn save(&self, storage: &mut dyn Storage) -> Result<(), ContractError> {
//         self.validate()?;
//         POOL_CONFIG.save(storage, &self.0)?;
//         Ok(())
//     }

//     /// ----------------------------
//     /// Getters
//     /// ----------------------------
//     pub fn owner(&self) -> &Addr {
//         &self.0.owner
//     }

//     pub fn collection(&self) -> &Addr {
//         &self.0.collection
//     }

//     pub fn is_active(&self) -> bool {
//         self.0.is_active
//     }

//     /// Get the recipient of assets for trades performed on this pool
//     pub fn recipient(&self) -> &Addr {
//         match &self.0.asset_recipient {
//             Some(addr) => addr,
//             None => &self.0.owner,
//         }
//     }

//     pub fn finders_fee_percent(&self) -> Decimal {
//         self.0.finders_fee_percent
//     }

//     pub fn reinvest_tokens(&self) -> bool {
//         self.0.reinvest_tokens
//     }

//     pub fn reinvest_nfts(&self) -> bool {
//         self.0.reinvest_nfts
//     }

//     /// ----------------------------
//     /// Setters
//     /// ----------------------------

//     /// Activate the pool so that it may begin accepting trades
//     pub fn set_is_active(&mut self, is_active: bool) {
//         self.0.is_active = is_active;
//     }

//     /// Verify that the pool is valid by checking invariants before save
//     pub fn validate(&self) -> Result<(), ContractError> {
//         ensure!(
//             !(self.0.bonding_curve == BondingCurve::Exponential
//                 && self.0.delta.u128() >= MAX_BASIS_POINTS),
//             ContractError::InvalidPool(
//                 "delta cannot exceed max basis points on exponential curves".to_string(),
//             )
//         );

//         match self.0.pool_type {
//             PoolType::Token => {
//                 ensure!(
//                     self.0.total_nfts == 0,
//                     ContractError::InvalidPool(
//                         "total_nfts must be zero for token pool".to_string(),
//                     )
//                 );
//                 ensure!(
//                     self.0.spot_price != Uint128::zero(),
//                     ContractError::InvalidPool(
//                         "spot_price must be non-zero for token pool".to_string(),
//                     )
//                 );
//                 ensure!(
//                     self.0.swap_fee_percent == Decimal::zero(),
//                     ContractError::InvalidPool(
//                         "swap_fee_percent must be 0 for token pool".to_string(),
//                     )
//                 );
//                 ensure!(
//                     self.0.bonding_curve != BondingCurve::ConstantProduct,
//                     ContractError::InvalidPool(
//                         "constant product bonding curve cannot be used with token pools"
//                             .to_string(),
//                     )
//                 );
//                 ensure!(
//                     !self.0.reinvest_tokens,
//                     ContractError::InvalidPool(
//                         "cannot reinvest tokens on one sided pools".to_string(),
//                     )
//                 );
//                 ensure!(
//                     !self.0.reinvest_nfts,
//                     ContractError::InvalidPool(
//                         "cannot reinvest nfts on one sided pools".to_string(),
//                     )
//                 );
//             },
//             PoolType::Nft => {
//                 ensure!(
//                     self.0.spot_price != Uint128::zero(),
//                     ContractError::InvalidPool(
//                         "spot_price must be non-zero for nft pool".to_string(),
//                     )
//                 );
//                 ensure!(
//                     self.0.swap_fee_percent == Decimal::zero(),
//                     ContractError::InvalidPool(
//                         "swap_fee_percent must be 0 for nft pool".to_string(),
//                     )
//                 );
//                 ensure!(
//                     self.0.bonding_curve != BondingCurve::ConstantProduct,
//                     ContractError::InvalidPool(
//                         "constant product bonding curve cannot be used with nft pools".to_string(),
//                     )
//                 );
//                 ensure!(
//                     !self.0.reinvest_tokens,
//                     ContractError::InvalidPool(
//                         "cannot reinvest tokens on one sided pools".to_string(),
//                     )
//                 );
//                 ensure!(
//                     !self.0.reinvest_nfts,
//                     ContractError::InvalidPool(
//                         "cannot reinvest nfts on one sided pools".to_string(),
//                     )
//                 );
//             },
//             PoolType::Trade => {},
//         }

//         Ok(())
//     }

//     // /// Deposit nfts into the pool
//     // pub fn deposit_nfts(&mut self, nft_token_ids: &Vec<String>) -> Result<(), ContractError> {
//     //     if self.pool_type == PoolType::Token {
//     //         return Err(ContractError::InvalidPool(
//     //             "cannot deposit nfts into token pool".to_string(),
//     //         ));
//     //     }
//     //     self.total_nfts += nft_token_ids.len() as u64;
//     //     Ok(())
//     // }

//     // /// Withdraw tokens from the pool
//     // pub fn withdraw_tokens(&mut self, amount: Uint128) -> Result<(), ContractError> {
//     //     if self.pool_type == PoolType::Nft {
//     //         return Err(ContractError::InvalidPool(
//     //             "cannot withdraw tokens from nft pool".to_string(),
//     //         ));
//     //     }
//     //     if self.total_tokens < amount {
//     //         return Err(ContractError::Std(StdError::overflow(OverflowError {
//     //             operation: cosmwasm_std::OverflowOperation::Sub,
//     //             operand1: self.total_tokens.to_string(),
//     //             operand2: amount.to_string(),
//     //         })));
//     //     }
//     //     self.total_tokens -= amount;
//     //     Ok(())
//     // }

//     // /// Withdraw nfts from the pool
//     // pub fn withdraw_nfts(&mut self, nft_token_ids: &Vec<String>) -> Result<(), ContractError> {
//     //     if self.pool_type == PoolType::Token {
//     //         return Err(ContractError::InvalidPool(
//     //             "cannot withdraw nfts from token pool".to_string(),
//     //         ));
//     //     }
//     //     if self.total_nfts < nft_token_ids.len() as u64 {
//     //         return Err(ContractError::InternalError(
//     //             "pool NFT overdraw".to_string(),
//     //         ));
//     //     }
//     //     self.total_nfts -= nft_token_ids.len() as u64;
//     //     Ok(())
//     // }

//     /// Returns whether or not the pool can buy NFTs
//     pub fn can_buy_nfts(&self) -> bool {
//         self.0.pool_type == PoolType::Trade || self.0.pool_type == PoolType::Token
//     }

//     /// Returns whether or not the pool can sell NFTs
//     pub fn can_sell_nfts(&self) -> bool {
//         self.0.pool_type == PoolType::Trade || self.0.pool_type == PoolType::Nft
//     }

//     // /// Returns the price at which this pool will sell NFTs
//     // /// Note: the buy_from_pool_quote is indexed by PoolQuote for future discovery
//     // pub fn get_buy_from_pool_quote(
//     //     &self,
//     //     min_quote: Uint128,
//     // ) -> Result<Option<Uint128>, ContractError> {
//     //     // Calculate the buy price with respect to pool types and bonding curves
//     //     let buy_price = match self.pool_type {
//     //         PoolType::Token => Err(ContractError::InvalidPool(
//     //             "pool cannot sell nfts".to_string(),
//     //         )),
//     //         PoolType::Nft => Ok(self.spot_price),
//     //         PoolType::Trade => match self.bonding_curve {
//     //             BondingCurve::Linear => self
//     //                 .spot_price
//     //                 .checked_add(self.delta)
//     //                 .map_err(|e| ContractError::Std(StdError::overflow(e))),
//     //             BondingCurve::Exponential => {
//     //                 let net_delta = Uint128::from(MAX_BASIS_POINTS)
//     //                     .checked_add(self.delta)
//     //                     .map_err(|e| ContractError::Std(StdError::overflow(e)))?;
//     //                 self.spot_price
//     //                     .checked_mul(net_delta)
//     //                     .map_err(|e| ContractError::Std(StdError::overflow(e)))?
//     //                     .checked_div(Uint128::from(MAX_BASIS_POINTS))
//     //                     .map_err(|e| ContractError::Std(StdError::divide_by_zero(e)))?
//     //                     .checked_add(Uint128::one())
//     //                     .map_err(|e| ContractError::Std(StdError::overflow(e)))
//     //             }
//     //             BondingCurve::ConstantProduct => {
//     //                 if self.total_nfts <= 1 {
//     //                     return Ok(None);
//     //                 }
//     //                 let buy_price = self
//     //                     .total_tokens
//     //                     .checked_div(Uint128::from(self.total_nfts - 1))
//     //                     .map_err(|e| ContractError::Std(StdError::divide_by_zero(e)))?
//     //                     .checked_add(Uint128::one())
//     //                     .map_err(|e| ContractError::Std(StdError::overflow(e)))?;
//     //                 Ok(buy_price)
//     //             }
//     //         },
//     //     }?;
//     //     // If the pool has no NFTs to sell, or quote is below min, return None
//     //     if self.total_nfts == 0 || buy_price < min_quote {
//     //         return Ok(None);
//     //     }
//     //     Ok(Some(buy_price))
//     // }

//     /// Returns the price at which this pool will buy NFTs
//     pub fn get_sell_to_pool_quote(
//         &self,
//         total_tokens: Uint128,
//         min_price: Uint128,
//     ) -> Result<Uint128, ContractError> {
//         let sell_quote = match self.0.bonding_curve {
//             BondingCurve::Linear | BondingCurve::Exponential => self.0.spot_price,
//             BondingCurve::ConstantProduct => {
//                 total_tokens.checked_div(Uint128::from(self.0.total_nfts + 1)).unwrap()
//             },
//         };
//         ensure!(
//             sell_quote >= min_price,
//             ContractError::InvalidPoolQuote("sale price is below min price".to_string(),)
//         );
//         ensure!(
//             sell_quote <= total_tokens,
//             ContractError::InvalidPoolQuote("pool has insufficient tokens".to_string(),)
//         );
//         Ok(sell_quote)
//     }

//     // /// Buy an NFT from the pool
//     // pub fn buy_nft_from_pool(
//     //     &mut self,
//     //     nft_swap: &NftSwap,
//     //     sale_price: Uint128,
//     // ) -> Result<(), ContractError> {
//     //     if !self.can_sell_nfts() {
//     //         return Err(ContractError::InvalidPool(
//     //             "pool cannot sell nfts".to_string(),
//     //         ));
//     //     }
//     //     if !self.is_active {
//     //         return Err(ContractError::InvalidPool("pool is not active".to_string()));
//     //     }

//     //     // If sale price exceeds the max expected, return an error
//     //     if sale_price > nft_swap.token_amount {
//     //         return Err(ContractError::SwapError(
//     //             "pool sale price is above max expected".to_string(),
//     //         ));
//     //     }

//     //     // Decrement total_nfts on pool
//     //     if self.total_nfts == 0 {
//     //         return Err(ContractError::SwapError(
//     //             "pool does not own any NFTS".to_string(),
//     //         ));
//     //     } else {
//     //         self.total_nfts -= 1;
//     //     }

//     //     Ok(())
//     // }

//     /// Updates the spot price of the pool depending on the transaction type
//     pub fn update_spot_price_after_buy(&mut self, total_tokens: Uint128) -> Result<(), StdError> {
//         let result = match self.0.bonding_curve {
//             BondingCurve::Linear => {
//                 self.0.spot_price.checked_sub(self.0.delta).map_err(StdError::overflow)
//             },
//             BondingCurve::Exponential => {
//                 let denominator = Uint128::from(MAX_BASIS_POINTS)
//                     .checked_add(self.0.delta)
//                     .map_err(StdError::overflow)?;
//                 self.0
//                     .spot_price
//                     .checked_mul(Uint128::from(MAX_BASIS_POINTS))
//                     .map_err(StdError::overflow)?
//                     .checked_div(denominator)
//                     .map_err(StdError::divide_by_zero)
//             },
//             BondingCurve::ConstantProduct => total_tokens
//                 .checked_div(Uint128::from(self.0.total_nfts))
//                 .map_err(StdError::divide_by_zero),
//         };
//         match result {
//             Ok(new_spot_price) => {
//                 self.0.spot_price = new_spot_price;
//                 Ok(())
//             },
//             Err(e) => {
//                 self.0.is_active = false;
//                 Err(e)
//             },
//         }
//     }
// }
