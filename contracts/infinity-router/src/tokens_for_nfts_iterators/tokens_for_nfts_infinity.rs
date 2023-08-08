use crate::ContractError;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, StdError, Uint128};
use infinity_global::GlobalConfig;
use infinity_index::{msg::QueryMsg as InfinityIndexQueryMsg, state::PairQuote};
use infinity_pair::msg::QueryMsg as PairQueryMsg;
use infinity_pair::pair::Pair;
use sg_index_query::QueryOptions;
use stargaze_royalty_registry::state::RoyaltyEntry;
use std::{cmp::Ordering, collections::BTreeSet};

#[cw_serde]
pub struct InfinityQuote {
    pub quote: Uint128,
    pub address: Addr,
    pub pair: Pair,
}

impl Ord for InfinityQuote {
    fn cmp(&self, other: &Self) -> Ordering {
        self.quote.cmp(&other.quote)
    }
}

impl PartialOrd for InfinityQuote {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for InfinityQuote {}

pub struct TokensForNftsInfinity<'a> {
    deps: Deps<'a>,
    global_config: GlobalConfig<Addr>,
    collection: Addr,
    denom: String,
    royalty_entry: Option<RoyaltyEntry>,
    quotes: BTreeSet<InfinityQuote>,
    latest_pair: Option<Addr>,
}

impl<'a> TokensForNftsInfinity<'a> {
    pub fn fetch_pair(&mut self) -> Result<Option<InfinityQuote>, ContractError> {
        let mut infinity_quote = None;

        let mut pair_quotes = self.deps.querier.query_wasm_smart::<Vec<PairQuote>>(
            &self.global_config.infinity_index,
            &InfinityIndexQueryMsg::BuyFromPairQuotes {
                collection: self.collection.to_string(),
                denom: self.denom.to_string(),
                query_options: Some(QueryOptions {
                    limit: Some(1),
                    descending: Some(false),
                    min: None,
                    max: None,
                }),
            },
        )?;

        if let Some(pair_quote) = pair_quotes.pop() {
            let pair = self
                .deps
                .querier
                .query_wasm_smart::<Pair>(&pair_quote.pair, &PairQueryMsg::Pair {})
                .map_err(|_| StdError::generic_err("pair not found"))
                .unwrap();

            infinity_quote = Some(InfinityQuote {
                quote: pair_quote.quote.amount,
                address: pair_quote.pair,
                pair,
            });
        }

        Ok(infinity_quote)
    }

    pub fn initialize(
        deps: Deps<'a>,
        global_config: GlobalConfig<Addr>,
        collection: Addr,
        denom: String,
        royalty_entry: Option<RoyaltyEntry>,
    ) -> Result<Self, ContractError> {
        let mut retval = Self {
            deps,
            global_config,
            collection,
            denom,
            royalty_entry,
            quotes: BTreeSet::new(),
            latest_pair: None,
        };

        let infinity_quote = retval.fetch_pair()?;
        if let Some(infinity_quote) = infinity_quote {
            retval.quotes.insert(infinity_quote);
        }

        Ok(retval)
    }
}

impl<'a> Iterator for TokensForNftsInfinity<'a> {
    type Item = InfinityQuote;

    fn next(&mut self) -> Option<Self::Item> {
        let next_quote = self.quotes.pop_first();
        let retval = next_quote.clone();

        if let Some(mut infinity_quote) = next_quote {
            if let Some(latest_pair) = &self.latest_pair {
                if latest_pair == &infinity_quote.address {
                    let fetched_quote = self.fetch_pair().unwrap();
                    if let Some(fetched_quote) = fetched_quote {
                        self.latest_pair = Some(fetched_quote.address.clone());
                        self.quotes.insert(fetched_quote);
                    } else {
                        self.latest_pair = None;
                    }
                }
            }

            infinity_quote.pair.swap_tokens_for_nft(
                self.global_config.fair_burn_fee_percent,
                self.royalty_entry.as_ref().map(|e| e.share),
            );

            if let Some(summary) = &infinity_quote.pair.internal.sell_to_pair_quote_summary {
                infinity_quote.quote = summary.total();
                self.quotes.insert(infinity_quote);
            }
        }

        retval
    }
}
