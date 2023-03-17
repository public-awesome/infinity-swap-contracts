use crate::error::ContractError;
use crate::helpers::{transfer_nft, transfer_token};
use crate::msg::{NftSwap, PoolNftSwap, SwapParams, TransactionType};
use crate::state::{buy_pool_quotes, pools, sell_pool_quotes, Pool, PoolQuote, PoolType};

use core::cmp::Ordering;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    attr, coin, Addr, ContractInfo, Decimal, Event, Order, StdResult, Storage, Uint128,
};
use sg1::fair_burn;
use sg721::RoyaltyInfoResponse;
use sg_std::{Response, NATIVE_DENOM};
use std::collections::{BTreeMap, BTreeSet};

/// A struct for tracking in memory pools that are involved in swaps
#[derive(Debug)]
pub struct PoolQueueItem {
    /// The pool object to handle a swap
    pub pool: Pool,
    /// The price at which to perform the swap
    pub quote_price: Uint128,
    /// When true, the pool will be saved to storage at the end of the transaction
    pub needs_saving: bool,
    /// Used to indicate whether the pool can continue to process swaps
    pub usable: bool,
}

impl Ord for PoolQueueItem {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.quote_price, self.pool.id).cmp(&(other.quote_price, other.pool.id))
    }
}

impl PartialOrd for PoolQueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PoolQueueItem {
    fn eq(&self, other: &Self) -> bool {
        (self.quote_price, self.pool.id) == (other.quote_price, other.pool.id)
    }
}

impl Eq for PoolQueueItem {}

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
    pub nft_payment: NftPayment,
    pub finder_payment: Option<TokenPayment>,
    pub royalty_payment: Option<TokenPayment>,
    pub seller_payment: Option<TokenPayment>,
}

impl From<&Swap> for Event {
    fn from(val: &Swap) -> Self {
        let mut attributes = vec![
            attr("pool_id", val.pool_id.to_string()),
            attr("transaction_type", val.transaction_type.to_string()),
            attr("spot_price", val.spot_price.to_string()),
            attr("network_fee", val.network_fee.to_string()),
        ];
        attributes.extend([
            attr("nft_payment_address", val.nft_payment.address.to_string()),
            attr(
                "nft_payment_token_id",
                val.nft_payment.nft_token_id.to_string(),
            ),
        ]);
        if val.finder_payment.is_some() {
            let finder_payment = val.finder_payment.as_ref().unwrap();
            attributes.extend([
                attr("finder_payment_address", finder_payment.address.to_string()),
                attr("finder_payment_amount", finder_payment.amount.to_string()),
            ]);
        }
        if val.royalty_payment.is_some() {
            let royalty_payment = val.royalty_payment.as_ref().unwrap();
            attributes.extend([
                attr(
                    "royalty_payment_address",
                    royalty_payment.address.to_string(),
                ),
                attr("royalty_payment_amount", royalty_payment.amount.to_string()),
            ]);
        }
        if val.seller_payment.is_some() {
            let seller_payment = val.seller_payment.as_ref().unwrap();
            attributes.extend([
                attr("seller_payment_address", seller_payment.address.to_string()),
                attr("seller_payment_amount", seller_payment.amount.to_string()),
            ]);
        }
        Event::new("swap").add_attributes(attributes)
    }
}

type IterResults = StdResult<(u64, PoolQuote)>;

/// A struct for managing a series of swaps
pub struct SwapProcessor<'a> {
    /// The type of transaction (buy or sell)
    tx_type: TransactionType,
    /// Contract info for this contract
    contract_info: ContractInfo,
    /// The address of the NFT collection
    collection: Addr,
    /// The sender address
    sender: Addr,
    /// The amount of tokens sent to the contract by the end user
    remaining_balance: Uint128,
    /// The address that will receive assets on the side of the end user
    seller_recipient: Addr,
    /// The trading fee percentage to be burned
    trading_fee_percent: Decimal,
    /// The minimum quote price to be handled by the contract
    min_quote: Uint128,
    /// The royalty info for the NFT collection
    royalty: Option<RoyaltyInfoResponse>,
    /// The address of the finder of the transaction
    finder: Option<Addr>,
    /// The address to receive developer burn fees
    developer: Option<Addr>,
    /// A set of in memory pools that are involved in the transaction
    pool_queue: BTreeSet<PoolQueueItem>,
    /// The latest pool that was retrieved
    latest: Option<u64>,
    /// Skip next pool load to improve efficiency
    skip_next_pool_load: bool,
    /// An iterator for retrieving sorted pool quotes
    pool_quote_iter: Option<Box<dyn Iterator<Item = IterResults> + 'a>>,
    /// A set of in memory pools that should be saved at the end of the transaction
    pub pools_to_save: BTreeMap<u64, Pool>,
    /// A list of swaps that have been processed
    pub swaps: Vec<Swap>,
}

impl<'a> SwapProcessor<'a> {
    /// Creates a new SwapProcessor object
    pub fn new(
        tx_type: TransactionType,
        contract_info: ContractInfo,
        collection: Addr,
        sender: Addr,
        remaining_balance: Uint128,
        seller_recipient: Addr,
        trading_fee_percent: Decimal,
        min_quote: Uint128,
        royalty: Option<RoyaltyInfoResponse>,
        finder: Option<Addr>,
        developer: Option<Addr>,
    ) -> Self {
        Self {
            tx_type,
            contract_info,
            collection,
            sender,
            remaining_balance,
            seller_recipient,
            trading_fee_percent,
            min_quote,
            royalty,
            finder,
            developer,
            pool_queue: BTreeSet::new(),
            pools_to_save: BTreeMap::new(),
            latest: None,
            skip_next_pool_load: false,
            pool_quote_iter: None,
            swaps: vec![],
        }
    }

    /// Create an individual swap object
    fn create_swap(&mut self, pool: &Pool, payment_amount: Uint128, nft_token_id: String) -> Swap {
        // Subtract from received amount in the case of a buy
        if self.tx_type == TransactionType::TokensForNfts {
            self.remaining_balance -= payment_amount;
        }

        // Calculate burn fee
        let network_fee = payment_amount * self.trading_fee_percent / Uint128::from(100u128);

        // Calculate seller payment (mutable)
        let mut seller_amount = payment_amount - network_fee;

        // Calculate finder payment, deduct from seller payment
        let mut finder_payment = None;
        if self.finder.is_some() && !pool.finders_fee_percent.is_zero() {
            let finder_amount = payment_amount * pool.finders_fee_percent / Uint128::from(100u128);
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
            TransactionType::TokensForNfts => (self.seller_recipient.clone(), pool.get_recipient()),
            TransactionType::NftsForTokens => (pool.get_recipient(), self.seller_recipient.clone()),
        };

        Swap {
            pool_id: pool.id,
            transaction_type: self.tx_type.clone(),
            spot_price: payment_amount,
            network_fee,
            nft_payment: NftPayment {
                nft_token_id,
                address: nft_recipient.to_string(),
            },
            finder_payment,
            royalty_payment,
            seller_payment: Some(TokenPayment {
                amount: seller_amount,
                address: token_recipient.to_string(),
            }),
        }
    }

    /// Process a swap
    fn process_swap(
        &mut self,
        pool_queue_item: PoolQueueItem,
        nft_swap: NftSwap,
        robust: bool,
    ) -> Result<(PoolQueueItem, bool), ContractError> {
        let mut pool_queue_item = pool_queue_item;

        // Withdraw assets from the pool
        let result = match self.tx_type {
            TransactionType::TokensForNfts => pool_queue_item
                .pool
                .buy_nft_from_pool(&nft_swap, pool_queue_item.quote_price),
            TransactionType::NftsForTokens => pool_queue_item
                .pool
                .sell_nft_to_pool(&nft_swap, pool_queue_item.quote_price),
        };
        match result {
            Ok(_) => {}
            Err(ContractError::SwapError(_err)) => {
                if robust {
                    pool_queue_item.usable = false;
                    return Ok((pool_queue_item, false));
                } else {
                    // otherwise throw the error
                    return Err(ContractError::SwapError(_err));
                }
            }
            Err(err) => return Err(err),
        };

        // Set the resulting swap with fees included
        let mut swap = self.create_swap(
            &pool_queue_item.pool,
            pool_queue_item.quote_price,
            nft_swap.nft_token_id,
        );

        // Reinvest tokens or NFTs if applicable
        if pool_queue_item.pool.pool_type == PoolType::Trade {
            if self.tx_type == TransactionType::TokensForNfts
                && pool_queue_item.pool.reinvest_tokens
            {
                let reinvest_amount = swap.seller_payment.unwrap().amount;
                swap.seller_payment = None;
                pool_queue_item.pool.deposit_tokens(reinvest_amount)?;
            } else if self.tx_type == TransactionType::NftsForTokens
                && pool_queue_item.pool.reinvest_nfts
            {
                swap.nft_payment.address = self.contract_info.address.to_string();
                pool_queue_item
                    .pool
                    .deposit_nfts(&vec![swap.nft_payment.nft_token_id.clone()])?;
            }
        }
        self.swaps.push(swap);

        // Pool needs saving past this point
        pool_queue_item.needs_saving = true;

        // Update the pool spot price
        let result = pool_queue_item.pool.update_spot_price(&self.tx_type);
        if result.is_err() {
            pool_queue_item.usable = false;
            return Ok((pool_queue_item, true));
        }
        let get_next_pool_quote = match self.tx_type {
            TransactionType::TokensForNfts => pool_queue_item.pool.get_sell_quote(self.min_quote),
            TransactionType::NftsForTokens => pool_queue_item.pool.get_buy_quote(self.min_quote),
        };
        if get_next_pool_quote.is_err() {
            pool_queue_item.usable = false;
            return Ok((pool_queue_item, true));
        }
        let next_pool_quote = get_next_pool_quote.unwrap();
        if next_pool_quote.is_none() {
            pool_queue_item.usable = false;
            return Ok((pool_queue_item, true));
        }
        pool_queue_item.quote_price = next_pool_quote.unwrap();
        pool_queue_item.usable = true;
        Ok((pool_queue_item, true))
    }

    /// Push asset transfer messages to the response
    fn commit_messages(&self, response: &mut Response) -> Result<(), ContractError> {
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

            // Push transfer NFT messages
            transfer_nft(
                &swap.nft_payment.nft_token_id,
                &swap.nft_payment.address,
                self.collection.as_ref(),
                response,
            )?;

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

    /// Move pools from pool_queue to pools_to_save
    fn move_pools(&mut self) {
        let mut pool_queue_item = self.pool_queue.pop_first();
        while let Some(_pool_queue_item) = pool_queue_item {
            if _pool_queue_item.needs_saving {
                self.pools_to_save
                    .insert(_pool_queue_item.pool.id, _pool_queue_item.pool);
            }
            pool_queue_item = self.pool_queue.pop_first();
        }
    }

    /// Load the pool with the next best price
    fn load_next_pool(
        &mut self,
        storage: &'a dyn Storage,
    ) -> Result<Option<PoolQueueItem>, ContractError> {
        // Init iter
        if self.pool_quote_iter.is_none() {
            self.pool_quote_iter = Some(match &self.tx_type {
                TransactionType::TokensForNfts => sell_pool_quotes()
                    .idx
                    .collection_sell_price
                    .sub_prefix(self.collection.clone())
                    .range(storage, None, None, Order::Ascending),
                TransactionType::NftsForTokens => buy_pool_quotes()
                    .idx
                    .collection_buy_price
                    .sub_prefix(self.collection.clone())
                    .range(storage, None, None, Order::Descending),
            })
        }

        // If the pool is empty, or the front of the pool is the latest fetched, load the next pool
        // Note: if the front of the pool is not the latest fetched, that means the next pool won't have the best price
        if self.pool_queue.len() < 2 || !self.skip_next_pool_load {
            // Try and fetch next pool quote
            let next_pool_quote = self.pool_quote_iter.as_mut().unwrap().next();

            // If next pool quote exists fetch and set PoolQueueItem
            if let Some(_next_pool_quote) = next_pool_quote {
                let (pool_id, pool_quote) = _next_pool_quote?;

                let pool = pools().load(storage, pool_id).map_err(|_| {
                    ContractError::PoolNotFound(format!("pool {} not found", pool_id))
                })?;

                self.pool_queue.insert(PoolQueueItem {
                    // Recently fetched pools do not need saving yet
                    pool,
                    quote_price: pool_quote.quote_price,
                    needs_saving: false,
                    usable: true,
                });
                self.latest = Some(pool_id);
            }
        }

        let loaded_pool_queue_item = match &self.tx_type {
            // For buys, the first pool will have the lowest quote
            TransactionType::TokensForNfts => self.pool_queue.pop_first(),
            // For sells, the last pool will have the highest quote
            TransactionType::NftsForTokens => self.pool_queue.pop_last(),
        };

        if let Some(_loaded_pool_queue_item) = &loaded_pool_queue_item {
            self.skip_next_pool_load = _loaded_pool_queue_item.pool.id != self.latest.unwrap();
        }

        Ok(loaded_pool_queue_item)
    }

    pub fn finalize_transaction(&mut self, response: &mut Response) -> Result<(), ContractError> {
        self.commit_messages(response)?;
        self.move_pools();

        Ok(())
    }

    pub fn get_transaction_events(&self) -> Vec<Event> {
        let mut events: Vec<Event> = vec![];
        for swap in self.swaps.iter() {
            events.push(swap.into());
        }
        for pool in self.pools_to_save.values() {
            events.push(
                pool.create_event(
                    "pool-swap-update",
                    vec![
                        "id",
                        "spot_price",
                        "nft_token_ids",
                        "total_tokens",
                        "is_active",
                    ],
                )
                .unwrap(),
            );
        }
        events
    }

    /// Swap NFTs to tokens directly with the specified pool
    pub fn direct_swap_nfts_for_tokens(
        &mut self,
        pool: Pool,
        nfts_to_swap: Vec<NftSwap>,
        swap_params: SwapParams,
    ) -> Result<(), ContractError> {
        let quote_price = pool.get_buy_quote(self.min_quote)?;
        if quote_price.is_none() {
            return Err(ContractError::NoQuoteForPool(format!(
                "pool {} cannot offer quote",
                pool.id
            )));
        }

        let mut pool_queue_item = PoolQueueItem {
            pool,
            quote_price: quote_price.unwrap(),
            needs_saving: false,
            usable: true,
        };
        let mut success: bool;

        for nft_swap in nfts_to_swap {
            (pool_queue_item, success) =
                self.process_swap(pool_queue_item, nft_swap, swap_params.robust)?;

            // If the swap failed, stop processing swaps
            if !success {
                break;
            }
        }
        if pool_queue_item.needs_saving {
            self.pools_to_save
                .insert(pool_queue_item.pool.id, pool_queue_item.pool);
        }

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
            let pool_queue_item_option = self.load_next_pool(storage)?;
            // No pools found, so return empty
            if pool_queue_item_option.is_none() {
                return Ok(());
            }
            let (pool_queue_item, success) = self.process_swap(
                pool_queue_item_option.unwrap(),
                nft_swap,
                swap_params.robust,
            )?;

            // If the swap failed, stop processing swaps
            if !success {
                break;
            }
            if pool_queue_item.usable {
                // If the swap was a success, and the quote price was updated, save into pool_queue
                self.pool_queue.insert(pool_queue_item);
            } else {
                // If the swap was a success, but the quote price was not updated,
                // withdraw from circulation by inserting into pools_to_save
                self.pools_to_save
                    .insert(pool_queue_item.pool.id, pool_queue_item.pool);
            }
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
        // Create a pool_queue_item map for tracking swap pools
        let mut pool_queue_item_map: BTreeMap<u64, PoolQueueItem> = BTreeMap::new();

        for pool_nfts in nfts_to_swap_for {
            // Check if pool is in pools_to_save map, indicating it cannot be involved in further swaps
            if self.pools_to_save.contains_key(&pool_nfts.pool_id) {
                if swap_params.robust {
                    continue;
                } else {
                    return Err(ContractError::InvalidPool(
                        "pool cannot be involved in further swaps".to_string(),
                    ));
                }
            }
            // If pool is not in pool_map, load it from storage
            if !pool_queue_item_map.contains_key(&pool_nfts.pool_id) {
                let pool_option = pools().may_load(storage, pool_nfts.pool_id)?;
                // If pool is not found, return error
                if pool_option.is_none() {
                    return Err(ContractError::PoolNotFound(format!(
                        "pool {} not found",
                        pool_nfts.pool_id
                    )));
                }
                // Create PoolQueueItem and insert into pool_queue_item_map
                let pool = pool_option.unwrap();
                let quote_price = pool.get_sell_quote(self.min_quote)?;
                if quote_price.is_none() {
                    if swap_params.robust {
                        continue;
                    } else {
                        return Err(ContractError::NoQuoteForPool(format!(
                            "pool {} cannot offer quote",
                            pool.id
                        )));
                    }
                }
                pool_queue_item_map.insert(
                    pool.id,
                    PoolQueueItem {
                        pool,
                        quote_price: quote_price.unwrap(),
                        needs_saving: false,
                        usable: true,
                    },
                );
            }

            // Iterate over all NFTs selected for the given pool
            for nft_swap in pool_nfts.nft_swaps {
                let pool_queue_item = pool_queue_item_map.remove(&pool_nfts.pool_id).unwrap();

                let (pool_queue_item, success) =
                    self.process_swap(pool_queue_item, nft_swap, swap_params.robust)?;

                // If the swap failed, stop processing swaps
                if !success {
                    break;
                }

                if pool_queue_item.usable {
                    // If the swap was a success, and the quote price was updated, save into pool_queue
                    pool_queue_item_map.insert(pool_queue_item.pool.id, pool_queue_item);
                } else {
                    // If the swap was a success, but the quote price was not updated,
                    // withdraw from circulation by inserting into pools_to_save
                    self.pools_to_save
                        .insert(pool_queue_item.pool.id, pool_queue_item.pool);
                }
            }
        }

        // Move all pools that need saving from pool_queue_item_map into pools_to_save
        for (_, pool_queue_item) in pool_queue_item_map.into_iter() {
            if pool_queue_item.needs_saving {
                self.pools_to_save
                    .insert(pool_queue_item.pool.id, pool_queue_item.pool);
            }
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
            let pool_queue_item_option = self.load_next_pool(storage)?;
            // No pools found, so return empty;
            if pool_queue_item_option.is_none() {
                return Ok(());
            }
            let pool_queue_item = pool_queue_item_option.unwrap();
            {
                // Grab first NFT from the pool
                let nft_token_id = pool_queue_item
                    .pool
                    .nft_token_ids
                    .first()
                    .unwrap()
                    .to_string();

                let (pool_queue_item, success) = self.process_swap(
                    pool_queue_item,
                    NftSwap {
                        nft_token_id,
                        token_amount,
                    },
                    swap_params.robust,
                )?;

                // If the swap failed, stop processing swaps
                if !success {
                    break;
                }

                if pool_queue_item.usable {
                    // If the swap was a success, and the quote price was updated, save into pool_queue
                    self.pool_queue.insert(pool_queue_item);
                } else {
                    // If the swap was a success, but the quote price was not updated,
                    // withdraw from circulation by inserting into pools_to_save
                    self.pools_to_save
                        .insert(pool_queue_item.pool.id, pool_queue_item.pool);
                }
            }
        }
        Ok(())
    }
}
