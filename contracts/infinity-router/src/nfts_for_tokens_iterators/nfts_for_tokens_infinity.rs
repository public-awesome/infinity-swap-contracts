use crate::nfts_for_tokens_iterators::types::{
    NftForTokensInternal, NftForTokensQuote, NftForTokensSourceData,
};
use crate::ContractError;

use cosmwasm_std::{Addr, Deps, StdError};
use infinity_index::{
    msg::{PairQuoteOffset, QueryMsg as InfinityIndexQueryMsg},
    state::PairQuote,
};
use infinity_pair::helpers::load_payout_context;
use infinity_pair::pair::Pair;
use infinity_pair::{helpers::PayoutContext, msg::QueryMsg as PairQueryMsg};
use sg_index_query::{QueryBound, QueryOptions};
use std::collections::BTreeSet;

pub struct NftsForTokensInfinity<'a> {
    deps: Deps<'a>,
    payout_context: PayoutContext,
    collection: Addr,
    quotes: BTreeSet<NftForTokensInternal>,
    cursor: Option<PairQuoteOffset>,
}

impl<'a> NftsForTokensInfinity<'a> {
    pub fn initialize(
        deps: Deps<'a>,
        infinity_global: &Addr,
        collection: &Addr,
        denom: &str,
    ) -> Result<Self, ContractError> {
        let payout_context = load_payout_context(deps, infinity_global, collection, denom)
            .map_err(|e| StdError::generic_err(e.to_string()))?;

        let mut retval = Self {
            deps,
            payout_context,
            collection: collection.clone(),
            quotes: BTreeSet::new(),
            cursor: None,
        };

        retval.fetch_quote();

        Ok(retval)
    }

    fn fetch_quote(&mut self) {
        let pair_quote_option = self
            .deps
            .querier
            .query_wasm_smart::<Vec<PairQuote>>(
                &self.payout_context.global_config.infinity_index,
                &InfinityIndexQueryMsg::SellToPairQuotes {
                    collection: self.collection.to_string(),
                    denom: self.payout_context.denom.to_string(),
                    query_options: Some(QueryOptions {
                        limit: Some(1),
                        descending: Some(true),
                        min: None,
                        max: self.cursor.as_ref().map(|c| QueryBound::Exclusive(c.clone())),
                    }),
                },
            )
            .unwrap()
            .pop();

        if let Some(pair_quote) = pair_quote_option {
            self.cursor = Some(PairQuoteOffset {
                pair: pair_quote.address.to_string(),
                amount: pair_quote.quote.amount,
            });

            let pair = self
                .deps
                .querier
                .query_wasm_smart::<Pair>(&pair_quote.address, &PairQueryMsg::Pair {})
                .map_err(|_| StdError::generic_err("pair not found"))
                .unwrap();

            self.quotes.insert(NftForTokensInternal {
                address: pair_quote.address.clone(),
                amount: pair_quote.quote.amount,
                source_data: NftForTokensSourceData::Infinity(pair),
            });
        } else {
            self.cursor = None;
        };
    }
}

impl<'a> Iterator for NftsForTokensInfinity<'a> {
    type Item = NftForTokensQuote;

    fn next(&mut self) -> Option<Self::Item> {
        let quote_option = self.quotes.pop_last();
        let retval: Option<NftForTokensQuote> = quote_option.as_ref().map(|qo| qo.into());

        if let Some(mut quote) = quote_option {
            if let Some(cursor) = &self.cursor {
                if cursor.pair == quote.address {
                    self.fetch_quote();
                }
            }

            match quote.source_data {
                NftForTokensSourceData::Infinity(ref mut pair) => {
                    pair.sim_swap_nft_for_tokens(&self.payout_context);

                    if let Some(summary) = &pair.internal.sell_to_pair_quote_summary {
                        quote.amount = summary.seller_amount;
                        self.quotes.insert(quote);
                    }
                },
            };
        }

        retval
    }
}
