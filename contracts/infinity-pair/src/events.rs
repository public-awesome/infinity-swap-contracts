use crate::{
    pair::Pair,
    state::{BondingCurve, PairType},
};

use cosmwasm_std::{attr, Event};

const NONE: &str = "None";

pub struct PairEvent<'a> {
    pub ty: &'a str,
    pub pair: &'a Pair,
}

impl<'a> From<PairEvent<'a>> for Event {
    fn from(pe: PairEvent) -> Self {
        let mut event: Event = pe.pair.into();
        event.ty = pe.ty.to_string();
        event
    }
}

impl From<&Pair> for Event {
    fn from(pair: &Pair) -> Self {
        let mut event = Event::new("");

        event = event.add_attributes(vec![
            attr("collection", pair.immutable.collection.to_string()),
            attr("denom", pair.immutable.denom.to_string()),
            attr("owner", pair.immutable.owner.to_string()),
        ]);

        match pair.config.pair_type {
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

        match pair.config.bonding_curve {
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

        event = event.add_attribute("is_active", pair.config.is_active.to_string());
        event = event.add_attribute(
            "asset_recipient",
            pair.config.asset_recipient.as_ref().map_or(NONE.to_string(), |ar| ar.to_string()),
        );

        event
    }
}
