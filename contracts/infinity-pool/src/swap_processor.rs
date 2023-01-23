use crate::error::ContractError;
use crate::helpers::{
    get_next_pool_counter, get_pool_attributes, only_owner, remove_pool, save_pool, transfer_nft,
    transfer_token,
};
use crate::msg::{ExecuteMsg, PoolNfts, QueryOptions};
use crate::query::query_pools_by_buy_price;
use crate::state::{
    buy_pool_quotes, pools, sell_pool_quotes, BondingCurve, Pool, PoolQuote, PoolType, CONFIG,
};

use cosmwasm_std::{coin, Addr, DepsMut, Env, Event, MessageInfo, StdResult, Storage, Uint128};
use cosmwasm_std::{entry_point, Decimal, Deps, Order};
use cw_utils::{maybe_addr, must_pay, nonpayable};
use sg1::fair_burn;
use sg721::RoyaltyInfoResponse;
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg_marketplace::msg::{ParamsResponse, QueryMsg as MarketplaceQueryMsg};
use sg_std::Response;
use std::collections::{BTreeMap, BTreeSet};

pub struct TokenPaymentPair {
    pub amount: Uint128,
    pub address: String,
}

pub struct NftPaymentPair {
    pub nft_token_id: String,
    pub address: String,
}

pub struct Fees {
    pub trading_fee_percent: Decimal,
    pub royalty: Option<RoyaltyInfoResponse>,
}

pub struct PaymentLedger {
    pub total_network_fee: Uint128,
    pub nfts: BTreeMap<String, Vec<String>>,
    pub royalties: BTreeMap<String, Uint128>,
    pub sellers: BTreeMap<String, Uint128>,
}

pub struct SwapProcessor<'a> {
    pub payment_ledger: PaymentLedger,
    pub marketplace_addr: Addr,
    pub collection: Addr,
    pub seller_recipient: Addr,
    pub pool_set: BTreeSet<Pool>,
    pub pool_quote_iter: Option<Box<dyn Iterator<Item = StdResult<u64>> + 'a>>,
    pub latest: Option<u64>,
    pub fees: Option<Fees>,
}

impl<'a> SwapProcessor<'a> {
    pub fn new(marketplace_addr: Addr, collection: Addr, seller_recipient: Addr) -> Self {
        Self {
            payment_ledger: PaymentLedger {
                total_network_fee: Uint128::zero(),
                nfts: BTreeMap::new(),
                royalties: BTreeMap::new(),
                sellers: BTreeMap::new(),
            },
            marketplace_addr,
            collection,
            seller_recipient,
            pool_set: BTreeSet::new(),
            pool_quote_iter: None,
            latest: None,
            fees: None,
        }
    }

    pub fn set_fees(&mut self, deps: Deps) -> Result<(), ContractError> {
        let marketplace_params: ParamsResponse = deps.querier.query_wasm_smart(
            self.marketplace_addr.clone(),
            &MarketplaceQueryMsg::Params {},
        )?;

        let collection_info: CollectionInfoResponse = deps
            .querier
            .query_wasm_smart(self.collection.clone(), &Sg721QueryMsg::CollectionInfo {})?;

        self.fees = Some(Fees {
            trading_fee_percent: marketplace_params.params.trading_fee_percent,
            royalty: collection_info.royalty_info,
        });

        Ok(())
    }

    fn calc_swap_fees(
        &mut self,
        payment_amount: Uint128,
        nft_token_id: String,
    ) -> Result<(), ContractError> {
        match &self.fees {
            None => {
                return Err(ContractError::InternalError(
                    "fees must be set before processing swap".to_string(),
                ))
            }
            Some(_fees) => {
                let network_fee =
                    payment_amount * _fees.trading_fee_percent / Uint128::from(100u128);
                let mut seller_amount = payment_amount - network_fee;

                // finders fee?

                match &_fees.royalty {
                    Some(royalty) => {
                        let royalty_amount = payment_amount * royalty.share;
                        seller_amount -= royalty_amount;
                        let royalty_entry = self
                            .payment_ledger
                            .royalties
                            .entry(royalty.payment_address.to_string())
                            .or_insert(Uint128::zero());
                        *royalty_entry += royalty_amount;
                    }
                    None => (),
                };

                self.payment_ledger.total_network_fee += network_fee;
                let seller_entry = self
                    .payment_ledger
                    .royalties
                    .entry(self.seller_recipient.to_string())
                    .or_insert(Uint128::zero());
                *seller_entry += seller_amount;

                let nft_entry = self
                    .payment_ledger
                    .nfts
                    .entry(self.seller_recipient.to_string())
                    .or_insert(vec![]);
                nft_entry.push(nft_token_id);

                Ok(())
            }
        }
    }

    pub fn load_next_pool(&mut self, storage: &dyn Storage) -> Result<Option<Pool>, ContractError> {
        if self.pool_set.is_empty() || Some(self.pool_set.first().unwrap().id) == self.latest {
            let pool_id = self.pool_quote_iter.as_mut().unwrap().next().unwrap()?;

            let pool = pools()
                .load(storage, pool_id)
                .map_err(|_| ContractError::InvalidInput("pool does not exist".to_string()))?;

            self.pool_set.insert(pool);
            self.latest = Some(pool_id);
        }

        Ok(self.pool_set.pop_first())
    }

    pub fn process_swap(
        &mut self,
        pool: &mut Pool,
        nft_token_id: String,
    ) -> Result<(), ContractError> {
        pool.buy_nft_from_pool(nft_token_id.clone())?;
        self.calc_swap_fees(pool.spot_price, nft_token_id)?;

        Ok(())
    }

    pub fn swap_token_for_specific_nfts(
        &mut self,
        deps: DepsMut,
        specific_nfts: Vec<PoolNfts>,
        max_expected_token_input: Uint128,
    ) -> Result<(), ContractError> {
        if specific_nfts.len() == 0 {
            return Err(ContractError::InvalidInput(
                "specific_nfts.len() must be greater than 0".to_string(),
            ));
        }

        let mut remaining_balance = max_expected_token_input.clone();

        for pool_nft in specific_nfts {
            if pool_nft.nft_token_ids.len() == 0 {
                return Err(ContractError::InvalidInput(format!(
                    "no nfts selected for pool_id {}",
                    pool_nft.pool_id
                )));
            }

            let mut pool = pools()
                .load(deps.storage, pool_nft.pool_id)
                .map_err(|_| ContractError::InvalidInput("pool does not exist".to_string()))?;

            if !pool.can_sell_nfts() {
                return Err(ContractError::InvalidPool(
                    "pool cannot sell NFTs".to_string(),
                ));
            }
            if !pool.is_active {
                return Err(ContractError::InvalidPool("pool is inactive".to_string()));
            }

            for nft_token_id in pool_nft.nft_token_ids {
                if pool.spot_price > remaining_balance {
                    return Err(ContractError::InsufficientFunds(
                        "insufficient funds to buy all NFTs".to_string(),
                    ));
                }
                remaining_balance -= pool.spot_price;
                self.process_swap(&mut pool, nft_token_id)?;
            }
        }

        Ok(())
    }

    pub fn swap_nft_for_tokens(
        &mut self,
        storage: &'a mut dyn Storage,
        collection: Addr,
        nft_token_ids: Vec<String>,
        min_expected_token_output: Uint128,
    ) -> Result<(), ContractError> {
        if nft_token_ids.len() == 0 {
            return Err(ContractError::InvalidInput(
                "nft_token_ids.len() must be greater than 0".to_string(),
            ));
        }

        let mut token_output = Uint128::zero();

        if let None = self.pool_quote_iter {
            self.pool_quote_iter = Some(
                buy_pool_quotes()
                    .idx
                    .collection_buy_price
                    .sub_prefix(self.collection.clone())
                    .keys(storage, None, None, Order::Descending),
            );
        }

        for nft_token_id in nft_token_ids {
            let pool = self.load_next_pool(storage)?;
            if let None = pool {
                return Err(ContractError::InvalidInput("no pools found".to_string()));
            }

            let mut pool = pool.unwrap();
            token_output += pool.spot_price;
            self.process_swap(&mut pool, nft_token_id);
        }

        if token_output < min_expected_token_output {
            return Err(ContractError::InsufficientFunds(
                "insufficient funds to buy all NFTs".to_string(),
            ));
        }

        Ok(())
    }
}
