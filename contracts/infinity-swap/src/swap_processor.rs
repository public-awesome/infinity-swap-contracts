use crate::error::ContractError;
use crate::helpers::{get_nft_deposit, transfer_nft, transfer_token};
use crate::state::{
    buy_from_pool_quotes, nft_deposit_key, nft_deposits, pools, sell_to_pool_quotes, Pool, PoolId,
    PoolQuote, PoolType,
};

use core::cmp::Ordering;
use cosmwasm_std::{coin, Addr, Decimal, Order, StdResult, Storage, Uint128};
use infinity_shared::interface::{
    tx_fees_to_swap, NftOrder, Swap, SwapParamsInternal, TransactionType,
};
use sg1::fair_burn;
use sg721::RoyaltyInfo;
use sg_marketplace_common::calculate_nft_sale_fees;
use sg_std::{Response, NATIVE_DENOM};
use std::collections::{BTreeMap, BTreeSet};

/// A struct for tracking in memory pools that are involved in swaps
#[derive(Debug)]
pub struct PoolQueueItem {
    /// The pool object to handle a swap
    pub pool: Pool,
    /// The price at which to perform the swap
    pub quote_price: Uint128,
    /// Used to indicate whether the pool can continue to process swaps
    pub usable: bool,
    /// Number of swaps processed
    pub num_swaps: u32,
}

impl PoolQueueItem {
    fn needs_saving(&self) -> bool {
        self.num_swaps > 0
    }
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

type IterResults = StdResult<(u64, PoolQuote)>;

/// A struct for managing a series of swaps
pub struct SwapProcessor<'a> {
    /// The type of transaction (buy or sell)
    tx_type: TransactionType,
    /// The address of this contract
    contract: Addr,
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
    royalty: Option<RoyaltyInfo>,
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
        contract: Addr,
        collection: Addr,
        sender: Addr,
        remaining_balance: Uint128,
        seller_recipient: Addr,
        trading_fee_percent: Decimal,
        min_quote: Uint128,
        royalty: Option<RoyaltyInfo>,
        finder: Option<Addr>,
        developer: Option<Addr>,
    ) -> Self {
        Self {
            tx_type,
            contract,
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
    fn create_swap(
        &mut self,
        pool: &Pool,
        payment_amount: Uint128,
        nft_token_id: String,
    ) -> Result<Swap, ContractError> {
        // Subtract from received amount in the case of a buy
        if self.tx_type == TransactionType::UserSubmitsTokens {
            self.remaining_balance -= payment_amount;
        }

        // Set the addresses that will receive the NFT and token payment
        let (nft_recipient, token_recipient) = match &self.tx_type {
            TransactionType::UserSubmitsTokens => {
                (self.seller_recipient.clone(), pool.get_recipient())
            }
            TransactionType::UserSubmitsNfts => {
                (pool.get_recipient(), self.seller_recipient.clone())
            }
        };

        let tx_fees = calculate_nft_sale_fees(
            payment_amount,
            self.trading_fee_percent,
            &token_recipient,
            self.finder.as_ref(),
            Some(pool.finders_fee_percent),
            self.royalty.as_ref(),
        )?;

        let mut swap = tx_fees_to_swap(
            tx_fees,
            self.tx_type.clone(),
            &self.collection,
            &nft_token_id,
            payment_amount,
            &nft_recipient,
            &self.contract,
        );
        swap.set_data(PoolId(pool.id));

        Ok(swap)
    }

    /// Process a swap
    fn process_swap(
        &mut self,
        pool_queue_item: PoolQueueItem,
        nft_order: NftOrder,
        robust: bool,
    ) -> Result<(PoolQueueItem, bool), ContractError> {
        let mut pool_queue_item = pool_queue_item;

        // Manage pool assets for swap
        let result = match self.tx_type {
            TransactionType::UserSubmitsNfts => pool_queue_item
                .pool
                .sell_nft_to_pool(&nft_order, pool_queue_item.quote_price),
            TransactionType::UserSubmitsTokens => pool_queue_item
                .pool
                .buy_nft_from_pool(&nft_order, pool_queue_item.quote_price),
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
            nft_order.token_id,
        )?;

        // Reinvest tokens or NFTs if applicable
        if pool_queue_item.pool.pool_type == PoolType::Trade {
            if self.tx_type == TransactionType::UserSubmitsTokens
                && pool_queue_item.pool.reinvest_tokens
            {
                let index = swap
                    .token_payments
                    .iter()
                    .position(|p| p.label == "seller")
                    .unwrap();
                let reinvest_amount = swap.token_payments[index].amount;
                swap.token_payments.remove(index);
                pool_queue_item.pool.deposit_tokens(reinvest_amount)?;
            } else if self.tx_type == TransactionType::UserSubmitsNfts
                && pool_queue_item.pool.reinvest_nfts
            {
                swap.nft_payments[0].address = self.contract.to_string();
                pool_queue_item
                    .pool
                    .deposit_nfts(&vec![swap.nft_payments[0].token_id.clone()])?;
            }
        }
        self.swaps.push(swap);

        // Pool needs saving past this point
        pool_queue_item.num_swaps += 1;

        // Update the pool spot price
        let result = pool_queue_item.pool.update_spot_price(&self.tx_type);
        if result.is_err() {
            pool_queue_item.usable = false;
            return Ok((pool_queue_item, true));
        }
        let get_next_pool_quote = match self.tx_type {
            TransactionType::UserSubmitsNfts => {
                pool_queue_item.pool.get_sell_to_pool_quote(self.min_quote)
            }
            TransactionType::UserSubmitsTokens => {
                pool_queue_item.pool.get_buy_from_pool_quote(self.min_quote)
            }
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
            let nft_payment = swap.nft_payments.first().unwrap();
            transfer_nft(
                &nft_payment.token_id,
                &nft_payment.address,
                self.collection.as_ref(),
                response,
            )?;

            // Track token payments
            for token_payment in &swap.token_payments {
                let payment = token_payments
                    .entry(&token_payment.address)
                    .or_insert(Uint128::zero());
                *payment += token_payment.amount;
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
            if _pool_queue_item.needs_saving() {
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
                TransactionType::UserSubmitsNfts => sell_to_pool_quotes()
                    .idx
                    .collection_sell_price
                    .sub_prefix(self.collection.clone())
                    .range(storage, None, None, Order::Descending),
                TransactionType::UserSubmitsTokens => buy_from_pool_quotes()
                    .idx
                    .collection_buy_price
                    .sub_prefix(self.collection.clone())
                    .range(storage, None, None, Order::Ascending),
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
                    pool,
                    quote_price: pool_quote.quote_price,
                    usable: true,
                    num_swaps: 0,
                });
                self.latest = Some(pool_id);
            }
        }

        let loaded_pool_queue_item = match &self.tx_type {
            // For sells, the last pool will have the highest quote
            TransactionType::UserSubmitsNfts => self.pool_queue.pop_last(),
            // For buys, the first pool will have the lowest quote
            TransactionType::UserSubmitsTokens => self.pool_queue.pop_first(),
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

    /// Swap NFTs to tokens directly with the specified pool
    pub fn direct_swap_nfts_for_tokens(
        &mut self,
        pool: Pool,
        nft_orders: Vec<NftOrder>,
        swap_params: SwapParamsInternal,
    ) -> Result<(), ContractError> {
        let quote_price = pool.get_sell_to_pool_quote(self.min_quote)?;
        if quote_price.is_none() {
            return Err(ContractError::NoQuoteForPool(format!(
                "pool {} cannot offer quote",
                pool.id
            )));
        }

        let mut pool_queue_item = PoolQueueItem {
            pool,
            quote_price: quote_price.unwrap(),
            usable: true,
            num_swaps: 0,
        };
        let mut success: bool;

        for nft_order in nft_orders {
            (pool_queue_item, success) =
                self.process_swap(pool_queue_item, nft_order, swap_params.robust)?;

            // If the swap failed, stop processing swaps
            if !success {
                break;
            }
        }
        if pool_queue_item.needs_saving() {
            self.pools_to_save
                .insert(pool_queue_item.pool.id, pool_queue_item.pool);
        }

        Ok(())
    }

    /// Swap NFTs to tokens, using the best priced pools
    pub fn swap_nfts_for_tokens(
        &mut self,
        storage: &'a dyn Storage,
        nft_orders: Vec<NftOrder>,
        swap_params: SwapParamsInternal,
    ) -> Result<(), ContractError> {
        for nft_order in nft_orders {
            // Load best priced pool
            let pool_queue_item_option = self.load_next_pool(storage)?;
            // No pools found, so return empty
            if pool_queue_item_option.is_none() {
                return Ok(());
            }
            let (pool_queue_item, success) = self.process_swap(
                pool_queue_item_option.unwrap(),
                nft_order,
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
        nft_orders: Vec<NftOrder>,
        swap_params: SwapParamsInternal,
    ) -> Result<(), ContractError> {
        // Create a pool_queue_item map for tracking swap pools
        let mut pool_queue_item_map: BTreeMap<u64, PoolQueueItem> = BTreeMap::new();

        for nft_order in nft_orders {
            // Load nft_deposit
            let nft_deposit = nft_deposits().may_load(
                storage,
                nft_deposit_key(&self.collection, &nft_order.token_id),
            )?;
            if nft_deposit.is_none() {
                if swap_params.robust {
                    continue;
                } else {
                    return Err(ContractError::NftNotFound(nft_order.token_id));
                }
            }
            let nft_deposit = nft_deposit.unwrap();

            let mut pool_queue_item = match pool_queue_item_map.remove(&nft_deposit.pool_id) {
                Some(_pool_queue_item) => _pool_queue_item,
                None => {
                    let pool_option = pools().may_load(storage, nft_deposit.pool_id)?;
                    // If pool is not found, return error
                    if pool_option.is_none() {
                        return Err(ContractError::PoolNotFound(format!(
                            "pool {} not found",
                            nft_deposit.pool_id
                        )));
                    }
                    // Create PoolQueueItem and insert into pool_queue_item_map
                    let pool = pool_option.unwrap();

                    if pool.collection != self.collection {
                        return Err(ContractError::InvalidPool(
                            "pool does not belong to this collection".to_string(),
                        ));
                    }

                    let quote_price = pool.get_buy_from_pool_quote(self.min_quote)?;
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
                    PoolQueueItem {
                        pool,
                        quote_price: quote_price.unwrap(),
                        usable: true,
                        num_swaps: 0,
                    }
                }
            };

            if !pool_queue_item.usable {
                if swap_params.robust {
                    break;
                } else {
                    return Err(ContractError::SwapError(
                        "unable to process swap".to_string(),
                    ));
                }
            }

            let (_pool_queue_item, success) =
                self.process_swap(pool_queue_item, nft_order, swap_params.robust)?;
            pool_queue_item = _pool_queue_item;

            // If the swap failed, stop processing swaps
            if !success {
                if swap_params.robust {
                    break;
                } else {
                    return Err(ContractError::SwapError(
                        "unable to process swap".to_string(),
                    ));
                }
            }

            pool_queue_item_map.insert(pool_queue_item.pool.id, pool_queue_item);
        }

        // Move all pools that need saving from pool_queue_item_map into pools_to_save
        for (_, pool_queue_item) in pool_queue_item_map.into_iter() {
            if pool_queue_item.needs_saving() {
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
        orders: Vec<Uint128>,
        swap_params: SwapParamsInternal,
    ) -> Result<(), ContractError> {
        for order in orders {
            // Load best priced pool
            let pool_queue_item_option = self.load_next_pool(storage)?;
            // No pools found, so return empty;
            if pool_queue_item_option.is_none() {
                return Ok(());
            }
            let pool_queue_item = pool_queue_item_option.unwrap();
            {
                // Grab first NFT from the pool
                let nft_deposit =
                    get_nft_deposit(storage, pool_queue_item.pool.id, pool_queue_item.num_swaps)?;
                if nft_deposit.is_none() {
                    return Err(ContractError::SwapError(format!(
                        "pool {} does not own any NFTs",
                        pool_queue_item.pool.id
                    )));
                }

                let (pool_queue_item, success) = self.process_swap(
                    pool_queue_item,
                    NftOrder {
                        token_id: nft_deposit.unwrap().token_id,
                        amount: order,
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
