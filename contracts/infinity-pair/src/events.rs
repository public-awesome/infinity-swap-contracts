use crate::{
    pair::Pair,
    state::{BondingCurve, PairType, QuoteSummary},
};

use cosmwasm_std::{attr, Addr, Event, Uint128};
use std::vec;

const NONE: &str = "None";

pub struct PairEvent<'a> {
    pub ty: &'a str,
    pub pair: &'a Pair,
}

impl<'a> From<PairEvent<'a>> for Event {
    fn from(pe: PairEvent) -> Self {
        let mut event = Event::new(pe.ty.to_string());

        event = event.add_attributes(vec![
            attr("collection", pe.pair.immutable.collection.to_string()),
            attr("denom", pe.pair.immutable.denom.to_string()),
            attr("owner", pe.pair.immutable.owner.to_string()),
        ]);

        match pe.pair.config.pair_type {
            PairType::Token => {
                event = event.add_attribute("pair_type", "token".to_string());
            },
            PairType::Nft => {
                event = event.add_attribute("pair_type", "nft".to_string());
            },
            PairType::Trade {
                swap_fee_percent,
                reinvest_tokens,
                reinvest_nfts,
            } => {
                event = event.add_attributes(vec![
                    attr("pair_type", "trade".to_string()),
                    attr("swap_fee_percent", swap_fee_percent.to_string()),
                    attr("reinvest_tokens", reinvest_tokens.to_string()),
                    attr("reinvest_nfts", reinvest_nfts.to_string()),
                ]);
            },
        }

        match pe.pair.config.bonding_curve {
            BondingCurve::Linear {
                spot_price,
                delta,
            } => {
                event = event.add_attributes(vec![
                    attr("bonding_curve", "linear".to_string()),
                    attr("spot_price", spot_price.to_string()),
                    attr("delta", delta.to_string()),
                ]);
            },
            BondingCurve::Exponential {
                spot_price,
                delta,
            } => {
                event = event.add_attributes(vec![
                    attr("bonding_curve", "exponential".to_string()),
                    attr("spot_price", spot_price.to_string()),
                    attr("delta", delta.to_string()),
                ]);
            },
            BondingCurve::ConstantProduct => {
                event = event.add_attribute("bonding_curve", "constant_product".to_string());
            },
        }

        event = event.add_attribute("is_active", pe.pair.config.is_active.to_string());
        event = event.add_attribute(
            "asset_recipient",
            pe.pair.config.asset_recipient.as_ref().map_or(NONE.to_string(), |ar| ar.to_string()),
        );

        event
    }
}

pub struct SwapEvent<'a> {
    pub ty: &'a str,
    pub token_id: &'a str,
    pub collection: &'a Addr,
    pub pair_owner: &'a Addr,
    pub sender_recipient: &'a Addr,
    pub quote_summary: &'a QuoteSummary,
}

impl<'a> From<SwapEvent<'a>> for Event {
    fn from(se: SwapEvent) -> Self {
        Event::new(se.ty.to_string()).add_attributes(vec![
            attr("token_id", se.token_id),
            attr("collection", se.collection),
            attr("pair_owner", se.pair_owner),
            attr("sender_recipient", se.sender_recipient),
            attr("fair_burn_fee", se.quote_summary.fair_burn.amount),
            attr(
                "royalty_fee",
                se.quote_summary.royalty.as_ref().map_or(Uint128::zero(), |r| r.amount),
            ),
            attr("swap_fee", se.quote_summary.swap.as_ref().map_or(Uint128::zero(), |s| s.amount)),
            attr("seller_amount", se.quote_summary.seller_amount),
            attr("total_price", se.quote_summary.total()),
        ])
    }
}
