use cosmwasm_std::{
    to_binary, Addr, ContractResult, Decimal, Querier, QuerierResult, QuerierWrapper, SystemResult,
    Uint128,
};
use cw_utils::Duration;
use infinity_shared::shared::load_marketplace_params;
use mockall::{mock, predicate};
use sg_marketplace::msg::ParamsResponse;
use sg_marketplace::state::SudoParams;
use sg_marketplace::ExpiryRange;

#[test]
fn try_load_marketplace_params() {
    mock! {
        QuerierStruct {}
        impl Querier for QuerierStruct {
            fn raw_query(&self, bin_request: &[u8]) -> QuerierResult;
        }
    }

    let listing_fee: u128 = 10;
    let trading_fee_bps = 200;
    let min_expiry: u64 = 24 * 60 * 60; // 24 hours (in seconds)
    let max_expiry: u64 = 180 * 24 * 60 * 60; // 6 months (in seconds)
    let max_finders_fee_bps: u64 = 1000; // 10%
    let bid_removal_reward_bps: u64 = 500; // 5%

    let sudo_params = SudoParams {
        trading_fee_percent: Decimal::percent(trading_fee_bps),
        ask_expiry: ExpiryRange {
            min: min_expiry,
            max: max_expiry,
        },
        bid_expiry: ExpiryRange {
            min: min_expiry,
            max: max_expiry,
        },
        operators: vec![Addr::unchecked("operator")],
        max_finders_fee_percent: Decimal::percent(max_finders_fee_bps),
        min_price: Uint128::from(5u128),
        stale_bid_duration: Duration::Time(100),
        bid_removal_reward_percent: Decimal::percent(bid_removal_reward_bps),
        listing_fee: Uint128::from(listing_fee),
    };

    let return_value = SystemResult::Ok(ContractResult::Ok(
        to_binary(&ParamsResponse {
            params: sudo_params.clone(),
        })
        .unwrap(),
    ));

    let mut mock = MockQuerierStruct::new();
    mock.expect_raw_query()
        .with(predicate::always())
        .times(1)
        .returning(move |_| return_value.clone());

    let querier_wrapper = QuerierWrapper::new(&mock);

    assert_eq!(
        Ok(sudo_params),
        load_marketplace_params(&querier_wrapper, &Addr::unchecked("marketplace"))
    );
}
