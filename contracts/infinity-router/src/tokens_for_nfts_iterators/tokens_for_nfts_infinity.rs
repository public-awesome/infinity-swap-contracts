use crate::tokens_for_nfts_iterators::types::{TokensForNftInternal, TokensForNftQuote};
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

use super::types::TokensForNftSourceData;

pub struct TokensForNftsInfinity<'a> {
    deps: Deps<'a>,
    payout_context: PayoutContext,
    collection: Addr,
    quotes: BTreeSet<TokensForNftInternal>,
    cursor: Option<PairQuoteOffset>,
}

impl<'a> TokensForNftsInfinity<'a> {
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

    pub fn fetch_quote(&mut self) {
        let pair_quote_option = self
            .deps
            .querier
            .query_wasm_smart::<Vec<PairQuote>>(
                &self.payout_context.global_config.infinity_index,
                &InfinityIndexQueryMsg::BuyFromPairQuotes {
                    collection: self.collection.to_string(),
                    denom: self.payout_context.denom.to_string(),
                    query_options: Some(QueryOptions {
                        limit: Some(1),
                        descending: Some(false),
                        min: self.cursor.as_ref().map(|c| QueryBound::Exclusive(c.clone())),
                        max: None,
                    }),
                },
            )
            .unwrap()
            .pop();

        if let Some(pair_quote) = pair_quote_option {
            self.cursor = Some(PairQuoteOffset {
                pair: pair_quote.address.to_string(),
                amount: pair_quote.quote.amount.u128(),
            });

            let pair = self
                .deps
                .querier
                .query_wasm_smart::<Pair>(&pair_quote.address, &PairQueryMsg::Pair {})
                .map_err(|_| StdError::generic_err("pair not found"))
                .unwrap();

            self.quotes.insert(TokensForNftInternal {
                address: pair_quote.address,
                amount: pair_quote.quote.amount,
                source_data: TokensForNftSourceData::Infinity(pair),
            });
        } else {
            self.cursor = None;
        }
    }
}

impl<'a> Iterator for TokensForNftsInfinity<'a> {
    type Item = TokensForNftQuote;

    fn next(&mut self) -> Option<Self::Item> {
        let quote_option = self.quotes.pop_first();
        let retval = quote_option.as_ref().map(|qo| qo.into());

        if let Some(mut next_quote) = quote_option {
            if let Some(cursor) = &self.cursor {
                if cursor.pair == next_quote.address {
                    self.fetch_quote();
                }
            }

            match next_quote.source_data {
                TokensForNftSourceData::Infinity(ref mut pair) => {
                    pair.sim_swap_tokens_for_nft(&self.payout_context);

                    if let Some(summary) = &pair.internal.sell_to_pair_quote_summary {
                        next_quote.amount = summary.total();
                        self.quotes.insert(next_quote);
                    }
                },
            };
        }

        retval
    }
}
