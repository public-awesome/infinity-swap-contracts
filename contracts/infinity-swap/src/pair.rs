use crate::msg::{NftSwap, PairOptions};
use crate::state::{pairs, BondingCurve, Pair, PairType};
use crate::ContractError;

use cosmwasm_std::{
    attr, ensure, Addr, Attribute, Decimal, Event, OverflowError, StdError, Storage, Uint128,
};
use sg_std::Response;

/// 100% represented as basis points
const MAX_BASIS_POINTS: u128 = 10000u128;
/// Maximum swap fee percent that can be set on trade pairs
const MAX_SWAP_FEE_PERCENT: u64 = 5000u64;

impl Pair {
    /// Create a Pair object
    pub fn new(
        id: u64,
        owner: Addr,
        collection: Addr,
        denom: String,
        pair_type: PairType,
        bonding_curve: BondingCurve,
        pair_options: PairOptions<Addr>,
    ) -> Self {
        Self {
            id,
            owner,
            collection,
            denom,
            pair_type,
            bonding_curve,
            pair_options,
            is_active: false,
        }
    }

    /// Verify that the pair is valid by checking invariants before save
    fn validate(&self) -> Result<(), ContractError> {
        if let Some(finders_fee_percent) = self.pair_options.finders_fee_percent {
            ensure!(
                finders_fee_percent <= Decimal::one(),
                ContractError::InvalidInput("finders_fee_percent is above 100%".to_string())
            );
        };

        match self.pair_type {
            PairType::Trade {
                swap_fee_percent, ..
            } => {
                ensure!(
                    swap_fee_percent <= Decimal::one(),
                    ContractError::InvalidInput("swap_fee_percent is above 100%".to_string())
                );
            }
            _ => {}
        };

        Ok(())
    }

    /// Save a pair, check invariants, update pair quotes
    /// IMPORTANT: this function must always be called when saving a pair!
    pub fn save(
        &self,
        store: &mut dyn Storage,
        mut response: Response,
    ) -> Result<Response, ContractError> {
        self.validate()?;

        // response = update_buy_from_pair_quotes(
        //     store,
        //     pair,
        //     marketplace_params.params.min_price,
        //     response,
        // )?;
        // response = update_sell_to_pair_quotes(
        //     store,
        //     pair,
        //     marketplace_params.params.min_price,
        //     response,
        // )?;

        pairs().save(store, self.id, self)?;

        Ok(response)
    }

    /// Get the recipient of assets for trades performed on this pair
    pub fn asset_recipient(&self) -> Addr {
        match &self.asset_recipient {
            Some(addr) => addr.clone(),
            None => self.owner.clone(),
        }
    }

    /// Track pair token deposits
    pub fn track_token_deposit(&mut self, amount: Uint128) -> Result<(), ContractError> {
        match self.pair_type {
            PairType::Token { mut total_tokens } => {
                total_tokens += amount;
            }
            PairType::Nft { .. } => {
                return Err(ContractError::InvalidPair(
                    "cannot deposit tokens into nft pair".to_string(),
                ));
            }
            PairType::Trade {
                mut total_tokens, ..
            } => {
                total_tokens += amount;
            }
        };

        Ok(())
    }

    /// Track pair nft deposits
    pub fn track_nft_deposit(&mut self) -> Result<(), ContractError> {
        match self.pair_type {
            PairType::Token { .. } => {
                return Err(ContractError::InvalidPair(
                    "cannot deposit nfts into token pair".to_string(),
                ));
            }
            PairType::Nft { mut total_nfts } => {
                total_nfts += 1;
            }
            PairType::Trade { mut total_nfts, .. } => {
                total_nfts += 1;
            }
        };
        Ok(())
    }

    // /// Activate the pair so that it may begin accepting trades
    // pub fn set_active(&mut self, is_active: bool) -> Result<(), ContractError> {
    //     self.is_active = is_active;
    //     Ok(())
    // }

    // /// Withdraw tokens from the pair
    // pub fn withdraw_tokens(&mut self, amount: Uint128) -> Result<(), ContractError> {
    //     if self.pair_type == PairType::Nft {
    //         return Err(ContractError::InvalidPair(
    //             "cannot withdraw tokens from nft pair".to_string(),
    //         ));
    //     }
    //     if self.total_tokens < amount {
    //         return Err(ContractError::Std(StdError::overflow(OverflowError {
    //             operation: cosmwasm_std::OverflowOperation::Sub,
    //             operand1: self.total_tokens.to_string(),
    //             operand2: amount.to_string(),
    //         })));
    //     }
    //     self.total_tokens -= amount;
    //     Ok(())
    // }

    // /// Withdraw nfts from the pair
    // pub fn withdraw_nfts(&mut self, nft_token_ids: &Vec<String>) -> Result<(), ContractError> {
    //     if self.pair_type == PairType::Token {
    //         return Err(ContractError::InvalidPair(
    //             "cannot withdraw nfts from token pair".to_string(),
    //         ));
    //     }
    //     if self.total_nfts < nft_token_ids.len() as u64 {
    //         return Err(ContractError::InternalError(
    //             "pair NFT overdraw".to_string(),
    //         ));
    //     }
    //     self.total_nfts -= nft_token_ids.len() as u64;
    //     Ok(())
    // }

    // /// Returns whether or not the pair can buy NFTs
    // pub fn can_buy_nfts(&self) -> bool {
    //     self.pair_type == PairType::Trade || self.pair_type == PairType::Token
    // }

    // /// Returns whether or not the pair can sell NFTs
    // pub fn can_sell_nfts(&self) -> bool {
    //     self.pair_type == PairType::Trade || self.pair_type == PairType::Nft
    // }

    // /// Returns the price at which this pair will sell NFTs
    // /// Note: the buy_from_pair_quote is indexed by PairQuote for future discovery
    // pub fn get_buy_from_pair_quote(
    //     &self,
    //     min_quote: Uint128,
    // ) -> Result<Option<Uint128>, ContractError> {
    //     // Calculate the buy price with respect to pair types and bonding curves
    //     let buy_price = match self.pair_type {
    //         PairType::Token => Err(ContractError::InvalidPair(
    //             "pair cannot sell nfts".to_string(),
    //         )),
    //         PairType::Nft => Ok(self.spot_price),
    //         PairType::Trade => match self.bonding_curve {
    //             BondingCurve::Linear => self
    //                 .spot_price
    //                 .checked_add(self.delta)
    //                 .map_err(|e| ContractError::Std(StdError::overflow(e))),
    //             BondingCurve::Exponential => {
    //                 let net_delta = Uint128::from(MAX_BASIS_POINTS)
    //                     .checked_add(self.delta)
    //                     .map_err(|e| ContractError::Std(StdError::overflow(e)))?;
    //                 self.spot_price
    //                     .checked_mul(net_delta)
    //                     .map_err(|e| ContractError::Std(StdError::overflow(e)))?
    //                     .checked_div(Uint128::from(MAX_BASIS_POINTS))
    //                     .map_err(|e| ContractError::Std(StdError::divide_by_zero(e)))?
    //                     .checked_add(Uint128::one())
    //                     .map_err(|e| ContractError::Std(StdError::overflow(e)))
    //             }
    //             BondingCurve::ConstantProduct => {
    //                 if self.total_nfts <= 1 {
    //                     return Ok(None);
    //                 }
    //                 let buy_price = self
    //                     .total_tokens
    //                     .checked_div(Uint128::from(self.total_nfts - 1))
    //                     .map_err(|e| ContractError::Std(StdError::divide_by_zero(e)))?
    //                     .checked_add(Uint128::one())
    //                     .map_err(|e| ContractError::Std(StdError::overflow(e)))?;
    //                 Ok(buy_price)
    //             }
    //         },
    //     }?;
    //     // If the pair has no NFTs to sell, or quote is below min, return None
    //     if self.total_nfts == 0 || buy_price < min_quote {
    //         return Ok(None);
    //     }
    //     Ok(Some(buy_price))
    // }

    // /// Returns the price at which this pair will buy NFTs
    // /// Note: the sell_to_pair_quote is indexed by PairQuote for future discovery
    // pub fn get_sell_to_pair_quote(
    //     &self,
    //     min_quote: Uint128,
    // ) -> Result<Option<Uint128>, ContractError> {
    //     if !self.can_buy_nfts() {
    //         return Err(ContractError::InvalidPair(
    //             "pair cannot buy nfts".to_string(),
    //         ));
    //     }
    //     let sell_price = match self.bonding_curve {
    //         BondingCurve::Linear | BondingCurve::Exponential => self.spot_price,
    //         BondingCurve::ConstantProduct => {
    //             if self.total_nfts < 1 {
    //                 return Ok(None);
    //             }
    //             self.total_tokens
    //                 .checked_div(Uint128::from(self.total_nfts + 1))
    //                 .unwrap()
    //         }
    //     };
    //     // If the pair has insufficient tokens to buy the NFT, or quote is below min, return None
    //     if self.total_tokens < sell_price || sell_price < min_quote {
    //         return Ok(None);
    //     }
    //     Ok(Some(sell_price))
    // }

    // /// Buy an NFT from the pair
    // pub fn buy_nft_from_pair(
    //     &mut self,
    //     nft_swap: &NftSwap,
    //     sale_price: Uint128,
    // ) -> Result<(), ContractError> {
    //     if !self.can_sell_nfts() {
    //         return Err(ContractError::InvalidPair(
    //             "pair cannot sell nfts".to_string(),
    //         ));
    //     }
    //     if !self.is_active {
    //         return Err(ContractError::InvalidPair(
    //             "pair is not active".to_string(),
    //         ));
    //     }

    //     // If sale price exceeds the max expected, return an error
    //     if sale_price > nft_swap.token_amount {
    //         return Err(ContractError::SwapError(
    //             "pair sale price is above max expected".to_string(),
    //         ));
    //     }

    //     // Decrement total_nfts on pair
    //     if self.total_nfts == 0 {
    //         return Err(ContractError::SwapError(
    //             "pair does not own any NFTS".to_string(),
    //         ));
    //     } else {
    //         self.total_nfts -= 1;
    //     }

    //     Ok(())
    // }

    // /// Sell an NFT to the pair
    // pub fn sell_nft_to_pair(
    //     &mut self,
    //     nft_swap: &NftSwap,
    //     sale_price: Uint128,
    // ) -> Result<(), ContractError> {
    //     if !self.can_buy_nfts() {
    //         return Err(ContractError::InvalidPair(
    //             "pair cannot buy nfts".to_string(),
    //         ));
    //     }
    //     if !self.is_active {
    //         return Err(ContractError::InvalidPair(
    //             "pair is not active".to_string(),
    //         ));
    //     }

    //     // If sale price is below the min expected, return an error
    //     if sale_price < nft_swap.token_amount {
    //         return Err(ContractError::SwapError(
    //             "pair sale price is below min expected".to_string(),
    //         ));
    //     }

    //     // Deduct the sale price from the pair's token balance
    //     self.total_tokens -= sale_price;

    //     Ok(())
    // }

    // /// Updates the spot price of the pair depending on the transaction type
    // pub fn update_spot_price(&mut self, tx_type: &TransactionType) -> Result<(), StdError> {
    //     let result = match tx_type {
    //         TransactionType::UserSubmitsNfts => match self.bonding_curve {
    //             BondingCurve::Linear => self
    //                 .spot_price
    //                 .checked_sub(self.delta)
    //                 .map_err(StdError::overflow),
    //             BondingCurve::Exponential => {
    //                 let denominator = Uint128::from(MAX_BASIS_POINTS)
    //                     .checked_add(self.delta)
    //                     .map_err(StdError::overflow)?;
    //                 self.spot_price
    //                     .checked_mul(Uint128::from(MAX_BASIS_POINTS))
    //                     .map_err(StdError::overflow)?
    //                     .checked_div(denominator)
    //                     .map_err(StdError::divide_by_zero)
    //             }
    //             BondingCurve::ConstantProduct => self
    //                 .total_tokens
    //                 .checked_div(Uint128::from(self.total_nfts))
    //                 .map_err(StdError::divide_by_zero),
    //         },
    //         TransactionType::UserSubmitsTokens => match self.bonding_curve {
    //             BondingCurve::Linear => self
    //                 .spot_price
    //                 .checked_add(self.delta)
    //                 .map_err(StdError::overflow),
    //             BondingCurve::Exponential => {
    //                 let net_delta = Uint128::from(MAX_BASIS_POINTS)
    //                     .checked_add(self.delta)
    //                     .map_err(StdError::overflow)?;
    //                 self.spot_price
    //                     .checked_mul(net_delta)
    //                     .map_err(StdError::overflow)?
    //                     .checked_div(Uint128::from(MAX_BASIS_POINTS))
    //                     .map_err(StdError::divide_by_zero)?
    //                     .checked_add(Uint128::one())
    //                     .map_err(StdError::overflow)
    //             }
    //             BondingCurve::ConstantProduct => self
    //                 .total_tokens
    //                 .checked_div(Uint128::from(self.total_nfts))
    //                 .map_err(StdError::divide_by_zero)?
    //                 .checked_add(Uint128::one())
    //                 .map_err(StdError::overflow),
    //         },
    //     };
    //     match result {
    //         Ok(_spot_price) => {
    //             self.spot_price = _spot_price;
    //             Ok(())
    //         }
    //         Err(_err) => {
    //             self.is_active = false;
    //             Err(_err)
    //         }
    //     }
    // }

    // /// Create an event with all the Pair properties
    // pub fn create_event_all_props(&self, event_type: &str) -> Result<Event, ContractError> {
    //     self.create_event(
    //         event_type,
    //         vec![
    //             "id",
    //             "collection",
    //             "owner",
    //             "asset_recipient",
    //             "pair_type",
    //             "bonding_curve",
    //             "spot_price",
    //             "delta",
    //             "total_tokens",
    //             "total_nfts",
    //             "is_active",
    //             "swap_fee_percent",
    //             "finders_fee_percent",
    //             "reinvest_tokens",
    //             "reinvest_nfts",
    //         ],
    //     )
    // }

    // /// Create an event with certain Pair properties
    // pub fn create_event(
    //     &self,
    //     event_type: &str,
    //     attr_keys: Vec<&str>,
    // ) -> Result<Event, ContractError> {
    //     let mut attributes: Vec<Attribute> = vec![];
    //     for attr_keys in attr_keys {
    //         let attribute = match attr_keys {
    //             "id" => attr("id", self.id.to_string()),
    //             "collection" => attr("collection", self.collection.to_string()),
    //             "owner" => attr("owner", self.owner.to_string()),
    //             "asset_recipient" => attr(
    //                 "asset_recipient",
    //                 self.asset_recipient
    //                     .as_ref()
    //                     .map_or("None".to_string(), |addr| addr.to_string()),
    //             ),
    //             "pair_type" => attr("pair_type", self.pair_type.to_string()),
    //             "bonding_curve" => attr("bonding_curve", self.bonding_curve.to_string()),
    //             "spot_price" => attr("spot_price", self.spot_price.to_string()),
    //             "delta" => attr("delta", self.delta.to_string()),
    //             "total_tokens" => attr("total_tokens", self.total_tokens.to_string()),
    //             "total_nfts" => attr("total_nfts", self.total_nfts.to_string()),
    //             "is_active" => attr("is_active", self.is_active.to_string()),
    //             "swap_fee_percent" => attr("swap_fee_percent", self.swap_fee_percent.to_string()),
    //             "finders_fee_percent" => {
    //                 attr("finders_fee_percent", self.finders_fee_percent.to_string())
    //             }
    //             "reinvest_tokens" => attr("reinvest_tokens", self.reinvest_tokens.to_string()),
    //             "reinvest_nfts" => attr("reinvest_nfts", self.reinvest_nfts.to_string()),
    //             _key => {
    //                 return Err(ContractError::InvalidPropertyKeyError(format!(
    //                     "Invalid property key: {}",
    //                     _key
    //                 )));
    //             }
    //         };
    //         attributes.push(attribute);
    //     }
    //     Ok(Event::new(event_type).add_attributes(attributes))
    // }
}
