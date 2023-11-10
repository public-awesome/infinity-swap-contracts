use crate::{pair::Pair, state::QuoteSummary};

use cosmwasm_std::{attr, Addr, Coin, Event};
use std::vec;

pub struct CreatePairEvent<'a> {
    pub ty: &'a str,
    pub pair: &'a Pair,
}

impl<'a> From<CreatePairEvent<'a>> for Event {
    fn from(pe: CreatePairEvent) -> Self {
        Event::new(pe.ty.to_string()).add_attributes(pe.pair.get_event_attrs(vec![
            "collection",
            "denom",
            "owner",
            "pair_type",
            "swap_fee_percent",
            "reinvest_tokens",
            "reinvest_nfts",
            "bonding_curve",
            "spot_price",
            "delta",
            "is_active",
            "asset_recipient",
        ]))
    }
}

pub struct UpdatePairEvent<'a> {
    pub ty: &'a str,
    pub pair: &'a Pair,
}

impl<'a> From<UpdatePairEvent<'a>> for Event {
    fn from(pe: UpdatePairEvent) -> Self {
        Event::new(pe.ty.to_string()).add_attributes(pe.pair.get_event_attrs(vec![
            "pair_type",
            "swap_fee_percent",
            "reinvest_tokens",
            "reinvest_nfts",
            "bonding_curve",
            "spot_price",
            "delta",
            "is_active",
            "asset_recipient",
            "total_tokens",
            "total_nfts",
            "sell_to_pair_quote",
            "buy_from_pair_quote",
        ]))
    }
}

pub struct NftTransferEvent<'a> {
    pub ty: &'a str,
    pub pair: &'a Pair,
    pub token_ids: &'a Vec<String>,
}

impl<'a> From<NftTransferEvent<'a>> for Event {
    fn from(nte: NftTransferEvent) -> Self {
        Event::new(nte.ty.to_string())
            .add_attributes(nte.pair.get_event_attrs(vec![
                "total_tokens",
                "total_nfts",
                "sell_to_pair_quote",
                "buy_from_pair_quote",
            ]))
            .add_attributes(nte.token_ids.iter().map(|token_id| ("token_id", token_id)))
    }
}

pub struct TokenTransferEvent<'a> {
    pub ty: &'a str,
    pub pair: &'a Pair,
    pub funds: &'a Coin,
}

impl<'a> From<TokenTransferEvent<'a>> for Event {
    fn from(tte: TokenTransferEvent) -> Self {
        Event::new(tte.ty.to_string())
            .add_attributes(tte.pair.get_event_attrs(vec![
                "total_tokens",
                "sell_to_pair_quote",
                "buy_from_pair_quote",
            ]))
            .add_attribute("funds", tte.funds.to_string())
    }
}

pub struct SwapEvent<'a> {
    pub ty: &'a str,
    pub pair: &'a Pair,
    pub token_id: &'a str,
    pub sender_recipient: &'a Addr,
    pub quote_summary: &'a QuoteSummary,
}

impl<'a> From<SwapEvent<'a>> for Event {
    fn from(se: SwapEvent) -> Self {
        let mut event =
            Event::new(se.ty.to_string()).add_attributes(se.pair.get_event_attrs(vec![
                "spot_price",
                "is_active",
                "total_tokens",
                "sell_to_pair_quote",
                "buy_from_pair_quote",
            ]));

        event = event.add_attributes(vec![
            attr("token_id", se.token_id),
            attr("sender_recipient", se.sender_recipient),
            attr("fair_burn_fee", se.quote_summary.fair_burn.amount),
            attr("seller_amount", se.quote_summary.seller_amount),
        ]);

        if let Some(royalty) = se.quote_summary.royalty.as_ref() {
            event = event.add_attribute("royalty_fee", royalty.amount);
        }
        if let Some(swap) = se.quote_summary.swap.as_ref() {
            event = event.add_attribute("swap_fee", swap.amount);
        }

        event
    }
}
