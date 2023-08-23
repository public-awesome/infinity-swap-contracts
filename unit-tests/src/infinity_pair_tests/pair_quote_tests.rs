use crate::helpers::pair_functions::create_pair_with_deposits;
use crate::setup::setup_accounts::MarketAccounts;
use crate::setup::templates::{setup_infinity_test, standard_minter_template, InfinityTestSetup};

use cosmwasm_std::{Addr, Decimal, Uint128};
use infinity_global::{GlobalConfig, QueryMsg as InfinityGlobalQueryMsg};
use infinity_pair::msg::{QueryMsg as InfinityPairQueryMsg, QuotesResponse};
use infinity_pair::state::{BondingCurve, PairConfig, PairType};
use sg_std::NATIVE_DENOM;
use test_suite::common_setup::msg::MinterTemplateResponse;

#[test]
fn try_generate_quotes_token_linear() {
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
        infinity_factory,
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

    let mut spot_price = Uint128::from(10_000_000u128);
    let delta = Uint128::from(1_000_000u128);
    let mut remaining_amount = Uint128::from(100_000_000u128);

    let test_pair = create_pair_with_deposits(
        &mut router,
        &infinity_global,
        &infinity_factory,
        &minter,
        &collection,
        &creator,
        &owner,
        PairConfig {
            pair_type: PairType::Token {},
            bonding_curve: BondingCurve::Linear {
                spot_price,
                delta,
            },
            is_active: true,
            asset_recipient: None,
        },
        0u64,
        remaining_amount,
    );

    let quotes_response = router
        .wrap()
        .query_wasm_smart::<QuotesResponse>(
            test_pair.address,
            &InfinityPairQueryMsg::SellToPairQuotes {
                limit: u32::MAX,
            },
        )
        .unwrap();

    let mut expected_quotes = vec![];
    while spot_price <= remaining_amount && spot_price >= delta {
        let quote = spot_price
            - spot_price.mul_ceil(global_config.fair_burn_fee_percent)
            - spot_price.mul_ceil(global_config.max_royalty_fee_percent);
        expected_quotes.push(quote);
        remaining_amount -= spot_price;
        spot_price -= delta;
    }

    assert_eq!(quotes_response.denom, NATIVE_DENOM.to_string());
    assert_eq!(quotes_response.quotes, expected_quotes);
}

#[test]
fn try_generate_quotes_token_exponential() {
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
        infinity_factory,
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

    let mut spot_price = Uint128::from(10_000_000u128);
    let delta = Decimal::percent(5);
    let mut remaining_amount = Uint128::from(100_000_000u128);

    let test_pair = create_pair_with_deposits(
        &mut router,
        &infinity_global,
        &infinity_factory,
        &minter,
        &collection,
        &creator,
        &owner,
        PairConfig {
            pair_type: PairType::Token {},
            bonding_curve: BondingCurve::Exponential {
                spot_price,
                delta,
            },
            is_active: true,
            asset_recipient: None,
        },
        0u64,
        remaining_amount,
    );

    let quotes_response = router
        .wrap()
        .query_wasm_smart::<QuotesResponse>(
            test_pair.address,
            &InfinityPairQueryMsg::SellToPairQuotes {
                limit: u32::MAX,
            },
        )
        .unwrap();

    let mut expected_quotes = vec![];
    while spot_price <= remaining_amount {
        let quote = spot_price
            - spot_price.mul_ceil(global_config.fair_burn_fee_percent)
            - spot_price.mul_ceil(global_config.max_royalty_fee_percent);
        expected_quotes.push(quote);
        remaining_amount -= spot_price;
        spot_price = spot_price.checked_div_floor(Decimal::one() + delta).unwrap();
    }

    assert_eq!(quotes_response.denom, NATIVE_DENOM.to_string());
    assert_eq!(quotes_response.quotes, expected_quotes);
}

#[test]
fn try_generate_quotes_nft_linear() {
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
        infinity_factory,
        ..
    } = setup_infinity_test(vt).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let mut spot_price = Uint128::from(10_000_000u128);
    let delta = Uint128::from(1_000_000u128);
    let mut num_nfts = 100u64;

    let test_pair = create_pair_with_deposits(
        &mut router,
        &infinity_global,
        &infinity_factory,
        &minter,
        &collection,
        &creator,
        &owner,
        PairConfig {
            pair_type: PairType::Nft {},
            bonding_curve: BondingCurve::Linear {
                spot_price,
                delta,
            },
            is_active: true,
            asset_recipient: None,
        },
        num_nfts,
        Uint128::zero(),
    );

    let quotes_response = router
        .wrap()
        .query_wasm_smart::<QuotesResponse>(
            test_pair.address,
            &InfinityPairQueryMsg::BuyFromPairQuotes {
                limit: u32::MAX,
            },
        )
        .unwrap();

    let mut expected_quotes = vec![];
    while num_nfts > 0 {
        expected_quotes.push(spot_price);
        spot_price += delta;
        num_nfts -= 1;
    }

    assert_eq!(quotes_response.denom, NATIVE_DENOM.to_string());
    assert_eq!(quotes_response.quotes, expected_quotes);
}

#[test]
fn try_generate_quotes_nft_exponential() {
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
        infinity_factory,
        ..
    } = setup_infinity_test(vt).unwrap();

    let collection_resp = &collection_response_vec[0];
    let minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let mut spot_price = Uint128::from(10_000_000u128);
    let delta = Decimal::percent(5);
    let mut num_nfts = 100u64;

    let test_pair = create_pair_with_deposits(
        &mut router,
        &infinity_global,
        &infinity_factory,
        &minter,
        &collection,
        &creator,
        &owner,
        PairConfig {
            pair_type: PairType::Nft {},
            bonding_curve: BondingCurve::Exponential {
                spot_price,
                delta,
            },
            is_active: true,
            asset_recipient: None,
        },
        num_nfts,
        Uint128::zero(),
    );

    let quotes_response = router
        .wrap()
        .query_wasm_smart::<QuotesResponse>(
            test_pair.address,
            &InfinityPairQueryMsg::BuyFromPairQuotes {
                limit: u32::MAX,
            },
        )
        .unwrap();

    let mut expected_quotes = vec![];
    while num_nfts > 0 {
        expected_quotes.push(spot_price);
        spot_price = spot_price.mul_ceil(Decimal::one() + delta);
        num_nfts -= 1;
    }

    assert_eq!(quotes_response.denom, NATIVE_DENOM.to_string());
    assert_eq!(quotes_response.quotes, expected_quotes);
}

#[test]
fn try_generate_quotes_trade_linear() {
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
        infinity_factory,
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

    let swap_fee_percent = Decimal::percent(1);
    let original_spot_price = Uint128::from(10_000_000u128);
    let delta = Uint128::from(1_000_000u128);
    let mut num_nfts = 100u64;
    let mut remaining_amount = Uint128::from(100_000_000u128);

    let test_pair = create_pair_with_deposits(
        &mut router,
        &infinity_global,
        &infinity_factory,
        &minter,
        &collection,
        &creator,
        &owner,
        PairConfig {
            pair_type: PairType::Trade {
                swap_fee_percent,
                reinvest_tokens: false,
                reinvest_nfts: false,
            },
            bonding_curve: BondingCurve::Linear {
                spot_price: original_spot_price,
                delta,
            },
            is_active: true,
            asset_recipient: None,
        },
        num_nfts,
        remaining_amount,
    );

    let quotes_response = router
        .wrap()
        .query_wasm_smart::<QuotesResponse>(
            test_pair.address.clone(),
            &InfinityPairQueryMsg::SellToPairQuotes {
                limit: u32::MAX,
            },
        )
        .unwrap();

    let mut expected_quotes = vec![];
    let mut spot_price = original_spot_price;
    while spot_price <= remaining_amount && spot_price >= delta {
        let quote = spot_price
            - spot_price.mul_ceil(global_config.fair_burn_fee_percent)
            - spot_price.mul_ceil(global_config.max_royalty_fee_percent)
            - spot_price.mul_ceil(swap_fee_percent);
        expected_quotes.push(quote);
        remaining_amount -= spot_price;
        spot_price -= delta;
    }

    assert_eq!(quotes_response.denom, NATIVE_DENOM.to_string());
    assert_eq!(quotes_response.quotes, expected_quotes);

    let quotes_response = router
        .wrap()
        .query_wasm_smart::<QuotesResponse>(
            test_pair.address,
            &InfinityPairQueryMsg::BuyFromPairQuotes {
                limit: u32::MAX,
            },
        )
        .unwrap();

    let mut expected_quotes = vec![];
    let mut spot_price = original_spot_price + delta;
    while num_nfts > 0 {
        expected_quotes.push(spot_price);
        spot_price += delta;
        num_nfts -= 1;
    }

    assert_eq!(quotes_response.denom, NATIVE_DENOM.to_string());
    assert_eq!(quotes_response.quotes, expected_quotes);
}

#[test]
fn try_generate_quotes_trade_exponential() {
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
        infinity_factory,
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

    let swap_fee_percent = Decimal::percent(1);
    let original_spot_price = Uint128::from(10_000_000u128);
    let delta = Decimal::percent(5);
    let mut num_nfts = 100u64;
    let mut remaining_amount = Uint128::from(100_000_000u128);

    let test_pair = create_pair_with_deposits(
        &mut router,
        &infinity_global,
        &infinity_factory,
        &minter,
        &collection,
        &creator,
        &owner,
        PairConfig {
            pair_type: PairType::Trade {
                swap_fee_percent,
                reinvest_tokens: false,
                reinvest_nfts: false,
            },
            bonding_curve: BondingCurve::Exponential {
                spot_price: original_spot_price,
                delta,
            },
            is_active: true,
            asset_recipient: None,
        },
        num_nfts,
        remaining_amount,
    );

    let quotes_response = router
        .wrap()
        .query_wasm_smart::<QuotesResponse>(
            test_pair.address.clone(),
            &InfinityPairQueryMsg::SellToPairQuotes {
                limit: u32::MAX,
            },
        )
        .unwrap();

    let mut expected_quotes = vec![];
    let mut spot_price = original_spot_price;
    while spot_price <= remaining_amount {
        let quote = spot_price
            - spot_price.mul_ceil(global_config.fair_burn_fee_percent)
            - spot_price.mul_ceil(global_config.max_royalty_fee_percent)
            - spot_price.mul_ceil(swap_fee_percent);
        expected_quotes.push(quote);
        remaining_amount -= spot_price;
        spot_price = spot_price.checked_div_floor(Decimal::one() + delta).unwrap();
    }

    assert_eq!(quotes_response.denom, NATIVE_DENOM.to_string());
    assert_eq!(quotes_response.quotes, expected_quotes);

    let quotes_response = router
        .wrap()
        .query_wasm_smart::<QuotesResponse>(
            test_pair.address,
            &InfinityPairQueryMsg::BuyFromPairQuotes {
                limit: u32::MAX,
            },
        )
        .unwrap();

    let mut expected_quotes = vec![];
    let mut spot_price = original_spot_price.mul_ceil(Decimal::one() + delta);
    while num_nfts > 0 {
        expected_quotes.push(spot_price);
        spot_price = spot_price.mul_ceil(Decimal::one() + delta);
        num_nfts -= 1;
    }

    assert_eq!(quotes_response.denom, NATIVE_DENOM.to_string());
    assert_eq!(quotes_response.quotes, expected_quotes);
}

#[test]
fn try_generate_quotes_trade_cp() {
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
        infinity_factory,
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

    let swap_fee_percent = Decimal::percent(1);
    let original_num_nfts = 100u64;
    let original_remaining_amount = Uint128::from(100_000_000u128);

    let test_pair = create_pair_with_deposits(
        &mut router,
        &infinity_global,
        &infinity_factory,
        &minter,
        &collection,
        &creator,
        &owner,
        PairConfig {
            pair_type: PairType::Trade {
                swap_fee_percent,
                reinvest_tokens: false,
                reinvest_nfts: false,
            },
            bonding_curve: BondingCurve::ConstantProduct {},
            is_active: true,
            asset_recipient: None,
        },
        original_num_nfts,
        original_remaining_amount,
    );

    let limit = 100u32;
    let quotes_response = router
        .wrap()
        .query_wasm_smart::<QuotesResponse>(
            test_pair.address.clone(),
            &InfinityPairQueryMsg::SellToPairQuotes {
                limit,
            },
        )
        .unwrap();

    let mut expected_quotes = vec![];
    let mut remaining_amount = original_remaining_amount;
    let mut spot_price =
        remaining_amount.div_floor((Uint128::from(original_num_nfts + 1), Uint128::one()));
    let mut counter = 0;
    while counter < limit {
        let quote = spot_price
            - spot_price.mul_ceil(global_config.fair_burn_fee_percent)
            - spot_price.mul_ceil(global_config.max_royalty_fee_percent)
            - spot_price.mul_ceil(swap_fee_percent);
        expected_quotes.push(quote);
        remaining_amount -= spot_price;
        spot_price =
            remaining_amount.div_floor((Uint128::from(original_num_nfts + 1), Uint128::one()));
        counter += 1;
    }

    assert_eq!(quotes_response.denom, NATIVE_DENOM.to_string());
    assert_eq!(quotes_response.quotes, expected_quotes);

    let quotes_response = router
        .wrap()
        .query_wasm_smart::<QuotesResponse>(
            test_pair.address,
            &InfinityPairQueryMsg::BuyFromPairQuotes {
                limit: u32::MAX,
            },
        )
        .unwrap();

    let mut expected_quotes = vec![];
    let mut num_nfts = original_num_nfts;
    let mut spot_price: Uint128;
    while num_nfts > 1 {
        spot_price =
            original_remaining_amount.div_ceil((Uint128::from(num_nfts - 1), Uint128::one()));
        expected_quotes.push(spot_price);
        num_nfts -= 1;
    }

    assert_eq!(quotes_response.denom, NATIVE_DENOM.to_string());
    assert_eq!(quotes_response.quotes, expected_quotes);
}
