use crate::error::ContractError;
use crate::helpers::{transfer_nft, transfer_token};
use crate::msg::{NftSwap, PoolNftSwap, SwapParams};
use crate::state::{buy_pool_quotes, pools, sell_pool_quotes, Pool, PoolQuote, PoolType};

use core::cmp::Ordering;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{coin, Addr, StdResult, Storage, Uint128};
use cosmwasm_std::{Decimal, Order};
use sg1::fair_burn;
use sg721::RoyaltyInfoResponse;
use sg_std::{Response, NATIVE_DENOM};
use std::collections::{BTreeMap, BTreeSet};

/// A struct for tracking in memory pools that are involved in swaps
pub struct PoolPair {
    /// When true, the pool will be saved to storage at the end of the transaction
    pub needs_saving: bool,
    /// The price at which to perform the swap
    pub quote_price: Uint128,
    /// The pool object to be swapped in
    pub pool: Pool,
}

impl Ord for PoolPair {
    fn cmp(&self, other: &Self) -> Ordering {
        self.quote_price.cmp(&other.quote_price)
    }
}

impl PartialOrd for PoolPair {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PoolPair {
    fn eq(&self, other: &Self) -> bool {
        self.quote_price == other.quote_price
    }
}

impl Eq for PoolPair {}

/// Defines whether the end user is buying or selling NFTs
#[cw_serde]
pub enum TransactionType {
    Sell,
    Buy,
}

/// A token payment that needs to be executed at the end of a transaction
#[cw_serde]
pub struct TokenPayment {
    pub amount: Uint128,
    pub address: String,
}

/// An NFT payment that needs to be executed at the end of a transaction
#[cw_serde]
pub struct NftPayment {
    pub nft_token_id: String,
    pub address: String,
}

/// A summary of an individual swap
#[cw_serde]
pub struct Swap {
    pub pool_id: u64,
    pub transaction_type: TransactionType,
    pub spot_price: Uint128,
    pub network_fee: Uint128,
    pub finder_payment: Option<TokenPayment>,
    pub royalty_payment: Option<TokenPayment>,
    pub nft_payment: Option<NftPayment>,
    pub seller_payment: Option<TokenPayment>,
}

type IterResults = StdResult<(u64, PoolQuote)>;

/// A struct for managing a series of swaps
pub struct SwapProcessor<'a> {
    /// The type of transaction (buy or sell)
    pub tx_type: TransactionType,
    /// The address of the NFT collection
    pub collection: Addr,
    /// The sender address
    pub sender: Addr,
    /// The amount of tokens sent to the contract by the end user
    pub remaining_balance: Uint128,
    /// The address that will receive assets on the side of the end user
    pub seller_recipient: Addr,
    /// The trading fee percentage to be burned
    pub trading_fee_percent: Decimal,
    /// The royalty info for the NFT collection
    pub royalty: Option<RoyaltyInfoResponse>,
    /// The address of the finder of the transaction
    pub finder: Option<Addr>,
    /// The address to receive developer burn fees
    pub developer: Option<Addr>,
    /// A set of in memory pools that are involved in the transaction
    pub pool_set: BTreeSet<PoolPair>,
    /// The latest pool that was retrieved
    pub latest: Option<u64>,
    /// An iterator for retrieving sorted pool quotes
    pub pool_quote_iter: Option<Box<dyn Iterator<Item = IterResults> + 'a>>,
    /// A list of swaps that have been processed
    pub swaps: Vec<Swap>,
}

impl<'a> SwapProcessor<'a> {
    /// Creates a new SwapProcessor object
    pub fn new(
        tx_type: TransactionType,
        collection: Addr,
        sender: Addr,
        remaining_balance: Uint128,
        seller_recipient: Addr,
        trading_fee_percent: Decimal,
        royalty: Option<RoyaltyInfoResponse>,
        finder: Option<Addr>,
        developer: Option<Addr>,
    ) -> Self {
        Self {
            tx_type,
            collection,
            sender,
            remaining_balance,
            seller_recipient,
            trading_fee_percent,
            royalty,
            finder,
            developer,
            pool_set: BTreeSet::new(),
            latest: None,
            pool_quote_iter: None,
            swaps: vec![],
        }
    }

    /// Create an individual swap object
    fn create_swap(&mut self, pool: &Pool, payment_amount: Uint128, nft_token_id: String) -> Swap {
        // Subtract from received amount in the case of a buy
        if self.tx_type == TransactionType::Buy {
            self.remaining_balance -= payment_amount;
        }

        // Calculate burn fee
        let network_fee = payment_amount * self.trading_fee_percent / Uint128::from(100u128);

        // Calculate seller payment (mutable)
        let mut seller_amount = payment_amount - network_fee;

        // Calculate finder payment, deduct from seller payment
        let mut finder_payment = None;
        if self.finder.is_some() && !pool.finders_fee_percent.is_zero() {
            let finder_amount = payment_amount * pool.finders_fee_percent;
            if !finder_amount.is_zero() {
                seller_amount -= finder_amount;
                finder_payment = Some(TokenPayment {
                    amount: finder_amount,
                    address: self.finder.as_ref().unwrap().to_string(),
                });
            }
        }

        // Calculate royalty payment, deduct from seller payment
        let mut royalty_payment = None;
        if let Some(_royalty) = &self.royalty {
            let royalty_amount = payment_amount * _royalty.share;
            if !royalty_amount.is_zero() {
                seller_amount -= royalty_amount;
                royalty_payment = Some(TokenPayment {
                    amount: royalty_amount,
                    address: _royalty.payment_address.clone(),
                });
            }
        }

        // Set the addresses that will receive the NFT and token payment
        let (nft_recipient, token_recipient) = match &self.tx_type {
            TransactionType::Buy => (self.seller_recipient.clone(), pool.get_recipient()),
            TransactionType::Sell => (pool.get_recipient(), self.seller_recipient.clone()),
        };

        Swap {
            pool_id: pool.id,
            transaction_type: self.tx_type.clone(),
            spot_price: payment_amount,
            network_fee,
            finder_payment,
            royalty_payment,
            nft_payment: Some(NftPayment {
                nft_token_id,
                address: nft_recipient.to_string(),
            }),
            seller_payment: Some(TokenPayment {
                amount: seller_amount,
                address: token_recipient.to_string(),
            }),
        }
    }

    /// Process a swap
    pub fn process_swap(
        &mut self,
        pool: &mut Pool,
        nft_swap: NftSwap,
        tx_type: TransactionType,
    ) -> Result<(), ContractError> {
        // Retrieve the transaction price from the pool
        let sale_price = match tx_type {
            TransactionType::Buy => pool.buy_nft_from_pool(&nft_swap)?,
            TransactionType::Sell => pool.sell_nft_to_pool(&nft_swap)?,
        };
        // Set the resulting swap with fees included
        let mut swap = self.create_swap(pool, sale_price, nft_swap.nft_token_id);

        // Reinvest tokens or NFTs if applicable
        if pool.pool_type == PoolType::Trade {
            if tx_type == TransactionType::Buy && pool.reinvest_tokens {
                let reinvest_amount = swap.seller_payment.unwrap().amount;
                swap.seller_payment = None;
                pool.deposit_tokens(reinvest_amount)?;
            } else if tx_type == TransactionType::Sell && pool.reinvest_nfts {
                let reinvest_nft_token_id = swap.nft_payment.unwrap().nft_token_id;
                swap.nft_payment = None;
                pool.deposit_nfts(&vec![reinvest_nft_token_id])?;
            }
        }
        // Update the pool spot price
        pool.update_spot_price(&self.tx_type);

        self.swaps.push(swap);
        Ok(())
    }

    /// Push asset transfer messages to the response
    pub fn commit_messages(&self, response: &mut Response) -> Result<(), ContractError> {
        if self.swaps.is_empty() {
            return Err(ContractError::SwapError("no swaps found".to_string()));
        }

        let mut total_network_fee = Uint128::zero();
        let mut token_payments: BTreeMap<&str, Uint128> = BTreeMap::new();

        // Insert refund amount if there is a remainder
        if !self.remaining_balance.is_zero() {
            token_payments.insert(self.sender.as_str(), self.remaining_balance);
        }

        // Iterate over swaps and reduce token payments that need to be made
        for swap in self.swaps.iter() {
            // Aggregate network fees
            total_network_fee += swap.network_fee;

            // Track finder payments
            if let Some(_finder_payment) = &swap.finder_payment {
                let payment = token_payments
                    .entry(&_finder_payment.address)
                    .or_insert(Uint128::zero());
                *payment += _finder_payment.amount;
            }

            // Track royalty payments
            if let Some(_royalty_payment) = &swap.royalty_payment {
                let payment = token_payments
                    .entry(&_royalty_payment.address)
                    .or_insert(Uint128::zero());
                *payment += _royalty_payment.amount;
            }

            // Track seller payments
            if let Some(_seller_payment) = &swap.seller_payment {
                let payment = token_payments
                    .entry(&_seller_payment.address)
                    .or_insert(Uint128::zero());
                *payment += _seller_payment.amount;
            }

            // Push transfer NFT messages
            if let Some(_nft_payment) = &swap.nft_payment {
                transfer_nft(
                    &_nft_payment.nft_token_id,
                    &_nft_payment.address,
                    self.collection.as_ref(),
                    response,
                )?;
            }
        }

        fair_burn(total_network_fee.u128(), self.developer.clone(), response);

        // Push transfer token messages
        for token_payment in token_payments {
            transfer_token(
                coin(token_payment.1.u128(), NATIVE_DENOM),
                token_payment.0,
                response,
            )?;
        }

        Ok(())
    }

    /// Load the pool with the next best price
    pub fn load_next_pool(
        &mut self,
        storage: &'a dyn Storage,
    ) -> Result<Option<PoolPair>, ContractError> {
        // Init iter
        if self.pool_quote_iter.is_none() {
            self.pool_quote_iter = Some(match &self.tx_type {
                TransactionType::Buy => buy_pool_quotes()
                    .idx
                    .collection_buy_price
                    .sub_prefix(self.collection.clone())
                    .range(storage, None, None, Order::Ascending),
                TransactionType::Sell => sell_pool_quotes()
                    .idx
                    .collection_sell_price
                    .sub_prefix(self.collection.clone())
                    .range(storage, None, None, Order::Descending),
            })
        }

        // Get the current pool
        let current = match &self.tx_type {
            // For buys, the first pool will have the lowest quote
            TransactionType::Buy => self.pool_set.first(),
            // For sells, the last pool will have the highest quote
            TransactionType::Sell => self.pool_set.last(),
        };

        // If the pool is empty, or the front of the pool is the latest fetched, load the next pool
        // Note: if the front of the pool is not the latest fetched, that means the next pool won't have the best price
        if current.is_none() || Some(current.unwrap().pool.id) == self.latest {
            let (pool_id, pool_quote) = self.pool_quote_iter.as_mut().unwrap().next().unwrap()?;

            let pool = pools()
                .load(storage, pool_id)
                .map_err(|_| ContractError::InvalidPool("pool does not exist".to_string()))?;

            self.pool_set.insert(PoolPair {
                // Recently fetched pools do not need saving yet
                needs_saving: false,
                quote_price: pool_quote.quote_price,
                pool,
            });
            self.latest = Some(pool_id);
        }

        Ok(match &self.tx_type {
            // For buys, the first pool will have the lowest quote
            TransactionType::Buy => self.pool_set.pop_first(),
            // For sells, the last pool will have the highest quote
            TransactionType::Sell => self.pool_set.pop_last(),
        })
    }

    /// Swap NFTs to tokens directly with the specified pool
    pub fn direct_swap_nfts_for_tokens(
        &mut self,
        pool: Pool,
        nfts_to_swap: Vec<NftSwap>,
        swap_params: SwapParams,
    ) -> Result<(), ContractError> {
        // Load the only pool that will be needed
        let mut pool = pool;
        {
            for nft_swap in nfts_to_swap {
                let result = self.process_swap(&mut pool, nft_swap, TransactionType::Sell);
                match result {
                    Ok(_) => {}
                    Err(ContractError::SwapError(_err)) => {
                        // If the swap is robust, break out of the loop and save the pool
                        if swap_params.robust {
                            break;
                        } else {
                            // If the swap is not robust, throw an error
                            return Err(ContractError::SwapError(_err));
                        }
                    }
                    Err(_err) => return Err(_err),
                }
            }
        }
        // Queue the only pool for saving
        self.pool_set.insert(PoolPair {
            needs_saving: true,
            quote_price: Uint128::zero(),
            pool,
        });
        Ok(())
    }

    /// Swap NFTs to tokens, using the best priced pools
    pub fn swap_nfts_for_tokens(
        &mut self,
        storage: &'a dyn Storage,
        nfts_to_swap: Vec<NftSwap>,
        swap_params: SwapParams,
    ) -> Result<(), ContractError> {
        for nft_swap in nfts_to_swap {
            // Load best priced pool
            let pool_pair_option = self.load_next_pool(storage)?;
            // No pools found, so return empty
            if pool_pair_option.is_none() {
                return Ok(());
            }
            let mut pool_pair = pool_pair_option.unwrap();
            {
                let result =
                    self.process_swap(&mut pool_pair.pool, nft_swap, TransactionType::Sell);
                match result {
                    Ok(_) => {}
                    Err(ContractError::SwapError(_err)) => {
                        // If the swap is robust, break out of function
                        if swap_params.robust {
                            return Ok(());
                        } else {
                            // If the swap is not robust, throw an error
                            return Err(ContractError::SwapError(_err));
                        }
                    }
                    Err(_err) => return Err(_err),
                }
            }
            // Queue the pool for saving
            pool_pair.needs_saving = true;
            self.pool_set.insert(pool_pair);
        }
        Ok(())
    }

    /// Swap tokens to specific NFTs directly with the specified pool
    pub fn swap_tokens_for_specific_nfts(
        &mut self,
        storage: &'a dyn Storage,
        nfts_to_swap_for: Vec<PoolNftSwap>,
        swap_params: SwapParams,
    ) -> Result<(), ContractError> {
        // Create a pool map for tracking swap pools
        let mut pool_map: BTreeMap<u64, Pool> = BTreeMap::new();

        for pool_nfts in nfts_to_swap_for {
            // Retrieve pool from pool_map
            let mut pool_option = pool_map.remove(&pool_nfts.pool_id);
            // If pool is not in pool_map, load it from storage
            if pool_option.is_none() {
                pool_option = pools().may_load(storage, pool_nfts.pool_id)?;
            }
            // If pool is still not found, return error
            if pool_option.is_none() {
                return Err(ContractError::InvalidPool("pool not found".to_string()));
            }
            let mut pool = pool_option.unwrap();

            // Iterate for all NFTs selected for the given
            for nft_swap in pool_nfts.nft_swaps {
                let result = self.process_swap(&mut pool, nft_swap, TransactionType::Buy);
                match result {
                    Ok(_) => {}
                    Err(ContractError::SwapError(_err)) => {
                        // If the swap is robust, break out of loop and continue processing other pools
                        if swap_params.robust {
                            break;
                        } else {
                            // If the swap is not robust, throw an error
                            return Err(ContractError::SwapError(_err));
                        }
                    }
                    Err(_err) => return Err(_err),
                }
            }
            // Put pool back in map
            pool_map.insert(pool.id, pool);
        }
        // Queue all pools for saving
        for (_, pool) in pool_map {
            self.pool_set.insert(PoolPair {
                needs_saving: true,
                quote_price: Uint128::zero(),
                pool,
            });
        }
        Ok(())
    }

    /// Swap tokens to any NFTs, using the best priced pools
    pub fn swap_tokens_for_any_nfts(
        &mut self,
        storage: &'a dyn Storage,
        min_expected_token_input: Vec<Uint128>,
        swap_params: SwapParams,
    ) -> Result<(), ContractError> {
        for token_amount in min_expected_token_input {
            // Load best priced pool
            let pool_pair_option = self.load_next_pool(storage)?;
            // No pools found, so return empty
            if pool_pair_option.is_none() {
                return Ok(());
            }
            let mut pool_pair = pool_pair_option.unwrap();
            {
                // Grab first NFT from the pool
                let nft_token_id = pool_pair.pool.nft_token_ids.first().unwrap().to_string();
                let result = self.process_swap(
                    &mut pool_pair.pool,
                    NftSwap {
                        nft_token_id,
                        token_amount,
                    },
                    TransactionType::Buy,
                );
                match result {
                    Ok(_) => {}
                    Err(ContractError::SwapError(_err)) => {
                        // If the swap is robust, break out of function
                        if swap_params.robust {
                            return Ok(());
                        } else {
                            // If the swap is not robust, throw an error
                            return Err(ContractError::SwapError(_err));
                        }
                    }
                    Err(_err) => return Err(_err),
                }
            }
            // Queue the pool for saving
            pool_pair.needs_saving = true;
            self.pool_set.insert(pool_pair);
        }
        Ok(())
    }
}
