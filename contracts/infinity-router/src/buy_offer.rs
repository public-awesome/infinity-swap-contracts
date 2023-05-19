use crate::reply::{TOKEN_TO_ANY_NFT_REPLY_ID, TOKEN_TO_SPECIFIC_NFT_REPLY_ID};

use cosmwasm_std::{coin, to_binary, Addr, Binary, SubMsg, Uint128, WasmMsg};
use infinity_index::state::PoolQuote;
use infinity_pool::msg::ExecuteMsg as InfinityPoolExecuteMsg;
use sg_marketplace::msg::ExecuteMsg as MarketplaceExecuteMsg;
use sg_marketplace::state::Ask;
use sg_std::{Response, NATIVE_DENOM};

pub enum BuyOfferVariant {
    Ask(Ask),
    PoolQuote(PoolQuote),
}

pub struct BuyOffer {
    pub contract: Addr,
    pub token_id: String,
    pub sale_price: Uint128,
    pub variant: BuyOfferVariant,
}

impl BuyOffer {
    pub fn new(
        contract: Addr,
        token_id: String,
        sale_price: Uint128,
        variant: BuyOfferVariant,
    ) -> Self {
        Self {
            contract,
            token_id,
            sale_price,
            variant,
        }
    }

    fn buy_order_binary(&self, finder: Option<Addr>) -> Binary {
        match &self.variant {
            BuyOfferVariant::Ask(ask) => to_binary(&MarketplaceExecuteMsg::BuyNow {
                collection: ask.collection.to_string(),
                token_id: self.token_id.parse::<u32>().unwrap(),
                expires: ask.expires_at,
                finder: finder.map(|f| f.to_string()),
                finders_fee_bps: None,
            })
            .unwrap(),
            BuyOfferVariant::PoolQuote(_pool_quote) => {
                to_binary(&InfinityPoolExecuteMsg::SwapTokensForNft {
                    token_id: self.token_id.to_string(),
                    max_input: self.sale_price,
                    asset_recipient: None,
                    finder: finder.map(|f| f.to_string()),
                })
                .unwrap()
            },
        }
    }

    pub fn place_buy_order_for_specific_nft(
        &self,
        finder: Option<Addr>,
        response: Response,
    ) -> Response {
        response.add_submessage(SubMsg::reply_always(
            WasmMsg::Execute {
                contract_addr: self.contract.to_string(),
                msg: self.buy_order_binary(finder),
                funds: vec![coin(self.sale_price.u128(), NATIVE_DENOM)],
            },
            TOKEN_TO_SPECIFIC_NFT_REPLY_ID,
        ))
    }

    pub fn place_buy_order_for_any_nft(
        &self,
        finder: Option<Addr>,
        response: Response,
    ) -> Response {
        response.add_submessage(SubMsg::reply_always(
            WasmMsg::Execute {
                contract_addr: self.contract.to_string(),
                msg: self.buy_order_binary(finder),
                funds: vec![coin(self.sale_price.u128(), NATIVE_DENOM)],
            },
            TOKEN_TO_ANY_NFT_REPLY_ID,
        ))
    }
}
