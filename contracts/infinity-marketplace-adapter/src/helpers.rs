use std::cmp::Ordering;
use std::collections::BTreeSet;

use crate::state::Config;
use crate::ContractError;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, BlockInfo, Deps};
use infinity_shared::interface::NftOrder;
use sg_marketplace::msg::{
    BidsResponse, CollectionBidOffset, CollectionBidsResponse, QueryMsg as MarketplaceQueryMsg,
};
use sg_marketplace::state::{Bid, CollectionBid, Order};
use sg_marketplace_common::only_owner;

/// Validate NftSwap vector token amounts, and NFT ownership
pub fn validate_user_submitted_nfts(
    deps: Deps,
    sender: &Addr,
    collection: &Addr,
    nft_orders: &Vec<NftOrder>,
    max_batch_size: u32,
) -> Result<(), ContractError> {
    if nft_orders.is_empty() {
        return Err(ContractError::InvalidInput(
            "nft orders must not be empty".to_string(),
        ));
    }
    if nft_orders.len() > max_batch_size as usize {
        return Err(ContractError::InvalidInput(
            "nft orders must not exceed max batch size".to_string(),
        ));
    }

    let mut uniq_token_ids: BTreeSet<String> = BTreeSet::new();
    for (idx, nft_order) in nft_orders.iter().enumerate() {
        only_owner(&deps.querier, sender, collection, &nft_order.token_id)?;
        if uniq_token_ids.contains(&nft_order.token_id) {
            return Err(ContractError::InvalidInput(
                "found duplicate nft token id".to_string(),
            ));
        }
        uniq_token_ids.insert(nft_order.token_id.clone());

        if idx == 0 {
            continue;
        }
        if nft_orders[idx - 1].amount < nft_order.amount {
            return Err(ContractError::InvalidInput(
                "nft order amounts must decrease monotonically".to_string(),
            ));
        }
    }
    Ok(())
}

// Fetch collection bids in a loop until a certain number are retrieved or there are no more to fetch,
// filters out invalid collection bids
pub fn fetch_collection_bids(
    deps: Deps,
    marketplace: &Addr,
    collection: &Addr,
    block: &BlockInfo,
    limit: u32,
) -> Result<Vec<CollectionBid>, ContractError> {
    let mut collection_bids: Vec<CollectionBid> = vec![];

    loop {
        let mut start_after: Option<CollectionBidOffset> = None;
        if let Some(last_collection_bid) = collection_bids.last() {
            start_after = Some(CollectionBidOffset {
                price: last_collection_bid.price,
                collection: last_collection_bid.collection.to_string(),
                bidder: last_collection_bid.bidder.to_string(),
            });
        }

        let query_limit = limit - collection_bids.len() as u32;
        let collection_bids_response: CollectionBidsResponse = deps.querier.query_wasm_smart(
            marketplace,
            &MarketplaceQueryMsg::CollectionBidsSortedByPrice {
                collection: collection.to_string(),
                start_after,
                limit: Some(query_limit),
            },
        )?;
        let num_results = collection_bids_response.bids.len();

        // Filter out expired collection bids
        collection_bids.extend(
            collection_bids_response
                .bids
                .into_iter()
                .filter(|cb| !cb.is_expired(block)),
        );

        if num_results < query_limit as usize || collection_bids.len() >= limit as usize {
            break;
        }
    }

    Ok(collection_bids)
}

#[cw_serde]
pub enum MatchedBid {
    Bid(Bid),
    CollectionBid(CollectionBid),
}

pub struct MatchedUserSubmittedNftOrder {
    pub nft_order: NftOrder,
    pub matched_bid: Option<MatchedBid>,
}

pub fn match_user_submitted_nfts(
    deps: Deps,
    block: &BlockInfo,
    config: &Config,
    collection: &Addr,
    nft_orders: Vec<NftOrder>,
) -> Result<Vec<MatchedUserSubmittedNftOrder>, ContractError> {
    let mut collection_bids = fetch_collection_bids(
        deps,
        &config.marketplace,
        &collection,
        block,
        config.max_batch_size,
    )?;

    let mut matched_user_submitted_nfts: Vec<MatchedUserSubmittedNftOrder> = vec![];

    for nft_order in nft_orders {
        let token_id = nft_order.token_id.parse::<u32>().unwrap();
        let mut bid_response: BidsResponse = deps.querier.query_wasm_smart(
            &config.marketplace,
            &MarketplaceQueryMsg::BidsSortedByTokenPrice {
                collection: collection.to_string(),
                token_id: token_id,
                start_after: None,
                limit: Some(1),
                descending: Some(true),
                include_expired: Some(false),
            },
        )?;

        let bid = bid_response.bids.pop();
        let collection_bid = collection_bids.pop();

        let mut matched_bid: Option<MatchedBid> = match (bid, collection_bid) {
            (Some(bid), Some(collection_bid)) => match bid.price.cmp(&collection_bid.price) {
                Ordering::Greater | Ordering::Equal => {
                    collection_bids.push(collection_bid);
                    Some(MatchedBid::Bid(bid))
                }
                Ordering::Less => Some(MatchedBid::CollectionBid(collection_bid)),
            },
            (Some(bid), None) => Some(MatchedBid::Bid(bid)),
            (None, Some(collection_bid)) => Some(MatchedBid::CollectionBid(collection_bid)),
            (None, None) => None,
        };

        if let Some(_matched_bid) = &matched_bid {
            let price = match _matched_bid {
                MatchedBid::Bid(bid) => bid.price,
                MatchedBid::CollectionBid(collection_bid) => collection_bid.price,
            };
            if price < nft_order.amount {
                matched_bid = None;
            }
        }

        matched_user_submitted_nfts.push(MatchedUserSubmittedNftOrder {
            nft_order: nft_order,
            matched_bid,
        });
    }
    Ok(matched_user_submitted_nfts)
}
