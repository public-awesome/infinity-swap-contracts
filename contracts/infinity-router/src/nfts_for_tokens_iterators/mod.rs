mod nfts_for_tokens_infinity;

use std::iter::Peekable;

use crate::nfts_for_tokens_iterators::nfts_for_tokens_infinity::NftsForTokensInfinity;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, Uint128};

#[cw_serde]
pub enum NftForTokensSource {
    Infinity,
}

#[cw_serde]
pub struct NftForTokensQuote {
    pub source: NftForTokensSource,
    pub address: Addr,
    pub amount: Uint128,
}

pub enum SourceIters<'a> {
    Infinity(Peekable<NftsForTokensInfinity<'a>>),
}

pub struct NftsForTokens<'a> {
    sources: Vec<SourceIters<'a>>,
}

impl<'a> NftsForTokens<'a> {
    pub fn initialize(
        deps: Deps<'a>,
        collection: Addr,
        denom: String,
        filter_sources: Vec<NftForTokensSource>,
    ) -> Self {
        let quote_sources = vec![NftForTokensSource::Infinity]
            .into_iter()
            .filter(|s| !filter_sources.contains(s))
            .collect::<Vec<NftForTokensSource>>();

        let mut sources: Vec<SourceIters> = Vec::new();
        for quote_source in quote_sources {
            match quote_source {
                NftForTokensSource::Infinity => {
                    sources.push(SourceIters::Infinity(
                        NftsForTokensInfinity::initialize(deps, collection.clone(), denom.clone())
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

impl<'a> Iterator for NftsForTokens<'a> {
    type Item = NftForTokensQuote;

    fn next(&mut self) -> Option<Self::Item> {
        let mut retval: Option<NftForTokensQuote> = None;
        for source in &mut self.sources {
            match source {
                SourceIters::Infinity(i) => {
                    let infinity_quote = i.peek();
                    retval = match (&retval, infinity_quote) {
                        (Some(retval_inner), Some(infinity_quote)) => {
                            if infinity_quote.amount >= retval_inner.amount {
                                let infinity_quote = i.next().unwrap();
                                Some(NftForTokensQuote {
                                    source: NftForTokensSource::Infinity,
                                    address: infinity_quote.address,
                                    amount: infinity_quote.amount,
                                })
                            } else {
                                retval
                            }
                        },
                        (None, Some(_)) => {
                            let infinity_quote = i.next().unwrap();
                            Some(NftForTokensQuote {
                                source: NftForTokensSource::Infinity,
                                address: infinity_quote.address,
                                amount: infinity_quote.amount,
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
