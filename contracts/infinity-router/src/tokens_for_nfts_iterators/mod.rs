mod tokens_for_nfts_infinity;

use std::iter::Peekable;

use crate::tokens_for_nfts_iterators::tokens_for_nfts_infinity::TokensForNftsInfinity;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, Uint128};
use infinity_global::GlobalConfig;
use stargaze_royalty_registry::state::RoyaltyEntry;

#[cw_serde]
pub enum TokensForNftSource {
    Infinity,
}

#[cw_serde]
pub struct TokensForNftQuote {
    pub source: TokensForNftSource,
    pub address: Addr,
    pub amount: Uint128,
}

pub enum SourceIters<'a> {
    Infinity(Peekable<TokensForNftsInfinity<'a>>),
}

pub struct TokensForNfts<'a> {
    sources: Vec<SourceIters<'a>>,
}

impl<'a> TokensForNfts<'a> {
    pub fn initialize(
        deps: Deps<'a>,
        global_config: GlobalConfig<Addr>,
        collection: Addr,
        denom: String,
        royalty_entry: Option<RoyaltyEntry>,
        filter_sources: Vec<TokensForNftSource>,
    ) -> Self {
        let quote_sources = vec![TokensForNftSource::Infinity]
            .into_iter()
            .filter(|s| !filter_sources.contains(s))
            .collect::<Vec<TokensForNftSource>>();

        let mut sources: Vec<SourceIters> = Vec::new();
        for quote_source in quote_sources {
            match quote_source {
                TokensForNftSource::Infinity => {
                    sources.push(SourceIters::Infinity(
                        TokensForNftsInfinity::initialize(
                            deps,
                            global_config.clone(),
                            collection.clone(),
                            denom.clone(),
                            royalty_entry.clone(),
                        )
                        .unwrap()
                        .peekable(),
                    ));
                },
            };
        }

        Self {
            sources,
        }
    }
}

impl<'a> Iterator for TokensForNfts<'a> {
    type Item = TokensForNftQuote;

    fn next(&mut self) -> Option<Self::Item> {
        let mut retval: Option<TokensForNftQuote> = None;
        for source in &mut self.sources {
            match source {
                SourceIters::Infinity(i) => {
                    let infinity_quote = i.peek();
                    retval = match (&retval, infinity_quote) {
                        (Some(retval_inner), Some(infinity_quote)) => {
                            if infinity_quote.quote <= retval_inner.amount {
                                let infinity_quote = i.next().unwrap();
                                Some(TokensForNftQuote {
                                    source: TokensForNftSource::Infinity,
                                    address: infinity_quote.address,
                                    amount: infinity_quote.quote,
                                })
                            } else {
                                retval
                            }
                        },
                        (None, Some(_)) => {
                            let infinity_quote = i.next().unwrap();
                            Some(TokensForNftQuote {
                                source: TokensForNftSource::Infinity,
                                address: infinity_quote.address,
                                amount: infinity_quote.quote,
                            })
                        },

                        (_, None) => retval,
                    }
                },
            };
        }

        retval
    }
}
