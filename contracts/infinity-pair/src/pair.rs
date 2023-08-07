use crate::error::ContractError;
use crate::math::{
    calc_cp_trade_buy_from_pair_price, calc_cp_trade_sell_to_pair_price,
    calc_exponential_spot_price_user_submits_nft, calc_exponential_spot_price_user_submits_tokens,
    calc_exponential_trade_buy_from_pair_price, calc_linear_spot_price_user_submits_nft,
    calc_linear_spot_price_user_submits_tokens, calc_linear_trade_buy_from_pair_price,
};
use crate::msg::TransactionType;
use crate::state::{
    BondingCurve, PairConfig, PairImmutable, PairInternal, PairType, QuoteSummary, PAIR_CONFIG,
    PAIR_INTERNAL,
};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{coin, to_binary, Addr, Decimal, Storage, Uint128, WasmMsg};
use infinity_index::msg::ExecuteMsg as InfinityIndexExecuteMsg;
use sg_marketplace_common::address::address_or;
use sg_marketplace_common::coin::transfer_coins;
use sg_std::Response;
use stargaze_fair_burn::append_fair_burn_msg;

impl QuoteSummary {
    pub fn new(
        sale_ammount: Uint128,
        fair_burn_fee_percent: Decimal,
        royalty_fee_percent: Option<Decimal>,
    ) -> Self {
        let fair_burn_amount = sale_ammount.mul_ceil(fair_burn_fee_percent);
        let royalty_amount = royalty_fee_percent.map(|fee| sale_ammount.mul_ceil(fee));
        let seller_amount =
            sale_ammount - fair_burn_amount - royalty_amount.unwrap_or(Uint128::zero());
        Self {
            fair_burn_amount,
            royalty_amount,
            seller_amount,
        }
    }

    pub fn total(&self) -> Uint128 {
        self.fair_burn_amount + self.royalty_amount.unwrap_or(Uint128::zero()) + self.seller_amount
    }

    pub fn payout(
        &self,
        denom: &str,
        fair_burn: &Addr,
        royalty_recipient: Option<&Addr>,
        seller_recipient: &Addr,
        mut response: Response,
    ) -> Response {
        response = append_fair_burn_msg(
            &fair_burn,
            vec![coin(self.fair_burn_amount.u128(), denom)],
            None,
            response,
        );

        match (self.royalty_amount, royalty_recipient) {
            (Some(royalty_amount), Some(royalty_recipient)) => {
                response = transfer_coins(
                    vec![coin(royalty_amount.u128(), denom)],
                    royalty_recipient,
                    response,
                );
            },
            _ => {},
        }

        response = transfer_coins(
            vec![coin(self.seller_amount.u128(), denom)],
            seller_recipient,
            response,
        );

        response
    }
}

#[cw_serde]
pub struct Pair {
    pub immutable: PairImmutable,
    pub config: PairConfig,
    pub internal: PairInternal,
    pub total_tokens: Uint128,
}

impl Pair {
    pub fn initialize(immutable: PairImmutable, config: PairConfig) -> Self {
        let pair_internal = PairInternal {
            total_nfts: Uint128::zero(),
            buy_from_pair_quote_summary: None,
            sell_to_pair_quote_summary: None,
        };
        Pair::new(immutable, config, pair_internal, Uint128::zero())
    }

    pub fn new(
        immutable: PairImmutable,
        config: PairConfig,
        internal: PairInternal,
        total_tokens: Uint128,
    ) -> Self {
        Self {
            immutable,
            config,
            internal,
            total_tokens,
        }
    }

    pub fn save_and_update_indices(
        &mut self,
        storage: &mut dyn Storage,
        infinity_index: &Addr,
        mut response: Response,
    ) -> Result<Response, ContractError> {
        PAIR_CONFIG.save(storage, &self.config)?;
        PAIR_INTERNAL.save(storage, &self.internal)?;

        response = self.update_index(infinity_index, response);

        Ok(response)
    }

    pub fn asset_recipient(&self) -> Addr {
        address_or(self.config.asset_recipient.as_ref(), &self.immutable.owner)
    }

    pub fn reinvest_nfts(&self) -> bool {
        match self.config.pair_type {
            PairType::Trade {
                reinvest_nfts,
                ..
            } => reinvest_nfts,
            _ => false,
        }
    }

    pub fn reinvest_tokens(&self) -> bool {
        match self.config.pair_type {
            PairType::Trade {
                reinvest_tokens,
                ..
            } => reinvest_tokens,
            _ => false,
        }
    }

    pub fn swap_nft_for_tokens(
        &mut self,
        fair_burn_fee_percent: Decimal,
        royalty_fee_percent: Option<Decimal>,
    ) {
        self.total_tokens -=
            self.internal.sell_to_pair_quote_summary.as_ref().unwrap().total().clone();

        if self.reinvest_nfts() {
            self.internal.total_nfts += Uint128::one();
        };

        self.update_spot_price(TransactionType::UserSubmitsNfts);
        self.update_sell_to_pair_quote_summary(fair_burn_fee_percent, royalty_fee_percent);
        self.update_buy_from_pair_quote_summary(fair_burn_fee_percent, royalty_fee_percent);
    }

    pub fn swap_tokens_for_nft(
        &mut self,
        fair_burn_fee_percent: Decimal,
        royalty_fee_percent: Option<Decimal>,
    ) {
        self.internal.total_nfts -= Uint128::one();

        if self.reinvest_tokens() {
            self.total_tokens +=
                self.internal.buy_from_pair_quote_summary.as_ref().unwrap().seller_amount;
        };

        self.update_spot_price(TransactionType::UserSubmitsNfts);
        self.update_sell_to_pair_quote_summary(fair_burn_fee_percent, royalty_fee_percent);
        self.update_buy_from_pair_quote_summary(fair_burn_fee_percent, royalty_fee_percent);
    }

    fn update_spot_price(&mut self, tx_type: TransactionType) {
        match self.config.bonding_curve {
            BondingCurve::Linear {
                mut spot_price,
                delta,
            } => {
                let result = match tx_type {
                    TransactionType::UserSubmitsNfts => {
                        calc_linear_spot_price_user_submits_nft(spot_price, delta)
                    },
                    TransactionType::UserSubmitsTokens => {
                        calc_linear_spot_price_user_submits_tokens(spot_price, delta)
                    },
                };
                match result {
                    Ok(new_spot_price) => {
                        spot_price = new_spot_price;
                    },
                    Err(_e) => {
                        self.config.is_active = false;
                    },
                }
            },
            BondingCurve::Exponential {
                mut spot_price,
                delta,
            } => {
                let result = match tx_type {
                    TransactionType::UserSubmitsNfts => {
                        calc_exponential_spot_price_user_submits_nft(spot_price, delta)
                    },
                    TransactionType::UserSubmitsTokens => {
                        calc_exponential_spot_price_user_submits_tokens(spot_price, delta)
                    },
                };
                match result {
                    Ok(new_spot_price) => {
                        spot_price = new_spot_price;
                    },
                    Err(_e) => {
                        self.config.is_active = false;
                    },
                }
            },
            BondingCurve::ConstantProduct => {},
        };
    }

    fn update_sell_to_pair_quote_summary(
        &mut self,
        fair_burn_fee_percent: Decimal,
        royalty_fee_percent: Option<Decimal>,
    ) {
        if !self.config.is_active {
            self.internal.sell_to_pair_quote_summary = None;
            return;
        }

        let quote = match self.config.bonding_curve {
            BondingCurve::Linear {
                spot_price,
                ..
            }
            | BondingCurve::Exponential {
                spot_price,
                ..
            } => Some(spot_price),
            BondingCurve::ConstantProduct => {
                calc_cp_trade_sell_to_pair_price(self.total_tokens, self.internal.total_nfts).ok()
            },
        };

        self.internal.sell_to_pair_quote_summary = match quote {
            Some(quote) if quote <= self.total_tokens => {
                Some(QuoteSummary::new(quote, fair_burn_fee_percent, royalty_fee_percent))
            },
            _ => None,
        };
    }

    fn update_buy_from_pair_quote_summary(
        &mut self,
        fair_burn_fee_percent: Decimal,
        royalty_fee_percent: Option<Decimal>,
    ) {
        if !self.config.is_active || self.internal.total_nfts.is_zero() {
            self.internal.buy_from_pair_quote_summary = None;
            return;
        }

        let price = match (&self.config.pair_type, &self.config.bonding_curve) {
            (
                PairType::Nft,
                BondingCurve::Linear {
                    spot_price,
                    ..
                }
                | BondingCurve::Exponential {
                    spot_price,
                    ..
                },
            ) => Some(*spot_price),
            (
                PairType::Trade {
                    ..
                },
                BondingCurve::Linear {
                    spot_price,
                    delta,
                },
            ) => calc_linear_trade_buy_from_pair_price(*spot_price, *delta).ok(),
            (
                PairType::Trade {
                    ..
                },
                BondingCurve::Exponential {
                    spot_price,
                    delta,
                },
            ) => calc_exponential_trade_buy_from_pair_price(*spot_price, *delta).ok(),
            (
                PairType::Trade {
                    ..
                },
                BondingCurve::ConstantProduct,
            ) => {
                calc_cp_trade_buy_from_pair_price(self.total_tokens, self.internal.total_nfts).ok()
            },
            _ => None,
        };

        self.internal.sell_to_pair_quote_summary = match price {
            Some(price) if price <= self.total_tokens => {
                Some(QuoteSummary::new(price, fair_burn_fee_percent, royalty_fee_percent))
            },
            _ => None,
        };
    }

    fn update_index(&self, infinity_index: &Addr, response: Response) -> Response {
        let sell_to_pair_quote = if let Some(summary) = &self.internal.sell_to_pair_quote_summary {
            Some(summary.seller_amount)
        } else {
            None
        };

        let buy_from_pair_quote = if let Some(summary) = &self.internal.buy_from_pair_quote_summary
        {
            Some(summary.total())
        } else {
            None
        };

        response.add_message(WasmMsg::Execute {
            contract_addr: infinity_index.to_string(),
            msg: to_binary(&InfinityIndexExecuteMsg::UpdatePairIndices {
                collection: self.immutable.collection.to_string(),
                denom: self.immutable.denom.clone(),
                sell_to_pair_quote,
                buy_from_pair_quote,
            })
            .unwrap(),
            funds: vec![],
        })
    }
}
