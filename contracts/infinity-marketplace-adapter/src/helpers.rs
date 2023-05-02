use crate::reply::BUY_NOW_REPLY_ID;
use crate::state::Config;
use crate::ContractError;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{
    coins, to_binary, Addr, BlockInfo, Deps, QuerierWrapper, SubMsg, Uint128, WasmMsg,
};
use infinity_shared::interface::{NftOrder, NftPayment, Swap, TokenPayment, TransactionType};
use sg_marketplace::msg::{
    AskOffset, AskResponse, AsksResponse, BidsResponse, CollectionBidOffset,
    CollectionBidsResponse, ExecuteMsg as MarketplaceExecuteMsg, QueryMsg as MarketplaceQueryMsg,
};
use sg_marketplace::state::{Ask, Bid, CollectionBid, Order};
use sg_marketplace_common::{only_owner, TransactionFees};
use sg_std::{Response, NATIVE_DENOM};
use std::cmp::Ordering;
use std::collections::BTreeSet;

pub fn validate_nft_owner(
    querier: &QuerierWrapper,
    sender: &Addr,
    collection: &Addr,
    nft_orders: &Vec<NftOrder>,
) -> Result<(), ContractError> {
    for nft_order in nft_orders.iter() {
        only_owner(querier, sender, collection, &nft_order.token_id)?;
    }
    Ok(())
}

/// Validate NftSwap vector token amounts, and NFT ownership
pub fn validate_nft_orders(
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
    for nft_order in nft_orders.iter() {
        if uniq_token_ids.contains(&nft_order.token_id) {
            return Err(ContractError::InvalidInput(
                "found duplicate nft token id".to_string(),
            ));
        }
        uniq_token_ids.insert(nft_order.token_id.clone());
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

// Fetch asks in a loop until a certain number are retrieved or there are no more to fetch,
// filters out invalid asks
pub fn fetch_asks(
    deps: Deps,
    marketplace: &Addr,
    collection: &Addr,
    block: &BlockInfo,
    limit: u32,
) -> Result<Vec<Ask>, ContractError> {
    let mut asks: Vec<Ask> = vec![];

    loop {
        let mut start_after: Option<AskOffset> = None;
        if let Some(last_ask) = asks.last() {
            start_after = Some(AskOffset {
                price: last_ask.price,
                token_id: last_ask.token_id,
            });
        }

        let query_limit = limit - asks.len() as u32;
        let asks_response: AsksResponse = deps.querier.query_wasm_smart(
            marketplace,
            &MarketplaceQueryMsg::AsksSortedByPrice {
                collection: collection.to_string(),
                include_inactive: Some(false),
                start_after,
                limit: Some(query_limit),
            },
        )?;
        let num_results = asks_response.asks.len();

        // Filter out expired asks
        asks.extend(
            asks_response
                .asks
                .into_iter()
                .filter(|ask| !ask.is_expired(block)),
        );

        if num_results < query_limit as usize || asks.len() >= limit as usize {
            break;
        }
    }

    asks.reverse();
    Ok(asks)
}

pub fn tx_fees_to_swap(
    tx_fees: TransactionFees,
    transaction_type: TransactionType,
    token_id: &str,
    sale_price: Uint128,
    buyer: &Addr,
    source: &Addr,
) -> Swap {
    let mut token_payments: Vec<TokenPayment> = vec![];
    if let Some(finders_fee) = tx_fees.finders_fee {
        token_payments.push(TokenPayment {
            label: "finder".to_string(),
            address: finders_fee.recipient.to_string(),
            amount: finders_fee.coin.amount,
        });
    }
    if let Some(royalty_fee) = tx_fees.royalty_fee {
        token_payments.push(TokenPayment {
            label: "royalty".to_string(),
            address: royalty_fee.recipient.to_string(),
            amount: royalty_fee.coin.amount,
        });
    }
    token_payments.push(TokenPayment {
        label: "seller".to_string(),
        address: tx_fees.seller_payment.recipient.to_string(),
        amount: tx_fees.seller_payment.coin.amount,
    });

    Swap {
        source: source.to_string(),
        transaction_type,
        sale_price,
        network_fee: tx_fees.fair_burn_fee,
        nft_payments: vec![NftPayment {
            label: "buyer".to_string(),
            token_id: token_id.to_string(),
            address: buyer.to_string(),
        }],
        token_payments,
    }
}

#[cw_serde]
pub enum MatchedBid {
    Bid(Bid),
    CollectionBid(CollectionBid),
}

pub struct MatchedNftAgainstTokens {
    pub nft_order: NftOrder,
    pub matched_bid: Option<MatchedBid>,
}

pub fn match_nfts_against_tokens(
    deps: Deps,
    block: &BlockInfo,
    config: &Config,
    collection: &Addr,
    nft_orders: Vec<NftOrder>,
    robust: bool,
) -> Result<Vec<MatchedNftAgainstTokens>, ContractError> {
    let mut collection_bids = fetch_collection_bids(
        deps,
        &config.marketplace,
        &collection,
        block,
        config.max_batch_size,
    )?;

    let mut matched_user_submitted_nfts: Vec<MatchedNftAgainstTokens> = vec![];

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

        if !robust && matched_bid.is_none() {
            return Err(ContractError::MatchError(
                "all nfts not matched with a bid".to_string(),
            ));
        }

        matched_user_submitted_nfts.push(MatchedNftAgainstTokens {
            nft_order: nft_order,
            matched_bid,
        });
    }

    Ok(matched_user_submitted_nfts)
}

pub struct MatchedTokensAgainstSpecificNfts {
    pub nft_order: NftOrder,
    pub matched_ask: Option<Ask>,
}

pub fn match_tokens_against_specific_nfts(
    deps: Deps,
    config: &Config,
    collection: &Addr,
    nft_orders: Vec<NftOrder>,
    robust: bool,
) -> Result<Vec<MatchedTokensAgainstSpecificNfts>, ContractError> {
    let mut matched_user_submitted_tokens: Vec<MatchedTokensAgainstSpecificNfts> = vec![];

    for nft_order in nft_orders {
        let token_id = nft_order.token_id.parse::<u32>().unwrap();
        let ask_response: AskResponse = deps.querier.query_wasm_smart(
            &config.marketplace,
            &MarketplaceQueryMsg::Ask {
                collection: collection.to_string(),
                token_id: token_id,
            },
        )?;
        let ask = match ask_response.ask {
            Some(_ask) => {
                if nft_order.amount >= _ask.price {
                    Some(_ask)
                } else {
                    None
                }
            }
            None => None,
        };

        if !robust && ask.is_none() {
            return Err(ContractError::MatchError(
                "all nft orders not matched with an ask".to_string(),
            ));
        }

        matched_user_submitted_tokens.push(MatchedTokensAgainstSpecificNfts {
            nft_order: nft_order,
            matched_ask: ask,
        });
    }
    Ok(matched_user_submitted_tokens)
}

pub struct MatchedTokensAgainstAnyNfts {
    pub token_amount: Uint128,
    pub matched_ask: Option<Ask>,
}

pub fn match_tokens_against_any_nfts(
    deps: Deps,
    block: &BlockInfo,
    config: &Config,
    collection: &Addr,
    nft_orders: Vec<Uint128>,
    robust: bool,
) -> Result<Vec<MatchedTokensAgainstAnyNfts>, ContractError> {
    let mut asks = fetch_asks(
        deps,
        &config.marketplace,
        &collection,
        block,
        config.max_batch_size,
    )?;

    let mut matched_user_submitted_tokens: Vec<MatchedTokensAgainstAnyNfts> = vec![];
    for nft_order in nft_orders {
        let ask = match asks.pop() {
            Some(_ask) => {
                if nft_order >= _ask.price {
                    Some(_ask)
                } else {
                    None
                }
            }
            None => None,
        };

        if !robust && ask.is_none() {
            return Err(ContractError::MatchError(
                "all nft orders not matched with an ask".to_string(),
            ));
        }

        matched_user_submitted_tokens.push(MatchedTokensAgainstAnyNfts {
            token_amount: nft_order,
            matched_ask: ask,
        });
    }
    Ok(matched_user_submitted_tokens)
}

pub fn buy_now(
    response: Response,
    ask: &Ask,
    finder: &Option<String>,
    marketplace: &Addr,
) -> Result<Response, ContractError> {
    let mut response = response;

    let buy_now_msg = MarketplaceExecuteMsg::BuyNow {
        collection: ask.collection.to_string(),
        token_id: ask.token_id,
        expires: ask.expires_at,
        finder: finder.clone(),
        finders_fee_bps: None,
    };

    response = response.add_submessage(SubMsg::reply_always(
        WasmMsg::Execute {
            contract_addr: marketplace.to_string(),
            msg: to_binary(&buy_now_msg)?,
            funds: coins(ask.price.u128(), NATIVE_DENOM),
        },
        BUY_NOW_REPLY_ID,
    ));

    Ok(response)
}
