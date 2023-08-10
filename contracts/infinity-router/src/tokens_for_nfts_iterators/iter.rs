use crate::tokens_for_nfts_iterators::{
    tokens_for_nfts_infinity::TokensForNftsInfinity,
    types::{TokensForNftQuote, TokensForNftSource},
};

use cosmwasm_std::{Addr, Deps};
use std::iter::Peekable;

pub enum SourceIters<'a> {
    Infinity(Peekable<TokensForNftsInfinity<'a>>),
}

pub struct TokensForNfts<'a> {
    sources: Vec<SourceIters<'a>>,
}

impl<'a> TokensForNfts<'a> {
    pub fn initialize(
        deps: Deps<'a>,
        infinity_global: &Addr,
        collection: &Addr,
        denom: &String,
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
                        TokensForNftsInfinity::initialize(deps, infinity_global, collection, denom)
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
        let result = self
            .sources
            .iter_mut()
            .enumerate()
            .filter_map(|(idx, iter)| match iter {
                SourceIters::Infinity(peekable) => peekable.peek().map(|peeked| (idx, peeked)),
            })
            .min_by_key(|&(_, q)| q.amount);

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
