use crate::buy_offer::{BuyOffer, BuyOfferVariant};
use crate::msg::NftOrder;
use crate::sell_offer::{SellOffer, SellOfferVariant};
use crate::ContractError;
use crate::{msg::SwapParams, state::SwapContext};

use cosmwasm_std::{Addr, Api, Empty, Env, MessageInfo, QuerierWrapper, Uint128};
use cw721_base::helpers::Cw721Contract;
use cw_utils::maybe_addr;
use infinity_index::msg::{PoolQuotesResponse, QueryMsg as InfinityIndexQueryMsg};
use infinity_index::state::PoolQuote;
use infinity_pool::msg::{
    NftDepositsResponse, PoolQuoteResponse, QueryMsg as InfinityPoolQueryMsg,
};
use infinity_shared::query::QueryOptions;
use sg_marketplace::msg::{
    AskResponse, AsksResponse, BidsResponse, CollectionBidsResponse,
    QueryMsg as MarketplaceQueryMsg,
};
use std::marker::PhantomData;

pub fn only_self_callable(info: &MessageInfo, env: &Env) -> Result<(), ContractError> {
    if info.sender != env.contract.address {
        Err(ContractError::OnlySelf)
    } else {
        Ok(())
    }
}

pub fn build_swap_context(
    api: &dyn Api,
    info: &MessageInfo,
    collection: String,
    swap_params: SwapParams,
) -> Result<SwapContext, ContractError> {
    Ok(SwapContext {
        original_sender: info.sender.clone(),
        collection: api.addr_validate(&collection)?,
        robust: swap_params.robust,
        asset_recipient: maybe_addr(api, swap_params.asset_recipient)?,
        finder: maybe_addr(api, swap_params.finder)?,
        balance: Uint128::zero(),
    })
}

pub fn find_highest_sell_to_offer(
    querier: &QuerierWrapper,
    infinity_index: &Addr,
    marketplace: &Addr,
    collection: &Addr,
    nft_order: &NftOrder,
) -> Result<Option<SellOffer>, ContractError> {
    let mut sell_offers: Vec<SellOffer> = vec![];

    let mut bids_response: BidsResponse = querier.query_wasm_smart(
        marketplace,
        &MarketplaceQueryMsg::BidsSortedByTokenPrice {
            collection: collection.to_string(),
            token_id: nft_order.token_id.parse::<u32>().unwrap(),
            start_after: None,
            limit: Some(1),
            descending: Some(true),
            include_expired: Some(false),
        },
    )?;

    if let Some(bid) = bids_response.bids.pop() {
        sell_offers.push(SellOffer {
            contract: marketplace.clone(),
            token_id: bid.token_id.to_string(),
            sale_price: bid.price,
            variant: SellOfferVariant::Bid(bid),
        });
    }

    let mut collection_bids_response: CollectionBidsResponse = querier.query_wasm_smart(
        marketplace,
        &MarketplaceQueryMsg::CollectionBidsSortedByPrice {
            collection: collection.to_string(),
            start_after: None,
            limit: Some(1),
        },
    )?;

    if let Some(collection_bid) = collection_bids_response.bids.pop() {
        sell_offers.push(SellOffer {
            contract: marketplace.clone(),
            token_id: nft_order.token_id.to_string(),
            sale_price: collection_bid.price,
            variant: SellOfferVariant::CollectionBid(collection_bid),
        });
    }

    let mut pool_quotes_response: PoolQuotesResponse = querier.query_wasm_smart(
        infinity_index,
        &InfinityIndexQueryMsg::SellToPoolQuotes {
            collection: collection.to_string(),
            query_options: Some(QueryOptions {
                descending: Some(true),
                limit: Some(1),
                start_after: None,
            }),
        },
    )?;

    if let Some(pool_quote) = pool_quotes_response.pool_quotes.pop() {
        sell_offers.push(SellOffer {
            contract: pool_quote.pool.clone(),
            token_id: nft_order.token_id.to_string(),
            sale_price: pool_quote.quote_price,
            variant: SellOfferVariant::PoolQuote(pool_quote),
        });
    }

    let best_offer = sell_offers.into_iter().max_by_key(|so| so.sale_price);

    if let Some(best_offer) = best_offer {
        if best_offer.sale_price >= nft_order.amount {
            return Ok(Some(best_offer));
        }
    }
    return Ok(None);
}

pub fn find_lowest_buy_specific_nft_offer(
    querier: &QuerierWrapper,
    infinity_factory: &Addr,
    marketplace: &Addr,
    collection: &Addr,
    nft_order: &NftOrder,
) -> Result<Option<BuyOffer>, ContractError> {
    // Check if the owner of the NFT is an infinity pool
    let owner_of = Cw721Contract::<Empty, Empty>(collection.clone(), PhantomData, PhantomData)
        .owner_of(&querier, nft_order.token_id.to_string(), false)?;
    let contract_info = querier.query_wasm_contract_info(&owner_of.owner);

    let buy_offer = if contract_info.is_ok()
        && contract_info.unwrap().creator == infinity_factory.to_string()
    {
        let infinity_pool = Addr::unchecked(owner_of.owner);

        let pool_quote_response: PoolQuoteResponse =
            querier.query_wasm_smart(&infinity_pool, &InfinityPoolQueryMsg::BuyFromPoolQuote {})?;

        let buy_offer = pool_quote_response.quote_price.map(|qp| BuyOffer {
            contract: infinity_pool.clone(),
            token_id: nft_order.token_id.to_string(),
            sale_price: qp,
            variant: BuyOfferVariant::PoolQuote(PoolQuote {
                pool: infinity_pool,
                collection: collection.clone(),
                quote_price: qp,
            }),
        });
        buy_offer
    } else {
        let ask_response: AskResponse = querier.query_wasm_smart(
            marketplace,
            &MarketplaceQueryMsg::Ask {
                collection: collection.to_string(),
                token_id: nft_order.token_id.parse::<u32>().unwrap(),
            },
        )?;

        let buy_offer = ask_response.ask.map(|ask| BuyOffer {
            contract: marketplace.clone(),
            token_id: nft_order.token_id.to_string(),
            sale_price: ask.price,
            variant: BuyOfferVariant::Ask(ask),
        });
        buy_offer
    };

    if let Some(buy_offer) = buy_offer {
        if buy_offer.sale_price <= nft_order.amount {
            return Ok(Some(buy_offer));
        }
    }
    return Ok(None);
}

pub fn find_lowest_buy_any_nft_offer(
    querier: &QuerierWrapper,
    marketplace: &Addr,
    infinity_index: &Addr,
    collection: &Addr,
    max_input: Uint128,
) -> Result<Option<BuyOffer>, ContractError> {
    let mut buy_offers: Vec<BuyOffer> = vec![];

    let mut asks_response: AsksResponse = querier.query_wasm_smart(
        marketplace,
        &MarketplaceQueryMsg::AsksSortedByPrice {
            collection: collection.to_string(),
            include_inactive: Some(false),
            start_after: None,
            limit: Some(1),
        },
    )?;

    if let Some(ask) = asks_response.asks.pop() {
        buy_offers.push(BuyOffer {
            contract: marketplace.clone(),
            token_id: ask.token_id.to_string(),
            sale_price: ask.price,
            variant: BuyOfferVariant::Ask(ask),
        });
    }

    let mut pool_quotes_response: PoolQuotesResponse = querier.query_wasm_smart(
        infinity_index,
        &InfinityIndexQueryMsg::BuyFromPoolQuotes {
            collection: collection.to_string(),
            query_options: Some(QueryOptions {
                descending: Some(false),
                limit: Some(1),
                start_after: None,
            }),
        },
    )?;

    if let Some(pool_quote) = pool_quotes_response.pool_quotes.pop() {
        let mut nft_deposits_response: NftDepositsResponse = querier.query_wasm_smart(
            pool_quote.pool.clone(),
            &InfinityPoolQueryMsg::NftDeposits {
                query_options: Some(QueryOptions {
                    descending: None,
                    start_after: None,
                    limit: Some(1),
                }),
            },
        )?;
        let token_id = nft_deposits_response.nft_deposits.pop().unwrap();
        buy_offers.push(BuyOffer {
            contract: pool_quote.pool.clone(),
            token_id: token_id,
            sale_price: pool_quote.quote_price,
            variant: BuyOfferVariant::PoolQuote(pool_quote),
        });
    }

    let best_offer = buy_offers.into_iter().min_by_key(|so| so.sale_price);

    if let Some(best_offer) = best_offer {
        if best_offer.sale_price <= max_input {
            return Ok(Some(best_offer));
        }
    }
    return Ok(None);
}
