use crate::helpers::pair_functions::create_pair_with_deposits;
use crate::setup::setup_accounts::MarketAccounts;
use crate::setup::templates::{setup_infinity_test, standard_minter_template, InfinityTestSetup};

use cosmwasm_std::{Addr, Decimal, Uint128};
use infinity_factory::msg::QueryMsg as InfinityFactoryQueryMsg;
use infinity_global::{GlobalConfig, QueryMsg as InfinityGlobalQueryMsg};
use infinity_pair::msg::{QueryMsg as InfinityPairQueryMsg, QuotesResponse};
use infinity_pair::state::{BondingCurve, PairConfig, PairType};
use test_suite::common_setup::msg::MinterTemplateResponse;

#[test]
fn try_sim_sell_to_pair_quotes() {
    let vt = standard_minter_template(1000u32);
    let InfinityTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts:
                    MarketAccounts {
                        creator,
                        owner,
                        ..
                    },
            },
        infinity_global,
        ..
    } = setup_infinity_test(vt).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let global_config = router
        .wrap()
        .query_wasm_smart::<GlobalConfig<Addr>>(
            infinity_global.clone(),
            &InfinityGlobalQueryMsg::GlobalConfig {},
        )
        .unwrap();

    let test_pair = create_pair_with_deposits(
        &mut router,
        &infinity_global,
        &global_config.infinity_factory,
        &minter,
        &collection,
        &creator,
        &owner,
        PairConfig {
            pair_type: PairType::Trade {
                swap_fee_percent: Decimal::percent(2),
                reinvest_nfts: true,
                reinvest_tokens: true,
            },
            bonding_curve: BondingCurve::ConstantProduct,
            is_active: false,
            asset_recipient: None,
        },
        20u64,
        Uint128::from(100_000_000u128),
    );

    let sell_to_pair_quotes = router
        .wrap()
        .query_wasm_smart::<QuotesResponse>(
            test_pair.address.clone(),
            &InfinityPairQueryMsg::SellToPairQuotes {
                limit: 100,
            },
        )
        .unwrap();

    let sim_sell_to_pair_quotes = router
        .wrap()
        .query_wasm_smart::<QuotesResponse>(
            global_config.infinity_factory.clone(),
            &InfinityFactoryQueryMsg::SimSellToPairQuotes {
                pair: test_pair.pair,
                limit: 100,
            },
        )
        .unwrap();

    assert_eq!(sell_to_pair_quotes, sim_sell_to_pair_quotes);
}

#[test]
fn try_sim_buy_from_pair_quotes() {
    let vt = standard_minter_template(1000u32);
    let InfinityTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts:
                    MarketAccounts {
                        creator,
                        owner,
                        ..
                    },
            },
        infinity_global,
        ..
    } = setup_infinity_test(vt).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let global_config = router
        .wrap()
        .query_wasm_smart::<GlobalConfig<Addr>>(
            infinity_global.clone(),
            &InfinityGlobalQueryMsg::GlobalConfig {},
        )
        .unwrap();

    let test_pair = create_pair_with_deposits(
        &mut router,
        &infinity_global,
        &global_config.infinity_factory,
        &minter,
        &collection,
        &creator,
        &owner,
        PairConfig {
            pair_type: PairType::Trade {
                swap_fee_percent: Decimal::percent(2),
                reinvest_nfts: true,
                reinvest_tokens: true,
            },
            bonding_curve: BondingCurve::ConstantProduct,
            is_active: false,
            asset_recipient: None,
        },
        20u64,
        Uint128::from(100_000_000u128),
    );

    let buy_from_pair_quotes = router
        .wrap()
        .query_wasm_smart::<QuotesResponse>(
            test_pair.address.clone(),
            &InfinityPairQueryMsg::BuyFromPairQuotes {
                limit: 100,
            },
        )
        .unwrap();

    let sim_buy_from_pair_quotes = router
        .wrap()
        .query_wasm_smart::<QuotesResponse>(
            global_config.infinity_factory.clone(),
            &InfinityFactoryQueryMsg::SimBuyFromPairQuotes {
                pair: test_pair.pair,
                limit: 100,
            },
        )
        .unwrap();

    assert_eq!(buy_from_pair_quotes, sim_buy_from_pair_quotes);
}
