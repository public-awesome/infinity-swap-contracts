use crate::helpers::nft_functions::{approve, assert_nft_owner, mint_to};
use crate::helpers::pair_functions::create_pair_with_deposits;
use crate::setup::setup_accounts::MarketAccounts;
use crate::setup::templates::{setup_infinity_test, standard_minter_template, InfinityTestSetup};

use cosmwasm_std::{Addr, Uint128};
use cw_multi_test::Executor;
use infinity_global::{msg::QueryMsg as InfinityGlobalQueryMsg, GlobalConfig};
use infinity_pair::state::{BondingCurve, PairConfig, PairType};
use infinity_router::msg::{
    ExecuteMsg as InfinityRouterExecuteMsg, QueryMsg as InfinityRouterQueryMsg, SellOrder,
};
use infinity_router::nfts_for_tokens_iterators::types::NftForTokensQuote;
use sg721_base::msg::{CollectionInfoResponse, QueryMsg as Sg721QueryMsg};
use sg_std::NATIVE_DENOM;
use test_suite::common_setup::msg::MinterTemplateResponse;

#[test]
fn try_router_nfts_for_tokens_swap_simple() {
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

    let mut pairs = vec![];
    for _ in 0..3 {
        pairs.push(create_pair_with_deposits(
            &mut router,
            &infinity_global,
            &infinity_factory,
            &minter,
            &collection,
            &creator,
            &owner,
            PairConfig {
                pair_type: PairType::Token,
                bonding_curve: BondingCurve::Linear {
                    spot_price: Uint128::from(100_000_000u128),
                    delta: Uint128::from(1_000_000u128),
                },
                is_active: true,
                asset_recipient: None,
            },
            0u64,
            Uint128::from(10_000_000_000u128),
        ));
    }

    let quotes = router
        .wrap()
        .query_wasm_smart::<Vec<NftForTokensQuote>>(
            &global_config.infinity_router.clone(),
            &InfinityRouterQueryMsg::NftsForTokens {
                collection: collection.to_string(),
                denom: NATIVE_DENOM.to_string(),
                limit: 10,
                filter_sources: None,
            },
        )
        .unwrap();

    assert_eq!(quotes.len(), 10);

    let num_nfts = 2;
    let mut token_ids: Vec<String> = vec![];
    for _ in 0..num_nfts {
        let token_id = mint_to(&mut router, &creator.clone(), &bidder.clone(), &minter);
        approve(
            &mut router,
            &bidder,
            &collection,
            &global_config.infinity_router,
            token_id.clone(),
        );
        token_ids.push(token_id)
    }

    let response = router.execute_contract(
        bidder.clone(),
        global_config.infinity_router.clone(),
        &InfinityRouterExecuteMsg::SwapNftsForTokens {
            collection: collection.to_string(),
            denom: NATIVE_DENOM.to_string(),
            sell_orders: token_ids
                .iter()
                .enumerate()
                .map(|(idx, token_id)| SellOrder {
                    input_token_id: token_id.clone(),
                    min_output: quotes[idx].amount,
                })
                .collect(),
            swap_params: None,
            filter_sources: None,
        },
        &[],
    );
    assert!(response.is_ok());

    assert_nft_owner(&router, &collection, token_ids[0].clone(), &owner);
    assert_nft_owner(&router, &collection, token_ids[1].clone(), &owner);
}
