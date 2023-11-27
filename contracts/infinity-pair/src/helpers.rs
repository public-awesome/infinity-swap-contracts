use crate::{
    pair::Pair,
    state::{QuoteSummary, TokenPayment, PAIR_CONFIG, PAIR_IMMUTABLE, PAIR_INTERNAL},
    ContractError,
};

use cosmwasm_std::{
    ensure_eq, Addr, Coin, Decimal, Deps, MessageInfo, QuerierWrapper, Storage, Uint128,
};
use infinity_global::{load_global_config, load_min_price, state::GlobalConfig};
use infinity_shared::InfinityError;
use stargaze_royalty_registry::{
    msg::{QueryMsg as RoyaltyRegistryQueryMsg, RoyaltyPaymentResponse},
    state::RoyaltyEntry,
};
use std::cmp::min;

pub fn only_pair_owner(info: &MessageInfo, pair: &Pair) -> Result<(), ContractError> {
    ensure_eq!(
        info.sender,
        &pair.immutable.owner,
        InfinityError::Unauthorized("sender is not the owner of the pair".to_string())
    );
    Ok(())
}

pub fn only_active(pair: &Pair) -> Result<(), ContractError> {
    ensure_eq!(
        pair.config.is_active,
        true,
        ContractError::InvalidPair("pair is inactive".to_string())
    );
    Ok(())
}

pub fn load_pair(
    contract: &Addr,
    storage: &dyn Storage,
    querier: &QuerierWrapper,
) -> Result<Pair, ContractError> {
    let immutable = PAIR_IMMUTABLE.load(storage)?;
    let config = PAIR_CONFIG.load(storage)?;
    let internal = PAIR_INTERNAL.load(storage)?;
    let total_tokens = querier.query_balance(contract, immutable.denom.clone())?.amount;
    Ok(Pair::new(immutable, config, internal, total_tokens))
}

pub struct PayoutContext {
    pub global_config: GlobalConfig<Addr>,
    pub royalty_entry: Option<RoyaltyEntry>,
    pub min_price: Coin,
    pub infinity_global: Addr,
    pub denom: String,
}

impl PayoutContext {
    pub fn build_quote_summary(&self, pair: &Pair, sale_ammount: Uint128) -> Option<QuoteSummary> {
        if sale_ammount < self.min_price.amount {
            return None;
        }

        let fair_burn = TokenPayment {
            recipient: self.global_config.fair_burn.clone(),
            amount: sale_ammount.mul_ceil(self.global_config.fair_burn_fee_percent),
        };

        let royalty = if let Some(royalty_entry) = &self.royalty_entry {
            let royalty_fee_percent = min(
                self.royalty_entry.as_ref().map_or(Decimal::zero(), |r| r.share),
                self.global_config.max_royalty_fee_percent,
            );
            if royalty_fee_percent > Decimal::zero() {
                Some(TokenPayment {
                    recipient: royalty_entry.recipient.clone(),
                    amount: sale_ammount.mul_ceil(royalty_fee_percent),
                })
            } else {
                None
            }
        } else {
            None
        };

        let swap_fee_percent =
            min(pair.swap_fee_percent(), self.global_config.max_swap_fee_percent);
        let swap = if swap_fee_percent > Decimal::zero() {
            Some(TokenPayment {
                recipient: pair.asset_recipient(),
                amount: sale_ammount.mul_ceil(swap_fee_percent),
            })
        } else {
            None
        };

        let seller_amount = sale_ammount
            - fair_burn.amount
            - royalty.as_ref().map_or(Uint128::zero(), |r| r.amount)
            - swap.as_ref().map_or(Uint128::zero(), |s| s.amount);

        Some(QuoteSummary {
            fair_burn,
            royalty,
            swap,
            seller_amount,
        })
    }
}

pub fn load_payout_context(
    deps: Deps,
    infinity_global: &Addr,
    collection: &Addr,
    denom: &str,
) -> Result<PayoutContext, ContractError> {
    let global_config = load_global_config(&deps.querier, infinity_global)?;

    let min_price = load_min_price(&deps.querier, infinity_global, denom)?
        .ok_or(InfinityError::InternalError("denom not supported".to_string()))?;

    let royalty_payment_response = deps.querier.query_wasm_smart::<RoyaltyPaymentResponse>(
        &global_config.royalty_registry,
        &RoyaltyRegistryQueryMsg::RoyaltyPayment {
            collection: collection.to_string(),
            protocol: Some(infinity_global.to_string()),
        },
    )?;

    let royalty_entry = if let Some(royalty_protocol) = royalty_payment_response.royalty_protocol {
        Some(royalty_protocol.royalty_entry)
    } else if let Some(royalty_default) = royalty_payment_response.royalty_default {
        Some(RoyaltyEntry {
            recipient: royalty_default.royalty_entry.recipient,
            share: global_config.default_royalty_fee_percent,
            updated: None,
        })
    } else {
        None
    };

    Ok(PayoutContext {
        global_config,
        royalty_entry,
        min_price,
        infinity_global: infinity_global.clone(),
        denom: denom.to_string(),
    })
}
