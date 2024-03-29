use crate::helpers::nft_functions::{approve, assert_nft_owner, mint_to};
use crate::helpers::pair_functions::create_pair_with_deposits;
use crate::helpers::utils::assert_error;
use crate::setup::setup_accounts::{setup_addtl_account, MarketAccounts, INITIAL_BALANCE};
use crate::setup::setup_infinity_contracts::UOSMO;
use crate::setup::templates::{setup_infinity_test, standard_minter_template, InfinityTestSetup};

use cosmwasm_std::{coin, Addr, Decimal, Uint128};
use cw_multi_test::Executor;
use infinity_global::{msg::QueryMsg as InfinityGlobalQueryMsg, GlobalConfig};
use infinity_pair::msg::{ExecuteMsg as InfinityPairExecuteMsg, QueryMsg as InfinityPairQueryMsg};
use infinity_pair::pair::Pair;
use infinity_pair::state::{BondingCurve, PairConfig, PairType, QuoteSummary, TokenPayment};
use infinity_pair::ContractError;
use infinity_shared::InfinityError;
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg_std::NATIVE_DENOM;
use test_suite::common_setup::msg::MinterTemplateResponse;

#[test]
fn try_nft_pair_invalid_swaps() {
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
                        bidder,
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

    let collection_info = router
        .wrap()
        .query_wasm_smart::<CollectionInfoResponse>(
            collection.clone(),
            &Sg721QueryMsg::CollectionInfo {},
        )
        .unwrap();

    let mut test_pair = create_pair_with_deposits(
        &mut router,
        &infinity_global,
        &infinity_factory,
        &minter,
        &collection,
        &creator,
        &owner,
        PairConfig {
            pair_type: PairType::Nft,
            bonding_curve: BondingCurve::Linear {
                spot_price: Uint128::from(10_000_000u128),
                delta: Uint128::from(1_000_000u128),
            },
            is_active: false,
            asset_recipient: None,
        },
        10u64,
        Uint128::zero(),
    );

    assert_eq!(test_pair.pair.internal.sell_to_pair_quote_summary, None);
    assert_eq!(test_pair.pair.internal.buy_from_pair_quote_summary, None);

    let token_id = test_pair.token_ids[0].clone();

    // Cannot swap with inactive pair
    let response = router.execute_contract(
        bidder.clone(),
        test_pair.address.clone(),
        &InfinityPairExecuteMsg::SwapTokensForSpecificNft {
            token_id,
            asset_recipient: None,
        },
        &[coin(10_000_000u128, NATIVE_DENOM)],
    );
    assert_error(response, ContractError::InvalidPair("pair is inactive".to_string()).to_string());

    // Set pair to active
    let response = router.execute_contract(
        owner.clone(),
        test_pair.address.clone(),
        &InfinityPairExecuteMsg::UpdatePairConfig {
            is_active: Some(true),
            pair_type: None,
            bonding_curve: None,
            asset_recipient: None,
        },
        &[],
    );
    assert!(response.is_ok());

    test_pair.pair = router
        .wrap()
        .query_wasm_smart::<Pair>(test_pair.address.clone(), &InfinityPairQueryMsg::Pair {})
        .unwrap();

    assert_eq!(test_pair.pair.internal.sell_to_pair_quote_summary, None);
    assert_eq!(
        test_pair.pair.internal.buy_from_pair_quote_summary,
        Some(QuoteSummary {
            fair_burn: TokenPayment {
                recipient: global_config.fair_burn,
                amount: Uint128::from(100_000u128),
            },
            royalty: Some(TokenPayment {
                recipient: Addr::unchecked(
                    collection_info.royalty_info.as_ref().unwrap().payment_address.clone()
                ),
                amount: Uint128::from(500_000u128),
            }),
            swap: None,
            seller_amount: Uint128::from(10_000_000u128),
        })
    );

    // Cannot do a NFT to token swap with NFT pair
    let seller = setup_addtl_account(&mut router, "seller", INITIAL_BALANCE).unwrap();
    let token_id = mint_to(&mut router, &creator.clone(), &seller.clone(), &minter);
    approve(&mut router, &seller, &collection, &test_pair.address, token_id.clone());
    let response = router.execute_contract(
        seller.clone(),
        test_pair.address.clone(),
        &InfinityPairExecuteMsg::SwapNftForTokens {
            token_id: token_id.clone(),
            min_output: coin(9_400_000u128, NATIVE_DENOM),
            asset_recipient: None,
        },
        &[],
    );
    assert_error(
        response,
        ContractError::InvalidPair("pair cannot produce quote".to_string()).to_string(),
    );

    // Cannot swap with insufficient funds
    let response = router.execute_contract(
        bidder.clone(),
        test_pair.address.clone(),
        &InfinityPairExecuteMsg::SwapTokensForSpecificNft {
            token_id: token_id.clone(),
            asset_recipient: None,
        },
        &[coin(1, NATIVE_DENOM)],
    );
    assert_error(
        response,
        InfinityError::InvalidInput("received funds does not equal quote".to_string()).to_string(),
    );

    // Cannot swap using alt denom funds
    let response = router.execute_contract(
        bidder.clone(),
        test_pair.address.clone(),
        &InfinityPairExecuteMsg::SwapTokensForSpecificNft {
            token_id,
            asset_recipient: None,
        },
        &[coin(10_600_000u128, UOSMO)],
    );
    assert_error(response, "Must send reserve token 'ustars'".to_string());

    // Cannot swap for unnowned NFT
    let response = router.execute_contract(
        bidder,
        test_pair.address,
        &InfinityPairExecuteMsg::SwapTokensForSpecificNft {
            token_id: "99999".to_string(),
            asset_recipient: None,
        },
        &[coin(10_600_000u128, NATIVE_DENOM)],
    );
    assert_error(
        response,
        InfinityError::InvalidInput("pair does not own NFT".to_string()).to_string(),
    );
}

#[test]
fn try_nft_pair_linear_user_submits_tokens_swap() {
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
                        bidder,
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

    let collection_info = router
        .wrap()
        .query_wasm_smart::<CollectionInfoResponse>(
            collection.clone(),
            &Sg721QueryMsg::CollectionInfo {},
        )
        .unwrap();

    let mut test_pair = create_pair_with_deposits(
        &mut router,
        &infinity_global,
        &infinity_factory,
        &minter,
        &collection,
        &creator,
        &owner,
        PairConfig {
            pair_type: PairType::Nft,
            bonding_curve: BondingCurve::Linear {
                spot_price: Uint128::from(10_000_000u128),
                delta: Uint128::from(1_000_000u128),
            },
            is_active: true,
            asset_recipient: None,
        },
        10u64,
        Uint128::zero(),
    );

    assert_eq!(test_pair.pair.internal.sell_to_pair_quote_summary, None);
    assert_eq!(
        test_pair.pair.internal.buy_from_pair_quote_summary,
        Some(QuoteSummary {
            fair_burn: TokenPayment {
                recipient: global_config.fair_burn.clone(),
                amount: Uint128::from(100_000u128),
            },
            royalty: Some(TokenPayment {
                recipient: Addr::unchecked(
                    collection_info.royalty_info.as_ref().unwrap().payment_address.clone()
                ),
                amount: Uint128::from(500_000u128),
            }),
            swap: None,
            seller_amount: Uint128::from(10_000_000u128),
        })
    );

    let token_id = test_pair.token_ids[0].clone();

    // Can swap approved NFT
    let response = router.execute_contract(
        bidder.clone(),
        test_pair.address.clone(),
        &InfinityPairExecuteMsg::SwapTokensForSpecificNft {
            token_id: token_id.clone(),
            asset_recipient: None,
        },
        &[coin(10_600_000u128, NATIVE_DENOM)],
    );
    assert!(response.is_ok());

    assert_nft_owner(&router, &collection, token_id, &bidder);

    test_pair.pair = router
        .wrap()
        .query_wasm_smart::<Pair>(test_pair.address.clone(), &InfinityPairQueryMsg::Pair {})
        .unwrap();

    assert_eq!(test_pair.pair.internal.sell_to_pair_quote_summary, None);
    assert_eq!(
        test_pair.pair.internal.buy_from_pair_quote_summary,
        Some(QuoteSummary {
            fair_burn: TokenPayment {
                recipient: global_config.fair_burn,
                amount: Uint128::from(110_000u128),
            },
            royalty: Some(TokenPayment {
                recipient: Addr::unchecked(collection_info.royalty_info.unwrap().payment_address),
                amount: Uint128::from(550_000u128),
            }),
            swap: None,
            seller_amount: Uint128::from(11_000_000u128),
        })
    );
}

#[test]
fn try_nft_pair_exponential_user_submits_tokens_swap() {
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
                        bidder,
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

    let collection_info = router
        .wrap()
        .query_wasm_smart::<CollectionInfoResponse>(
            collection.clone(),
            &Sg721QueryMsg::CollectionInfo {},
        )
        .unwrap();

    let mut test_pair = create_pair_with_deposits(
        &mut router,
        &infinity_global,
        &infinity_factory,
        &minter,
        &collection,
        &creator,
        &owner,
        PairConfig {
            pair_type: PairType::Nft,
            bonding_curve: BondingCurve::Exponential {
                spot_price: Uint128::from(10_000_000u128),
                delta: Decimal::percent(12),
            },
            is_active: true,
            asset_recipient: None,
        },
        10u64,
        Uint128::zero(),
    );

    assert_eq!(test_pair.pair.internal.sell_to_pair_quote_summary, None);
    assert_eq!(
        test_pair.pair.internal.buy_from_pair_quote_summary,
        Some(QuoteSummary {
            fair_burn: TokenPayment {
                recipient: global_config.fair_burn.clone(),
                amount: Uint128::from(100_000u128),
            },
            royalty: Some(TokenPayment {
                recipient: Addr::unchecked(
                    collection_info.royalty_info.as_ref().unwrap().payment_address.clone()
                ),
                amount: Uint128::from(500_000u128),
            }),
            swap: None,
            seller_amount: Uint128::from(10_000_000u128),
        })
    );

    let token_id = test_pair.token_ids[0].clone();

    // Can swap approved NFT
    let response = router.execute_contract(
        bidder.clone(),
        test_pair.address.clone(),
        &InfinityPairExecuteMsg::SwapTokensForSpecificNft {
            token_id: token_id.clone(),
            asset_recipient: None,
        },
        &[coin(10_600_000u128, NATIVE_DENOM)],
    );
    assert!(response.is_ok());

    assert_nft_owner(&router, &collection, token_id, &bidder);

    test_pair.pair = router
        .wrap()
        .query_wasm_smart::<Pair>(test_pair.address.clone(), &InfinityPairQueryMsg::Pair {})
        .unwrap();

    assert_eq!(test_pair.pair.internal.sell_to_pair_quote_summary, None);
    assert_eq!(
        test_pair.pair.internal.buy_from_pair_quote_summary,
        Some(QuoteSummary {
            fair_burn: TokenPayment {
                recipient: global_config.fair_burn,
                amount: Uint128::from(112_000u128),
            },
            royalty: Some(TokenPayment {
                recipient: Addr::unchecked(collection_info.royalty_info.unwrap().payment_address),
                amount: Uint128::from(560_000u128),
            }),
            swap: None,
            seller_amount: Uint128::from(11_200_000u128),
        })
    );
}
