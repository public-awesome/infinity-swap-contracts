use crate::ContractError;
use cosmwasm_std::{Addr, BlockInfo, Decimal, Deps, Uint128};
use infinity_interface::{NftOrder, NftPayment, Swap, TokenPayment, TransactionType};
use sg721::RoyaltyInfoResponse;
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg_marketplace::msg::{
    AskOffset, AsksResponse, CollectionBidOffset, CollectionBidsResponse, ParamsResponse,
    QueryMsg as MarketplaceQueryMsg,
};
use sg_marketplace::state::{Ask, CollectionBid, Order, SaleType, SudoParams};

/// Load the marketplace params for use within the contract
pub fn load_marketplace_params(
    deps: Deps,
    marketplace_addr: &Addr,
) -> Result<ParamsResponse, ContractError> {
    let marketplace_params: ParamsResponse = deps
        .querier
        .query_wasm_smart(marketplace_addr, &MarketplaceQueryMsg::Params {})?;
    Ok(marketplace_params)
}

/// Load the collection royalties as defined on the NFT collection contract
pub fn load_collection_royalties(
    deps: Deps,
    collection_addr: &Addr,
) -> Result<Option<RoyaltyInfoResponse>, ContractError> {
    let collection_info: CollectionInfoResponse = deps
        .querier
        .query_wasm_smart(collection_addr, &Sg721QueryMsg::CollectionInfo {})?;
    Ok(collection_info.royalty_info)
}

// Check if an Ask is valid for a swap
pub fn validate_ask(
    block: &BlockInfo,
    ask: &Ask,
    bidder: &Addr,
    nft_order: &Option<NftOrder>,
) -> Result<(), ContractError> {
    if ask.is_expired(block) {
        return Err(ContractError::InvalidAsk("expired".to_string()));
    }
    if !ask.is_active {
        return Err(ContractError::InvalidAsk("inactive".to_string()));
    }
    if ask.sale_type == SaleType::Auction {
        return Err(ContractError::InvalidAsk(
            "auction type not supported".to_string(),
        ));
    }
    if let Some(reserved_for) = &ask.reserve_for {
        if reserved_for != bidder {
            return Err(ContractError::InvalidAsk("reserved for bidder".to_string()));
        }
    }
    if nft_order.is_some() && ask.price > nft_order.as_ref().unwrap().amount {
        return Err(ContractError::PriceMismatch("price too high".to_string()));
    }
    Ok(())
}

// Fetch asks in a loop until a certain number are retrieved or there are no more to fetch,
// filters out invalid asks
pub fn fetch_asks(
    deps: Deps,
    marketplace: &Addr,
    collection: &Addr,
    block: &BlockInfo,
    bidder: &Addr,
    limit: u32,
) -> Result<Vec<Ask>, ContractError> {
    let mut asks: Vec<Ask> = vec![];

    loop {
        let start_after = if asks.is_empty() {
            None
        } else {
            let last_ask = asks.last().unwrap();
            Some(AskOffset {
                price: last_ask.price,
                token_id: last_ask.token_id,
            })
        };
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

        for ask in asks_response.asks {
            if validate_ask(block, &ask, bidder, &None).is_err() {
                continue;
            }
            asks.push(ask);
        }

        if num_results < query_limit as usize || asks.len() >= limit as usize {
            break;
        }
    }

    Ok(asks)
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
        let start_after = if collection_bids.is_empty() {
            None
        } else {
            let last_collection_bid = collection_bids.last().unwrap();
            Some(CollectionBidOffset {
                price: last_collection_bid.price,
                collection: last_collection_bid.collection.to_string(),
                bidder: last_collection_bid.bidder.to_string(),
            })
        };
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

        for collection_bid in collection_bids_response.bids {
            if collection_bid.is_expired(block) {
                continue;
            }
            collection_bids.push(collection_bid);
        }

        if num_results < query_limit as usize || collection_bids.len() >= limit as usize {
            break;
        }
    }

    Ok(collection_bids)
}

pub fn build_swap(
    token_id: String,
    sale_price: Uint128,
    token_recipient: String,
    nft_recipient: String,
    finders_fee_bps: u64,
    finder: Option<String>,
    marketplace_params: &SudoParams,
    royalty_info: &Option<RoyaltyInfoResponse>,
) -> Swap {
    let mut token_payments = vec![];

    let network_fee = sale_price * marketplace_params.trading_fee_percent / Uint128::from(100u128);
    let mut seller_payment = sale_price - network_fee;

    if finder.is_some() && finders_fee_bps > 0 {
        let finders_fee = sale_price * Decimal::percent(finders_fee_bps) / Uint128::from(100u128);
        if finders_fee > Uint128::zero() {
            token_payments.push(TokenPayment {
                label: "finder".to_string(),
                address: finder.unwrap().to_string(),
                amount: finders_fee,
            });
            seller_payment -= finders_fee;
        }
    }

    if let Some(_royalty_info) = royalty_info {
        let royalty_fee = sale_price * _royalty_info.share;
        if royalty_fee > Uint128::zero() {
            token_payments.push(TokenPayment {
                label: "royalty".to_string(),
                address: _royalty_info.payment_address.to_string(),
                amount: royalty_fee,
            });
            seller_payment -= royalty_fee;
        }
    }

    token_payments.push(TokenPayment {
        label: "seller".to_string(),
        address: token_recipient,
        amount: seller_payment,
    });

    Swap {
        transaction_type: TransactionType::TokensForNfts,
        sale_price: sale_price,
        network_fee,
        nft_payment: NftPayment {
            address: nft_recipient,
            token_id,
        },
        token_payments,
    }
}
