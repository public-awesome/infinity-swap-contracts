use crate::helpers::pair_functions::create_pair_with_deposits;
use crate::setup::setup_accounts::MarketAccounts;
use crate::setup::templates::{setup_infinity_test, standard_minter_template, InfinityTestSetup};

use cosmwasm_std::{coin, Addr, Uint128};
use cw_multi_test::Executor;
use infinity_global::{GlobalConfig, QueryMsg as InfinityGlobalQueryMsg};
use infinity_pair::state::{BondingCurve, PairConfig, PairType};
use infinity_router::msg::{
    ExecuteMsg as InfinityRouterExecuteMsg, QueryMsg as InfinityRouterQueryMsg,
};
use infinity_router::tokens_for_nfts_iterators::types::{
    TokensForNftQuote, TokensForNftSourceData,
};
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg_std::NATIVE_DENOM;
use test_suite::common_setup::msg::MinterTemplateResponse;

#[test]
fn try_router_tokens_for_nfts_swap_simple() {
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

    let _collection_info = router
        .wrap()
        .query_wasm_smart::<CollectionInfoResponse>(
            collection.clone(),
            &Sg721QueryMsg::CollectionInfo {},
        )
        .unwrap();

    let test_pair_0 = create_pair_with_deposits(
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
        100u64,
        Uint128::zero(),
    );

    let test_pair_1 = create_pair_with_deposits(
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
                spot_price: Uint128::from(10_100_000u128),
                delta: Uint128::from(1_000_000u128),
            },
            is_active: true,
            asset_recipient: None,
        },
        100u64,
        Uint128::zero(),
    );

    let quotes = router
        .wrap()
        .query_wasm_smart::<Vec<TokensForNftQuote>>(
            &global_config.infinity_router.clone(),
            &InfinityRouterQueryMsg::TokensForNfts {
                collection: collection.to_string(),
                denom: NATIVE_DENOM.to_string(),
                limit: 2,
                filter_sources: None,
            },
        )
        .unwrap();

    assert_eq!(quotes[0].source_data, TokensForNftSourceData::Infinity(test_pair_0.pair));
    assert_eq!(quotes[1].source_data, TokensForNftSourceData::Infinity(test_pair_1.pair));

    let max_inputs = quotes.iter().map(|q| q.amount).collect::<Vec<Uint128>>();
    let total_tokens = max_inputs.iter().sum::<Uint128>();
    let response = router.execute_contract(
        bidder.clone(),
        global_config.infinity_router.clone(),
        &InfinityRouterExecuteMsg::SwapTokensForNfts {
            collection: collection.to_string(),
            denom: NATIVE_DENOM.to_string(),
            max_inputs,
            swap_params: None,
            filter_sources: None,
        },
        &[coin(total_tokens.u128(), NATIVE_DENOM)],
    );
    assert!(response.is_ok());
}
