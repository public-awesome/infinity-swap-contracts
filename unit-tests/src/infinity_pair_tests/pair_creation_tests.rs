use crate::helpers::pair_functions::create_pair;
use crate::helpers::utils::assert_error;
use crate::setup::templates::{setup_infinity_test, standard_minter_template, InfinityTestSetup};

use cosmwasm_std::{Addr, Uint128};
use cw_multi_test::Executor;
use infinity_factory::ExecuteMsg as InfinityFactoryExecuteMsg;
use infinity_global::{GlobalConfig, QueryMsg as InfinityGlobalQueryMsg};
use infinity_pair::msg::{ExecuteMsg as InfinityPairExecuteMsg, QueryMsg as InfinityPairQueryMsg};
use infinity_pair::pair::Pair;
use infinity_pair::state::{BondingCurve, PairConfig, PairImmutable, PairInternal, PairType};
use infinity_shared::InfinityError;
use sg_multi_test::mock_deps;
use sg_std::NATIVE_DENOM;
use test_suite::common_setup::msg::MinterTemplateResponse;

#[test]
fn try_create_pair() {
    let vt = standard_minter_template(1000u32);
    let InfinityTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts,
            },
        infinity_global,
        infinity_factory,
        ..
    } = setup_infinity_test(vt).unwrap();

    let collection_resp = &collection_response_vec[0];
    let _minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let global_config = router
        .wrap()
        .query_wasm_smart::<GlobalConfig<Addr>>(
            infinity_global,
            &InfinityGlobalQueryMsg::GlobalConfig {},
        )
        .unwrap();

    let pair_immutable = PairImmutable {
        collection: collection.to_string(),
        owner: accts.creator.to_string(),
        denom: NATIVE_DENOM.to_string(),
    };

    let pair_config = PairConfig {
        pair_type: PairType::Token,
        bonding_curve: BondingCurve::Linear {
            spot_price: Uint128::from(10_000_000u128),
            delta: Uint128::from(1_000_000u128),
        },
        is_active: false,
        asset_recipient: None,
    };

    // Fails without funds sent
    let response = router.execute_contract(
        accts.creator.clone(),
        infinity_factory.clone(),
        &InfinityFactoryExecuteMsg::CreatePair {
            pair_immutable: pair_immutable.clone(),
            pair_config: pair_config.clone(),
        },
        &[],
    );
    assert!(response.is_err());

    // Works with correct funds
    let response = router.execute_contract(
        accts.creator.clone(),
        infinity_factory.clone(),
        &InfinityFactoryExecuteMsg::CreatePair {
            pair_immutable: pair_immutable.clone(),
            pair_config: pair_config.clone(),
        },
        &[global_config.pair_creation_fee],
    );
    assert!(response.is_ok());

    let pair_addr = response.unwrap().events[1].attributes[0].value.clone();

    let pair =
        router.wrap().query_wasm_smart::<Pair>(pair_addr, &InfinityPairQueryMsg::Pair {}).unwrap();

    let deps = mock_deps();
    assert_eq!(pair.immutable, pair_immutable.str_to_addr(&deps.api).unwrap());
    assert_eq!(pair.config, pair_config.str_to_addr(&deps.api).unwrap());
    assert_eq!(
        pair.internal,
        PairInternal {
            total_nfts: 0u64,
            sell_to_pair_quote_summary: None,
            buy_from_pair_quote_summary: None,
        }
    );
}

#[test]
fn try_update_pair_config() {
    let vt = standard_minter_template(1000u32);
    let InfinityTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts,
            },
        infinity_global,
        infinity_factory,
        ..
    } = setup_infinity_test(vt).unwrap();

    let collection_resp = &collection_response_vec[0];
    let _minter = collection_resp.minter.clone().unwrap();
    let collection = collection_resp.collection.clone().unwrap();

    let (pair_addr, _pair) = create_pair(
        &mut router,
        &infinity_global,
        &infinity_factory,
        &collection,
        &accts.owner.clone(),
    );

    // Non owner cannot withdraw tokens
    let response = router.execute_contract(
        accts.creator.clone(),
        pair_addr.clone(),
        &InfinityPairExecuteMsg::UpdatePairConfig {
            is_active: None,
            pair_type: None,
            bonding_curve: None,
            asset_recipient: None,
        },
        &[],
    );
    assert_error(
        response,
        InfinityError::Unauthorized("sender is not the owner of the pair".to_string()).to_string(),
    );

    // Owner can update config with no args
    let response = router.execute_contract(
        accts.owner.clone(),
        pair_addr.clone(),
        &InfinityPairExecuteMsg::UpdatePairConfig {
            is_active: None,
            pair_type: None,
            bonding_curve: None,
            asset_recipient: None,
        },
        &[],
    );
    assert!(response.is_ok());

    // Owner can update config with args
    let is_active = true;
    let pair_type = PairType::Nft;
    let bonding_curve = BondingCurve::ConstantProduct;
    let asset_recipient = Addr::unchecked("asset_recipient");
    let response = router.execute_contract(
        accts.owner.clone(),
        pair_addr.clone(),
        &InfinityPairExecuteMsg::UpdatePairConfig {
            is_active: Some(is_active),
            pair_type: Some(pair_type.clone()),
            bonding_curve: Some(bonding_curve.clone()),
            asset_recipient: Some(asset_recipient.to_string()),
        },
        &[],
    );
    assert!(response.is_ok());

    let pair = router
        .wrap()
        .query_wasm_smart::<Pair>(pair_addr.clone(), &InfinityPairQueryMsg::Pair {})
        .unwrap();
    assert_eq!(pair.config.is_active, is_active);
    assert_eq!(pair.config.pair_type, pair_type);
    assert_eq!(pair.config.bonding_curve, bonding_curve);
    assert_eq!(pair.config.asset_recipient, Some(asset_recipient));
}
