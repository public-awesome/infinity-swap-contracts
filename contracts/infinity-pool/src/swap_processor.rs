use crate::error::ContractError;
use crate::helpers::{transfer_nft, transfer_token};
use crate::msg::{NftSwap, PoolNftSwap, SwapParams};
use crate::state::{buy_pool_quotes, pools, sell_pool_quotes, Pool, PoolType};

use core::cmp::Ordering;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{coin, Addr, StdResult, Storage, Uint128};
use cosmwasm_std::{Decimal, Order};
use sg1::fair_burn;
use sg721::RoyaltyInfoResponse;
use sg_std::{Response, NATIVE_DENOM};
use std::collections::{BTreeMap, BTreeSet};

pub struct PoolPair {
    pub needs_saving: bool,
    pub pool: Pool,
}

impl Ord for PoolPair {
    fn cmp(&self, other: &Self) -> Ordering {
        self.pool.spot_price.cmp(&other.pool.spot_price)
    }
}

impl PartialOrd for PoolPair {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for PoolPair {
    fn eq(&self, other: &Self) -> bool {
        self.pool.spot_price == other.pool.spot_price
    }
}

impl Eq for PoolPair {}

#[cw_serde]
pub enum TransactionType {
    Sell,
    Buy,
}
#[cw_serde]
pub struct TokenPayment {
    pub amount: Uint128,
    pub address: String,
}

#[cw_serde]
pub struct NftPayment {
    pub nft_token_id: String,
    pub address: String,
}

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

pub struct SwapProcessor<'a> {
    pub swaps: Vec<Swap>,
    pub collection: Addr,
    pub seller_recipient: Addr,
    pub trading_fee_percent: Decimal,
    pub royalty: Option<RoyaltyInfoResponse>,
    pub finder: Option<Addr>,
    pub developer: Option<Addr>,
    pub pool_set: BTreeSet<PoolPair>,
    pub latest: Option<u64>,
    pub pool_quote_iter: Option<Box<dyn Iterator<Item = StdResult<u64>> + 'a>>,
}

impl<'a> SwapProcessor<'a> {
    pub fn new(
        collection: Addr,
        seller_recipient: Addr,
        trading_fee_percent: Decimal,
        royalty: Option<RoyaltyInfoResponse>,
        finder: Option<Addr>,
        developer: Option<Addr>,
    ) -> Self {
        Self {
            swaps: vec![],
            collection,
            seller_recipient,
            trading_fee_percent,
            royalty,
            finder,
            developer,
            pool_set: BTreeSet::new(),
            latest: None,
            pool_quote_iter: None,
        }
    }

    fn create_swap(
        &mut self,
        pool: &Pool,
        payment_amount: Uint128,
        nft_token_id: String,
        tx_type: TransactionType,
    ) -> Swap {
        let network_fee = payment_amount * self.trading_fee_percent / Uint128::from(100u128);
        let mut seller_amount = payment_amount - network_fee;

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

        let (nft_recipient, token_recipient) = match tx_type {
            TransactionType::Buy => (self.seller_recipient.clone(), pool.get_recipient()),
            TransactionType::Sell => (pool.get_recipient(), self.seller_recipient.clone()),
        };

        Swap {
            pool_id: pool.id,
            transaction_type: tx_type,
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

    pub fn process_swap(
        &mut self,
        pool: &mut Pool,
        nft_swap: NftSwap,
        tx_type: TransactionType,
    ) -> Result<(), ContractError> {
        let sale_price = match tx_type {
            TransactionType::Buy => pool.buy_nft_from_pool(&nft_swap)?,
            TransactionType::Sell => pool.sell_nft_to_pool(&nft_swap)?,
        };
        let mut swap = self.create_swap(pool, sale_price, nft_swap.nft_token_id, tx_type);
        // pool.update_spot_price()?;

        // if pool.pool_type == PoolType::Trade {
        //     if tx_type == TransactionType::Buy && pool.reinvest_tokens {
        //         let reinvest_amount = swap.seller_payment.unwrap().amount;
        //         swap.seller_payment = None;
        //         pool.deposit_tokens(reinvest_amount)?;
        //     } else if tx_type == TransactionType::Sell && pool.reinvest_nfts {
        //         let reinvest_nft_token_id = swap.nft_payment.unwrap().nft_token_id;
        //         swap.nft_payment = None;
        //         pool.deposit_nfts(&vec![reinvest_nft_token_id])?;
        //     }
        // }

        self.swaps.push(swap);
        Ok(())
    }

    pub fn commit_messages(&self, response: &mut Response) -> Result<(), ContractError> {
        if self.swaps.is_empty() {
            return Err(ContractError::SwapError("no swaps found".to_string()));
        }

        let mut total_network_fee = Uint128::zero();
        let mut token_payments = BTreeMap::new();

        for swap in self.swaps.iter() {
            total_network_fee += swap.network_fee;

            if let Some(_finder_payment) = &swap.finder_payment {
                let payment = token_payments
                    .entry(&_finder_payment.address)
                    .or_insert(Uint128::zero());
                *payment += _finder_payment.amount;
            }

            if let Some(_royalty_payment) = &swap.royalty_payment {
                let payment = token_payments
                    .entry(&_royalty_payment.address)
                    .or_insert(Uint128::zero());
                *payment += _royalty_payment.amount;
            }

            if let Some(_seller_payment) = &swap.seller_payment {
                let payment = token_payments
                    .entry(&_seller_payment.address)
                    .or_insert(Uint128::zero());
                *payment += _seller_payment.amount;
            }

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

        for token_payment in token_payments {
            transfer_token(
                coin(token_payment.1.u128(), NATIVE_DENOM),
                &token_payment.0.to_string(),
                response,
            )?;
        }

        Ok(())
    }

    pub fn load_next_pool(
        &mut self,
        storage: &dyn Storage,
    ) -> Result<Option<PoolPair>, ContractError> {
        if self.pool_set.is_empty() || Some(self.pool_set.first().unwrap().pool.id) == self.latest {
            let pool_id = self.pool_quote_iter.as_mut().unwrap().next().unwrap()?;

            let pool = pools()
                .load(storage, pool_id)
                .map_err(|_| ContractError::InvalidPool("pool does not exist".to_string()))?;

            self.pool_set.insert(PoolPair {
                needs_saving: false,
                pool,
            });
            self.latest = Some(pool_id);
        }

        Ok(self.pool_set.pop_first())
    }

    pub fn direct_swap_nfts_for_tokens(
        &mut self,
        pool: Pool,
        nfts_to_swap: Vec<NftSwap>,
        swap_params: SwapParams,
    ) -> Result<(), ContractError> {
        let mut pool = pool;
        {
            for nft_swap in nfts_to_swap {
                let result = self.process_swap(&mut pool, nft_swap, TransactionType::Sell);
                match result {
                    Ok(_) => {}
                    Err(ContractError::SwapError(_err)) => {
                        if swap_params.robust {
                            break;
                        } else {
                            return Err(ContractError::SwapError(_err));
                        }
                    }
                    Err(_err) => return Err(_err),
                }
            }
        }
        self.pool_set.insert(PoolPair {
            needs_saving: true,
            pool,
        });
        Ok(())
    }

    pub fn swap_nfts_for_tokens(
        &mut self,
        storage: &'a dyn Storage,
        nfts_to_swap: Vec<NftSwap>,
        swap_params: SwapParams,
    ) -> Result<(), ContractError> {
        self.pool_quote_iter = Some(
            sell_pool_quotes()
                .idx
                .collection_sell_price
                .sub_prefix(self.collection.clone())
                .keys(storage, None, None, Order::Descending),
        );

        for nft_swap in nfts_to_swap {
            let pool_pair_option = self.load_next_pool(storage)?;
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
                        if swap_params.robust {
                            return Ok(());
                        } else {
                            return Err(ContractError::SwapError(_err));
                        }
                    }
                    Err(_err) => return Err(_err),
                }
            }
            pool_pair.needs_saving = true;
            self.pool_set.insert(pool_pair);
        }
        Ok(())
    }

    pub fn swap_tokens_for_specific_nfts(
        &mut self,
        storage: &'a dyn Storage,
        nfts_to_swap_for: Vec<PoolNftSwap>,
        swap_params: SwapParams,
    ) -> Result<(), ContractError> {
        let mut pool_map: BTreeMap<u64, Pool> = BTreeMap::new();

        for pool_nfts in nfts_to_swap_for {
            let mut pool_option = pool_map.remove(&pool_nfts.pool_id);
            if pool_option.is_none() {
                pool_option = pools().may_load(storage, pool_nfts.pool_id)?;
            }
            if pool_option.is_none() {
                return Err(ContractError::InvalidPool("pool not found".to_string()));
            }
            let mut pool = pool_option.unwrap();

            for nft_swap in pool_nfts.nft_swaps {
                let result = self.process_swap(&mut pool, nft_swap, TransactionType::Buy);
                match result {
                    Ok(_) => {}
                    Err(ContractError::SwapError(_err)) => {
                        if swap_params.robust {
                            break;
                        } else {
                            return Err(ContractError::SwapError(_err));
                        }
                    }
                    Err(_err) => return Err(_err),
                }
            }
            pool_map.insert(pool.id, pool);
        }
        for (_, pool) in pool_map {
            self.pool_set.insert(PoolPair {
                needs_saving: true,
                pool,
            });
        }
        Ok(())
    }

    pub fn swap_tokens_for_any_nfts(
        &mut self,
        storage: &'a dyn Storage,
        min_expected_token_input: Vec<Uint128>,
        swap_params: SwapParams,
    ) -> Result<(), ContractError> {
        self.pool_quote_iter = Some(
            buy_pool_quotes()
                .idx
                .collection_buy_price
                .sub_prefix(self.collection.clone())
                .keys(storage, None, None, Order::Ascending),
        );

        for token_amount in min_expected_token_input {
            let pool_pair_option = self.load_next_pool(storage)?;
            if pool_pair_option.is_none() {
                return Ok(());
            }
            let mut pool_pair = pool_pair_option.unwrap();
            {
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
                        if swap_params.robust {
                            return Ok(());
                        } else {
                            return Err(ContractError::SwapError(_err));
                        }
                    }
                    Err(_err) => return Err(_err),
                }
            }
            pool_pair.needs_saving = true;
            self.pool_set.insert(pool_pair);
        }
        Ok(())
    }
}
