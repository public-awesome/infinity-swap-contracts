use infinity_global::QueryMsg as InfinityGlobalQueryMsg;

use cosmwasm_std::{Addr, Uint128};
use cw_multi_test::Executor;
use infinity_factory::ExecuteMsg as InfinityFactoryExecuteMsg;
use infinity_global::GlobalConfig;
use infinity_pair::{
    msg::QueryMsg as InfinityPairQueryMsg,
    pair::Pair,
    state::{BondingCurve, PairConfig, PairImmutable, PairType},
};
use sg_multi_test::StargazeApp;
use sg_std::NATIVE_DENOM;

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
        collection: collection.clone(),
        owner: owner.clone(),
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
