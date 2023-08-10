use crate::{
    nfts_for_tokens_iterators::{
        nfts_for_tokens_infinity::NftsForTokensInfinity,
        types::{NftForTokensQuote, NftForTokensSource},
    },
    ContractError,
};

use cosmwasm_std::{Addr, Deps};
use std::iter::Peekable;

pub enum SourceIters<'a> {
    Infinity(Peekable<NftsForTokensInfinity<'a>>),
}

pub struct NftsForTokens<'a> {
    sources: Vec<SourceIters<'a>>,
}

impl<'a> NftsForTokens<'a> {
    pub fn initialize(
        deps: Deps<'a>,
        infinity_global: &Addr,
        collection: &Addr,
        denom: &String,
        filter_sources: Vec<NftForTokensSource>,
    ) -> Result<Self, ContractError> {
        let quote_sources = vec![NftForTokensSource::Infinity]
            .into_iter()
            .filter(|s| !filter_sources.contains(s))
            .collect::<Vec<NftForTokensSource>>();

        let mut sources: Vec<SourceIters> = Vec::new();
        for quote_source in quote_sources {
            match quote_source {
                NftForTokensSource::Infinity => {
                    sources.push(SourceIters::Infinity(
                        NftsForTokensInfinity::initialize(
                            deps,
                            infinity_global,
                            collection,
                            denom,
                        )?
                        .peekable(),
                    ));
                },
            };
        }

        Ok(Self {
            sources,
        })
    }
}

impl<'a> Iterator for NftsForTokens<'a> {
    type Item = NftForTokensQuote;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self
            .sources
            .iter_mut()
            .enumerate()
            .filter_map(|(i, iter)| match iter {
                SourceIters::Infinity(peekable) => peekable.peek().map(|peeked| (i, peeked)),
            })
            .max_by_key(|&(_, q)| q.amount);

        if result.is_none() {
            return None;
        }

        let (idx, _) = result.unwrap();

        let quote = match &mut self.sources[idx] {
            SourceIters::Infinity(peekable) => peekable.next().unwrap(),
        };

        Some(quote)
    }
}
