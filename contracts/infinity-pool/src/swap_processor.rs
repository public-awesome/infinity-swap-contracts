use crate::error::ContractError;
use crate::helpers::{
    get_next_pool_counter, get_pool_attributes, only_owner, remove_pool, save_pool, transfer_nft,
    transfer_token,
};
use crate::msg::{ExecuteMsg, PoolNfts, QueryOptions, SwapNft, SwapParams};
use crate::query::{query_pool_quotes_by_buy_price, query_pool_quotes_by_sell_price};
use crate::state::{
    buy_pool_quotes, pools, sell_pool_quotes, BondingCurve, Pool, PoolQuote, PoolType, CONFIG,
};

use core::cmp::Ordering;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{coin, Addr, DepsMut, Env, Event, MessageInfo, StdResult, Storage, Uint128};
use cosmwasm_std::{entry_point, Decimal, Deps, Order};
use cw_utils::{maybe_addr, must_pay, nonpayable};
use sg1::fair_burn;
use sg721::RoyaltyInfoResponse;
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg_marketplace::msg::{ParamsResponse, QueryMsg as MarketplaceQueryMsg};
use sg_std::{Response, NATIVE_DENOM};
use std::collections::{BTreeMap, BTreeSet};

pub enum TransactionType {
    Sell,
    Buy,
}

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
        self.pool.spot_price == &other.pool.spot_price
    }
}

impl Eq for PoolPair {}

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
    pub pool_type: PoolType,
    pub spot_price: Uint128,
    pub network_fee: Uint128,
    pub royalty_payment: Option<TokenPayment>,
    pub nft_payment: NftPayment,
    pub seller_payment: TokenPayment,
}

pub struct SwapProcessor<'a> {
    pub swaps: Vec<Swap>,
    pub collection: Addr,
    pub seller_recipient: Addr,
    pub trading_fee_percent: Decimal,
    pub royalty: Option<RoyaltyInfoResponse>,
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
    ) -> Self {
        Self {
            swaps: vec![],
            collection,
            seller_recipient,
            trading_fee_percent,
            royalty,
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
        nft_recipient: &Addr,
        token_recipient: &Addr,
    ) -> Swap {
        let network_fee = payment_amount * self.trading_fee_percent / Uint128::from(100u128);
        let mut seller_amount = payment_amount - network_fee;

        // finders fee?

        let mut royalty_payment = None;
        if let Some(_royalty) = &self.royalty {
            let royalty_amount = payment_amount * _royalty.share;
            seller_amount -= royalty_amount;
            royalty_payment = Some(TokenPayment {
                amount: royalty_amount,
                address: _royalty.payment_address.clone(),
            });
        }

        Swap {
            pool_id: pool.id,
            pool_type: pool.pool_type.clone(),
            spot_price: payment_amount,
            network_fee,
            royalty_payment,
            nft_payment: NftPayment {
                nft_token_id,
                address: nft_recipient.to_string(),
            },
            seller_payment: TokenPayment {
                amount: seller_amount,
                address: token_recipient.to_string(),
            },
        }
    }

    pub fn process_swap(
        &mut self,
        pool: &mut Pool,
        swap_nft: SwapNft,
    ) -> Result<(), ContractError> {
        let sale_price = pool.sell_nft_to_pool(&swap_nft);
        match sale_price {
            Ok(_sale_price) => {
                let swap = self.create_swap(
                    &pool,
                    _sale_price,
                    swap_nft.nft_token_id,
                    &pool.get_recipient(),
                    &pool.get_recipient(),
                );
                self.swaps.push(swap);
                Ok(())
            }
            Err(_err) => return Err(_err),
        }
    }

    pub fn commit_messages(&self, response: &mut Response) -> Result<(), ContractError> {
        let mut total_network_fee = Uint128::zero();
        let mut token_payments = BTreeMap::new();

        for swap in self.swaps.iter() {
            total_network_fee += swap.network_fee;

            if let Some(_royalty_payment) = &swap.royalty_payment {
                let payment = token_payments
                    .entry(&_royalty_payment.address)
                    .or_insert(Uint128::zero());
                *payment += _royalty_payment.amount;
            }
            let payment = token_payments
                .entry(&swap.seller_payment.address)
                .or_insert(Uint128::zero());
            *payment += swap.seller_payment.amount;

            transfer_nft(
                &swap.nft_payment.nft_token_id,
                &swap.nft_payment.address,
                &self.collection.to_string(),
                response,
            )?;
        }

        fair_burn(total_network_fee.u128(), None, response);

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
        swap_nfts: Vec<SwapNft>,
        swap_params: SwapParams,
    ) -> Result<(), ContractError> {
        let mut pool = pool;
        {
            for swap_nft in swap_nfts {
                let result = self.process_swap(&mut pool, swap_nft);
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
        swap_nfts: Vec<SwapNft>,
        swap_params: SwapParams,
    ) -> Result<(), ContractError> {
        self.pool_quote_iter = Some(
            sell_pool_quotes()
                .idx
                .collection_sell_price
                .sub_prefix(self.collection.clone())
                .keys(storage, None, None, Order::Descending),
        );

        for swap_nft in swap_nfts {
            let pool_pair_option = self.load_next_pool(storage)?;
            if pool_pair_option == None {
                return Ok(());
            }
            let mut pool_pair = pool_pair_option.unwrap();
            {
                let result = self.process_swap(&mut pool_pair.pool, swap_nft);
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

    // pub fn process_swap(
    //     &mut self,
    //     pool: &mut Pool,
    //     nft_token_id: String,
    // ) -> Result<(), ContractError> {
    //     // pool.buy_nft_from_pool(nft_token_id.clone())?;
    //     self.calc_swap_fees(pool.spot_price, nft_token_id)?;

    //     Ok(())
    // }

    // pub fn swap_token_for_specific_nfts(
    //     &mut self,
    //     deps: DepsMut,
    //     specific_nfts: Vec<PoolNfts>,
    //     max_expected_token_input: Uint128,
    // ) -> Result<(), ContractError> {
    //     if specific_nfts.len() == 0 {
    //         return Err(ContractError::InvalidInput(
    //             "specific_nfts.len() must be greater than 0".to_string(),
    //         ));
    //     }

    //     let mut remaining_balance = max_expected_token_input.clone();

    //     for pool_nft in specific_nfts {
    //         if pool_nft.nft_token_ids.len() == 0 {
    //             return Err(ContractError::InvalidInput(format!(
    //                 "no nfts selected for pool_id {}",
    //                 pool_nft.pool_id
    //             )));
    //         }

    //         let mut pool = pools()
    //             .load(deps.storage, pool_nft.pool_id)
    //             .map_err(|_| ContractError::InvalidInput("pool does not exist".to_string()))?;

    //         if !pool.can_sell_nfts() {
    //             return Err(ContractError::InvalidPool(
    //                 "pool cannot sell NFTs".to_string(),
    //             ));
    //         }
    //         if !pool.is_active {
    //             return Err(ContractError::InvalidPool("pool is inactive".to_string()));
    //         }

    //         for nft_token_id in pool_nft.nft_token_ids {
    //             if pool.spot_price > remaining_balance {
    //                 return Err(ContractError::InsufficientFunds(
    //                     "insufficient funds to buy all NFTs".to_string(),
    //                 ));
    //             }
    //             remaining_balance -= pool.spot_price;
    //             self.process_swap(&mut pool, nft_token_id)?;
    //         }
    //     }

    //     Ok(())
    // }

    // pub fn swap_nft_for_tokens(
    //     &mut self,
    //     storage: &'a mut dyn Storage,
    //     collection: Addr,
    //     nft_token_ids: Vec<String>,
    //     min_expected_token_output: Uint128,
    // ) -> Result<(), ContractError> {
    //     if nft_token_ids.len() == 0 {
    //         return Err(ContractError::InvalidInput(
    //             "nft_token_ids.len() must be greater than 0".to_string(),
    //         ));
    //     }

    //     let mut token_output = Uint128::zero();

    //     if let None = self.pool_quote_iter {
    //         self.pool_quote_iter = Some(
    //             buy_pool_quotes()
    //                 .idx
    //                 .collection_buy_price
    //                 .sub_prefix(self.collection.clone())
    //                 .keys(storage, None, None, Order::Descending),
    //         );
    //     }

    //     for nft_token_id in nft_token_ids {
    //         let pool = self.load_next_pool(storage)?;
    //         if let None = pool {
    //             return Err(ContractError::InvalidInput("no pools found".to_string()));
    //         }

    //         let mut pool = pool.unwrap();
    //         token_output += pool.spot_price;
    //         self.process_swap(&mut pool, nft_token_id);
    //     }

    //     if token_output < min_expected_token_output {
    //         return Err(ContractError::InsufficientFunds(
    //             "insufficient funds to buy all NFTs".to_string(),
    //         ));
    //     }

    //     Ok(())
    // }
}
