use cosmwasm_std::{coins, Addr, Timestamp};
use cw_multi_test::{AppResponse, Executor};
use sg_marketplace::{msg::ExecuteMsg, state::SaleType};
use sg_multi_test::StargazeApp;
use sg_std::NATIVE_DENOM;

pub fn set_bid(
    router: &mut StargazeApp,
    marketplace: Addr,
    bidder: Addr,
    bid_amount: u128,
    collection: String,
    token_id: u32,
    expires: Timestamp,
    sale_type: SaleType,
    finder: Option<String>,
    finders_fee_bps: Option<u64>,
) -> AppResponse {
    let set_bid_msg = ExecuteMsg::SetBid {
        collection,
        token_id,
        expires,
        sale_type,
        finders_fee_bps,
        finder,
    };
    let response = router.execute_contract(
        bidder.clone(),
        marketplace.clone(),
        &set_bid_msg,
        &coins(bid_amount, NATIVE_DENOM),
    );
    response.unwrap()
}

pub fn set_collection_bid(
    router: &mut StargazeApp,
    marketplace: Addr,
    bidder: Addr,
    collection: String,
    expires: Timestamp,
    finders_fee_bps: Option<u64>,
    amount: u128,
) -> AppResponse {
    let set_bid_msg = ExecuteMsg::SetCollectionBid {
        collection,
        expires,
        finders_fee_bps,
    };
    let response = router.execute_contract(
        bidder.clone(),
        marketplace.clone(),
        &set_bid_msg,
        &coins(amount, NATIVE_DENOM),
    );
    response.unwrap()
}
