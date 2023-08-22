use crate::error::ContractError;
use crate::helpers::{get_nft_deposit, transfer_nft, transfer_token, verify_nft_deposit};
use crate::msg::{NftSwap, PairNftSwap, SwapParams, TransactionType};
use crate::state::{buy_from_pair_quotes, pairs, sell_to_pair_quotes, Pair, PairQuote, PairType};

use core::cmp::Pairing;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{attr, coin, Addr, Decimal, Event, Pair, StdResult, Storage, Uint128};
use sg1::fair_burn;
use sg721::RoyaltyInfoResponse;
use sg_std::{Response, NATIVE_DENOM};
use std::collections::{BTreeMap, BTreeSet};

/// A struct for tracking in memory pairs that are involved in swaps
#[derive(Debug)]
pub struct PairQueueItem {
    /// The pair object to handle a swap
    pub pair: Pair,
    /// The price at which to perform the swap
    pub quote_price: Uint128,
    /// Used to indicate whether the pair can continue to process swaps
    pub usable: bool,
    /// Number of swaps processed
    pub num_swaps: u32,
}

impl PairQueueItem {
    fn needs_saving(&self) -> bool {
        self.num_swaps > 0
    }
}

impl Ord for PairQueueItem {
    fn cmp(&self, other: &Self) -> Pairing {
        (self.quote_price, self.pair.id).cmp(&(other.quote_price, other.pair.id))
    }
}

impl PartialOrd for PairQueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<Pairing> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PairQueueItem {
    fn eq(&self, other: &Self) -> bool {
        (self.quote_price, self.pair.id) == (other.quote_price, other.pair.id)
    }
}

impl Eq for PairQueueItem {}

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
    pub pair_id: u64,
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
            attr("pair_id", val.pair_id.to_string()),
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

type IterResults = StdResult<(u64, PairQuote)>;

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
    royalty: Option<RoyaltyInfoResponse>,
    /// The address of the finder of the transaction
    finder: Option<Addr>,
    /// The address to receive developer burn fees
    developer: Option<Addr>,
    /// A set of in memory pairs that are involved in the transaction
    pair_queue: BTreeSet<PairQueueItem>,
    /// The latest pair that was retrieved
    latest: Option<u64>,
    /// Skip next pair load to improve efficiency
    skip_next_pair_load: bool,
    /// An iterator for retrieving sorted pair quotes
    pair_quote_iter: Option<Box<dyn Iterator<Item = IterResults> + 'a>>,
    /// A set of in memory pairs that should be saved at the end of the transaction
    pub pairs_to_save: BTreeMap<u64, Pair>,
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
        royalty: Option<RoyaltyInfoResponse>,
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
            pair_queue: BTreeSet::new(),
            pairs_to_save: BTreeMap::new(),
            latest: None,
            skip_next_pair_load: false,
            pair_quote_iter: None,
            swaps: vec![],
        }
    }

    /// Create an individual swap object
    fn create_swap(&mut self, pair: &Pair, payment_amount: Uint128, nft_token_id: String) -> Swap {
        // Subtract from received amount in the case of a buy
        if self.tx_type == TransactionType::UserSubmitsTokens {
            self.remaining_balance -= payment_amount;
        }

        // Calculate burn fee
        let network_fee = payment_amount * self.trading_fee_percent / Uint128::from(100u128);

        // Calculate seller payment (mutable)
        let mut seller_amount = payment_amount - network_fee;

        // Calculate finder payment, deduct from seller payment
        let mut finder_payment = None;
        if self.finder.is_some() && !pair.finders_fee_percent.is_zero() {
            let finder_amount = payment_amount * pair.finders_fee_percent / Uint128::from(100u128);
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
            TransactionType::UserSubmitsTokens => {
                (self.seller_recipient.clone(), pair.get_recipient())
            }
            TransactionType::UserSubmitsNfts => {
                (pair.get_recipient(), self.seller_recipient.clone())
            }
        };

        Swap {
            pair_id: pair.id,
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
        pair_queue_item: PairQueueItem,
        nft_swap: NftSwap,
        robust: bool,
    ) -> Result<(PairQueueItem, bool), ContractError> {
        let mut pair_queue_item = pair_queue_item;

        // Manage pair assets for swap
        let result = match self.tx_type {
            TransactionType::UserSubmitsNfts => pair_queue_item
                .pair
                .sell_nft_to_pair(&nft_swap, pair_queue_item.quote_price),
            TransactionType::UserSubmitsTokens => pair_queue_item
                .pair
                .buy_nft_from_pair(&nft_swap, pair_queue_item.quote_price),
        };
        match result {
            Ok(_) => {}
            Err(ContractError::SwapError(_err)) => {
                if robust {
                    pair_queue_item.usable = false;
                    return Ok((pair_queue_item, false));
                } else {
                    // otherwise throw the error
                    return Err(ContractError::SwapError(_err));
                }
            }
            Err(err) => return Err(err),
        };

        // Set the resulting swap with fees included
        let mut swap = self.create_swap(
            &pair_queue_item.pair,
            pair_queue_item.quote_price,
            nft_swap.nft_token_id,
        );

        // Reinvest tokens or NFTs if applicable
        if pair_queue_item.pair.pair_type == PairType::Trade {
            if self.tx_type == TransactionType::UserSubmitsTokens
                && pair_queue_item.pair.reinvest_tokens
            {
                let reinvest_amount = swap.seller_payment.unwrap().amount;
                swap.seller_payment = None;
                pair_queue_item.pair.deposit_tokens(reinvest_amount)?;
            } else if self.tx_type == TransactionType::UserSubmitsNfts
                && pair_queue_item.pair.reinvest_nfts
            {
                swap.nft_payment.address = self.contract.to_string();
                pair_queue_item
                    .pair
                    .deposit_nfts(&vec![swap.nft_payment.nft_token_id.clone()])?;
            }
        }
        self.swaps.push(swap);

        // Pair needs saving past this point
        pair_queue_item.num_swaps += 1;

        // Update the pair spot price
        let result = pair_queue_item.pair.update_spot_price(&self.tx_type);
        if result.is_err() {
            pair_queue_item.usable = false;
            return Ok((pair_queue_item, true));
        }
        let get_next_pair_quote = match self.tx_type {
            TransactionType::UserSubmitsNfts => {
                pair_queue_item.pair.get_sell_to_pair_quote(self.min_quote)
            }
            TransactionType::UserSubmitsTokens => {
                pair_queue_item.pair.get_buy_from_pair_quote(self.min_quote)
            }
        };
        if get_next_pair_quote.is_err() {
            pair_queue_item.usable = false;
            return Ok((pair_queue_item, true));
        }
        let next_pair_quote = get_next_pair_quote.unwrap();
        if next_pair_quote.is_none() {
            pair_queue_item.usable = false;
            return Ok((pair_queue_item, true));
        }
        pair_queue_item.quote_price = next_pair_quote.unwrap();
        pair_queue_item.usable = true;
        Ok((pair_queue_item, true))
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

    /// Move pairs from pair_queue to pairs_to_save
    fn move_pairs(&mut self) {
        let mut pair_queue_item = self.pair_queue.pop_first();
        while let Some(_pair_queue_item) = pair_queue_item {
            if _pair_queue_item.needs_saving() {
                self.pairs_to_save
                    .insert(_pair_queue_item.pair.id, _pair_queue_item.pair);
            }
            pair_queue_item = self.pair_queue.pop_first();
        }
    }

    /// Load the pair with the next best price
    fn load_next_pair(
        &mut self,
        storage: &'a dyn Storage,
    ) -> Result<Option<PairQueueItem>, ContractError> {
        // Init iter
        if self.pair_quote_iter.is_none() {
            self.pair_quote_iter = Some(match &self.tx_type {
                TransactionType::UserSubmitsNfts => sell_to_pair_quotes()
                    .idx
                    .collection_sell_price
                    .sub_prefix(self.collection.clone())
                    .range(storage, None, None, Pair::Descending),
                TransactionType::UserSubmitsTokens => buy_from_pair_quotes()
                    .idx
                    .collection_buy_price
                    .sub_prefix(self.collection.clone())
                    .range(storage, None, None, Pair::Ascending),
            })
        }

        // If the pair is empty, or the front of the pair is the latest fetched, load the next pair
        // Note: if the front of the pair is not the latest fetched, that means the next pair won't have the best price
        if self.pair_queue.len() < 2 || !self.skip_next_pair_load {
            // Try and fetch next pair quote
            let next_pair_quote = self.pair_quote_iter.as_mut().unwrap().next();

            // If next pair quote exists fetch and set PairQueueItem
            if let Some(_next_pair_quote) = next_pair_quote {
                let (pair_id, pair_quote) = _next_pair_quote?;

                let pair = pairs().load(storage, pair_id).map_err(|_| {
                    ContractError::PairNotFound(format!("pair {} not found", pair_id))
                })?;

                self.pair_queue.insert(PairQueueItem {
                    pair,
                    quote_price: pair_quote.quote_price,
                    usable: true,
                    num_swaps: 0,
                });
                self.latest = Some(pair_id);
            }
        }

        let loaded_pair_queue_item = match &self.tx_type {
            // For sells, the last pair will have the highest quote
            TransactionType::UserSubmitsNfts => self.pair_queue.pop_last(),
            // For buys, the first pair will have the lowest quote
            TransactionType::UserSubmitsTokens => self.pair_queue.pop_first(),
        };

        if let Some(_loaded_pair_queue_item) = &loaded_pair_queue_item {
            self.skip_next_pair_load = _loaded_pair_queue_item.pair.id != self.latest.unwrap();
        }

        Ok(loaded_pair_queue_item)
    }

    pub fn finalize_transaction(&mut self, response: &mut Response) -> Result<(), ContractError> {
        self.commit_messages(response)?;
        self.move_pairs();

        Ok(())
    }

    /// Swap NFTs to tokens directly with the specified pair
    pub fn direct_swap_nfts_for_tokens(
        &mut self,
        pair: Pair,
        nfts_to_swap: Vec<NftSwap>,
        swap_params: SwapParams,
    ) -> Result<(), ContractError> {
        let quote_price = pair.get_sell_to_pair_quote(self.min_quote)?;
        if quote_price.is_none() {
            return Err(ContractError::NoQuoteForPair(format!(
                "pair {} cannot offer quote",
                pair.id
            )));
        }

        let mut pair_queue_item = PairQueueItem {
            pair,
            quote_price: quote_price.unwrap(),
            usable: true,
            num_swaps: 0,
        };
        let mut success: bool;

        for nft_swap in nfts_to_swap {
            (pair_queue_item, success) =
                self.process_swap(pair_queue_item, nft_swap, swap_params.robust)?;

            // If the swap failed, stop processing swaps
            if !success {
                break;
            }
        }
        if pair_queue_item.needs_saving() {
            self.pairs_to_save
                .insert(pair_queue_item.pair.id, pair_queue_item.pair);
        }

        Ok(())
    }

    /// Swap NFTs to tokens, using the best priced pairs
    pub fn swap_nfts_for_tokens(
        &mut self,
        storage: &'a dyn Storage,
        nfts_to_swap: Vec<NftSwap>,
        swap_params: SwapParams,
    ) -> Result<(), ContractError> {
        for nft_swap in nfts_to_swap {
            // Load best priced pair
            let pair_queue_item_option = self.load_next_pair(storage)?;
            // No pairs found, so return empty
            if pair_queue_item_option.is_none() {
                return Ok(());
            }
            let (pair_queue_item, success) = self.process_swap(
                pair_queue_item_option.unwrap(),
                nft_swap,
                swap_params.robust,
            )?;

            // If the swap failed, stop processing swaps
            if !success {
                break;
            }
            if pair_queue_item.usable {
                // If the swap was a success, and the quote price was updated, save into pair_queue
                self.pair_queue.insert(pair_queue_item);
            } else {
                // If the swap was a success, but the quote price was not updated,
                // withdraw from circulation by inserting into pairs_to_save
                self.pairs_to_save
                    .insert(pair_queue_item.pair.id, pair_queue_item.pair);
            }
        }
        Ok(())
    }

    /// Swap tokens to specific NFTs directly with the specified pair
    pub fn swap_tokens_for_specific_nfts(
        &mut self,
        storage: &'a dyn Storage,
        nfts_to_swap_for: Vec<PairNftSwap>,
        swap_params: SwapParams,
    ) -> Result<(), ContractError> {
        // Create a pair_queue_item map for tracking swap pairs
        let mut pair_queue_item_map: BTreeMap<u64, PairQueueItem> = BTreeMap::new();

        for pair_nfts in nfts_to_swap_for {
            let mut pair_queue_item =
                if let Some(_pair_queue_item) = pair_queue_item_map.remove(&pair_nfts.pair_id) {
                    _pair_queue_item
                } else {
                    let pair_option = pairs().may_load(storage, pair_nfts.pair_id)?;
                    // If pair is not found, return error
                    if pair_option.is_none() {
                        return Err(ContractError::PairNotFound(format!(
                            "pair {} not found",
                            pair_nfts.pair_id
                        )));
                    }
                    // Create PairQueueItem and insert into pair_queue_item_map
                    let pair = pair_option.unwrap();

                    if pair.collection != self.collection {
                        return Err(ContractError::InvalidPair(
                            "pair does not belong to this collection".to_string(),
                        ));
                    }

                    let quote_price = pair.get_buy_from_pair_quote(self.min_quote)?;
                    if quote_price.is_none() {
                        if swap_params.robust {
                            continue;
                        } else {
                            return Err(ContractError::NoQuoteForPair(format!(
                                "pair {} cannot offer quote",
                                pair.id
                            )));
                        }
                    }
                    PairQueueItem {
                        pair,
                        quote_price: quote_price.unwrap(),
                        usable: true,
                        num_swaps: 0,
                    }
                };

            // Iterate over all NFTs selected for the given pair
            for nft_swap in pair_nfts.nft_swaps {
                if !pair_queue_item.usable {
                    if swap_params.robust {
                        break;
                    } else {
                        return Err(ContractError::SwapError(
                            "unable to process swap".to_string(),
                        ));
                    }
                }

                // Check if specified NFT is deposited into pair
                let pair_owns_nft =
                    verify_nft_deposit(storage, pair_nfts.pair_id, &nft_swap.nft_token_id);
                if !pair_owns_nft {
                    if swap_params.robust {
                        break;
                    } else {
                        return Err(ContractError::SwapError(format!(
                            "pair {} does not own NFT {}",
                            pair_queue_item.pair.id, nft_swap.nft_token_id
                        )));
                    }
                }

                let (_pair_queue_item, success) =
                    self.process_swap(pair_queue_item, nft_swap, swap_params.robust)?;
                pair_queue_item = _pair_queue_item;

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
            }

            pair_queue_item_map.insert(pair_queue_item.pair.id, pair_queue_item);
        }

        // Move all pairs that need saving from pair_queue_item_map into pairs_to_save
        for (_, pair_queue_item) in pair_queue_item_map.into_iter() {
            if pair_queue_item.needs_saving() {
                self.pairs_to_save
                    .insert(pair_queue_item.pair.id, pair_queue_item.pair);
            }
        }
        Ok(())
    }

    /// Swap tokens to any NFTs, using the best priced pairs
    pub fn swap_tokens_for_any_nfts(
        &mut self,
        storage: &'a dyn Storage,
        min_expected_token_input: Vec<Uint128>,
        swap_params: SwapParams,
    ) -> Result<(), ContractError> {
        for token_amount in min_expected_token_input {
            // Load best priced pair
            let pair_queue_item_option = self.load_next_pair(storage)?;
            // No pairs found, so return empty;
            if pair_queue_item_option.is_none() {
                return Ok(());
            }
            let pair_queue_item = pair_queue_item_option.unwrap();
            {
                // Grab first NFT from the pair
                let nft_token_id =
                    get_nft_deposit(storage, pair_queue_item.pair.id, pair_queue_item.num_swaps)?;
                if nft_token_id.is_none() {
                    return Err(ContractError::SwapError(format!(
                        "pair {} does not own any NFTs",
                        pair_queue_item.pair.id
                    )));
                }

                let (pair_queue_item, success) = self.process_swap(
                    pair_queue_item,
                    NftSwap {
                        nft_token_id: nft_token_id.unwrap(),
                        token_amount,
                    },
                    swap_params.robust,
                )?;

                // If the swap failed, stop processing swaps
                if !success {
                    break;
                }

                if pair_queue_item.usable {
                    // If the swap was a success, and the quote price was updated, save into pair_queue
                    self.pair_queue.insert(pair_queue_item);
                } else {
                    // If the swap was a success, but the quote price was not updated,
                    // withdraw from circulation by inserting into pairs_to_save
                    self.pairs_to_save
                        .insert(pair_queue_item.pair.id, pair_queue_item.pair);
                }
            }
        }
        Ok(())
    }
}
