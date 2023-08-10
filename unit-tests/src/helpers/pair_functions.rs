use cw721::Cw721ExecuteMsg;
use infinity_global::QueryMsg as InfinityGlobalQueryMsg;

use cosmwasm_std::{coin, to_binary, Addr, Empty, Uint128};
use cw_multi_test::Executor;
use infinity_factory::ExecuteMsg as InfinityFactoryExecuteMsg;
use infinity_global::GlobalConfig;
use infinity_pair::{
    msg::{ExecuteMsg as InfinityPairExecuteMsg, QueryMsg as InfinityPairQueryMsg},
    pair::Pair,
    state::{BondingCurve, PairConfig, PairImmutable, PairType},
};
use sg_multi_test::StargazeApp;
use sg_std::NATIVE_DENOM;

use crate::helpers::nft_functions::mint_to;

pub fn create_pair(
    router: &mut StargazeApp,
    infinity_global: &Addr,
    infinity_factory: &Addr,
    collection: &Addr,
    owner: &Addr,
) -> (Addr, Pair) {
    let global_config = router
        .wrap()
        .query_wasm_smart::<GlobalConfig<Addr>>(
            infinity_global,
            &InfinityGlobalQueryMsg::GlobalConfig {},
        )
        .unwrap();

    let pair_immutable = PairImmutable {
        collection: collection.to_string(),
        owner: owner.to_string(),
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

    let response = router.execute_contract(
        owner.clone(),
        infinity_factory.clone(),
        &InfinityFactoryExecuteMsg::CreatePair {
            pair_immutable: pair_immutable.clone(),
            pair_config: pair_config.clone(),
        },
        &[global_config.pair_creation_fee],
    );
    let pair_addr = response.unwrap().events[1].attributes[0].value.clone();

    let pair = router
        .wrap()
        .query_wasm_smart::<Pair>(pair_addr.clone(), &InfinityPairQueryMsg::Pair {})
        .unwrap();

    (Addr::unchecked(pair_addr), pair)
}

#[derive(Debug)]
pub struct TestPair {
    pub address: Addr,
    pub token_ids: Vec<String>,
    pub pair: Pair,
}

pub fn create_pair_with_deposits(
    router: &mut StargazeApp,
    infinity_global: &Addr,
    infinity_factory: &Addr,
    minter: &Addr,
    collection: &Addr,
    creator: &Addr,
    owner: &Addr,
    pair_config: PairConfig<String>,
    num_nfts: u64,
    num_tokens: Uint128,
) -> TestPair {
    let (pair_addr, _pair) =
        create_pair(router, infinity_global, infinity_factory, collection, owner);

    let response = router.execute_contract(
        owner.clone(),
        pair_addr.clone(),
        &InfinityPairExecuteMsg::UpdatePairConfig {
            is_active: Some(pair_config.is_active),
            pair_type: Some(pair_config.pair_type),
            bonding_curve: Some(pair_config.bonding_curve),
            asset_recipient: pair_config.asset_recipient.map(|a| a.to_string()),
        },
        &[],
    );
    assert!(response.is_ok());

    let mut token_ids: Vec<String> = vec![];
    for _ in 0..num_nfts {
        let token_id = mint_to(router, &creator.clone(), &owner.clone(), &minter);

        let response = router.execute_contract(
            owner.clone(),
            collection.clone(),
            &Cw721ExecuteMsg::SendNft {
                contract: pair_addr.to_string(),
                token_id: token_id.clone(),
                msg: to_binary(&Empty {}).unwrap(),
            },
            &[],
        );
        assert!(response.is_ok());

        token_ids.push(token_id)
    }

    if !num_tokens.is_zero() {
        let response = router.execute_contract(
            owner.clone(),
            pair_addr.clone(),
            &InfinityPairExecuteMsg::DepositTokens {},
            &[coin(num_tokens.u128(), NATIVE_DENOM)],
        );
        assert!(response.is_ok());
    }

    let pair = router
        .wrap()
        .query_wasm_smart::<Pair>(pair_addr.clone(), &InfinityPairQueryMsg::Pair {})
        .unwrap();

    TestPair {
        address: pair_addr,
        token_ids,
        pair,
    }
}
