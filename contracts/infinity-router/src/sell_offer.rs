use crate::reply::NFT_TO_TOKEN_REPLY_ID;

use cosmwasm_std::{to_binary, Addr, Binary, SubMsg, Uint128, WasmMsg};
use infinity_index::state::PoolQuote;
use infinity_pool::msg::ExecuteMsg as InfinityPoolExecuteMsg;
use sg_marketplace::msg::ExecuteMsg as MarketplaceExecuteMsg;
use sg_marketplace::state::{Bid, CollectionBid};
use sg_std::Response;

pub enum SellOfferVariant {
    Bid(Bid),
    CollectionBid(CollectionBid),
    PoolQuote(PoolQuote),
}

pub struct SellOffer {
    pub contract: Addr,
    pub token_id: String,
    pub sale_price: Uint128,
    pub variant: SellOfferVariant,
}

impl SellOffer {
    pub fn new(
        contract: Addr,
        token_id: String,
        sale_price: Uint128,
        variant: SellOfferVariant,
    ) -> Self {
        Self {
            contract,
            token_id,
            sale_price,
            variant,
        }
    }

    fn sell_order_binary(&self, finder: Option<Addr>) -> Binary {
        match &self.variant {
            SellOfferVariant::Bid(bid) => to_binary(&MarketplaceExecuteMsg::AcceptBid {
                collection: bid.collection.to_string(),
                token_id: self.token_id.parse::<u32>().unwrap(),
                bidder: bid.bidder.to_string(),
                finder: finder.map(|f| f.to_string()),
            })
            .unwrap(),
            SellOfferVariant::CollectionBid(collection_bid) => {
                to_binary(&MarketplaceExecuteMsg::AcceptCollectionBid {
                    collection: collection_bid.collection.to_string(),
                    token_id: self.token_id.parse::<u32>().unwrap(),
                    bidder: collection_bid.bidder.to_string(),
                    finder: finder.map(|f| f.to_string()),
                })
                .unwrap()
            },
            SellOfferVariant::PoolQuote(_pool_quote) => {
                to_binary(&InfinityPoolExecuteMsg::SwapNftForTokens {
                    token_id: self.token_id.to_string(),
                    min_output: self.sale_price,
                    asset_recipient: None,
                    finder: finder.map(|f| f.to_string()),
                })
                .unwrap()
            },
        }
    }

    pub fn place_sell_order(&self, finder: Option<Addr>, response: Response) -> Response {
        response.add_submessage(SubMsg::reply_always(
            WasmMsg::Execute {
                contract_addr: self.contract.to_string(),
                msg: self.sell_order_binary(finder),
                funds: vec![],
            },
            NFT_TO_TOKEN_REPLY_ID,
        ))
    }
}
