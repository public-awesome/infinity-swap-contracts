use cosmwasm_std::Uint128;
use infinity_pool::msg::PoolInfo;
use infinity_pool::state::{BondingCurve, PoolType};

pub fn get_pool_fixtures(
    owner: &str,
    collection: &str,
    asset_recipient: &Option<String>,
    finders_fee_bps: u64,
    swap_fee_bps: u64,
) -> Vec<PoolInfo> {
    vec![
        PoolInfo {
            collection: collection.to_string(),
            owner: owner.to_string(),
            asset_recipient: asset_recipient.clone(),
            pool_type: PoolType::Token,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(100u64),
            delta: Uint128::from(10u64),
            finders_fee_bps,
            swap_fee_bps,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        PoolInfo {
            collection: collection.to_string(),
            owner: owner.to_string(),
            asset_recipient: asset_recipient.clone(),
            pool_type: PoolType::Token,
            bonding_curve: BondingCurve::Exponential,
            spot_price: Uint128::from(200u64),
            delta: Uint128::from(200u64),
            finders_fee_bps,
            swap_fee_bps,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        PoolInfo {
            collection: collection.to_string(),
            owner: owner.to_string(),
            asset_recipient: asset_recipient.clone(),
            pool_type: PoolType::Nft,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(300u64),
            delta: Uint128::from(30u64),
            finders_fee_bps,
            swap_fee_bps,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        PoolInfo {
            collection: collection.to_string(),
            owner: owner.to_string(),
            asset_recipient: asset_recipient.clone(),
            pool_type: PoolType::Nft,
            bonding_curve: BondingCurve::Exponential,
            spot_price: Uint128::from(400u64),
            delta: Uint128::from(400u64),
            finders_fee_bps,
            swap_fee_bps,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        PoolInfo {
            collection: collection.to_string(),
            owner: owner.to_string(),
            asset_recipient: asset_recipient.clone(),
            pool_type: PoolType::Trade,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(500u64),
            delta: Uint128::from(50u64),
            finders_fee_bps,
            swap_fee_bps,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        PoolInfo {
            collection: collection.to_string(),
            owner: owner.to_string(),
            asset_recipient: asset_recipient.clone(),
            pool_type: PoolType::Trade,
            bonding_curve: BondingCurve::Exponential,
            spot_price: Uint128::from(600u64),
            delta: Uint128::from(600u64),
            finders_fee_bps,
            swap_fee_bps,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        PoolInfo {
            collection: collection.to_string(),
            owner: owner.to_string(),
            asset_recipient: asset_recipient.clone(),
            pool_type: PoolType::Trade,
            bonding_curve: BondingCurve::ConstantProduct,
            spot_price: Uint128::from(0u64),
            delta: Uint128::from(0u64),
            finders_fee_bps,
            swap_fee_bps,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        PoolInfo {
            collection: collection.to_string(),
            owner: owner.to_string(),
            asset_recipient: asset_recipient.clone(),
            pool_type: PoolType::Token,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(800u64),
            delta: Uint128::from(80u64),
            finders_fee_bps,
            swap_fee_bps,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        PoolInfo {
            collection: collection.to_string(),
            owner: owner.to_string(),
            asset_recipient: asset_recipient.clone(),
            pool_type: PoolType::Token,
            bonding_curve: BondingCurve::Exponential,
            spot_price: Uint128::from(900u64),
            delta: Uint128::from(900u64),
            finders_fee_bps,
            swap_fee_bps,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        PoolInfo {
            collection: collection.to_string(),
            owner: owner.to_string(),
            asset_recipient: asset_recipient.clone(),
            pool_type: PoolType::Nft,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(1000u64),
            delta: Uint128::from(100u64),
            finders_fee_bps,
            swap_fee_bps,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        PoolInfo {
            collection: collection.to_string(),
            owner: owner.to_string(),
            asset_recipient: asset_recipient.clone(),
            pool_type: PoolType::Nft,
            bonding_curve: BondingCurve::Exponential,
            spot_price: Uint128::from(1100u64),
            delta: Uint128::from(1100u64),
            finders_fee_bps,
            swap_fee_bps,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        PoolInfo {
            collection: collection.to_string(),
            owner: owner.to_string(),
            asset_recipient: asset_recipient.clone(),
            pool_type: PoolType::Trade,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(1200u64),
            delta: Uint128::from(120u64),
            finders_fee_bps,
            swap_fee_bps,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        PoolInfo {
            collection: collection.to_string(),
            owner: owner.to_string(),
            asset_recipient: asset_recipient.clone(),
            pool_type: PoolType::Trade,
            bonding_curve: BondingCurve::Exponential,
            spot_price: Uint128::from(1300u64),
            delta: Uint128::from(1300u64),
            finders_fee_bps,
            swap_fee_bps,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        PoolInfo {
            collection: collection.to_string(),
            owner: owner.to_string(),
            asset_recipient: asset_recipient.clone(),
            pool_type: PoolType::Trade,
            bonding_curve: BondingCurve::ConstantProduct,
            spot_price: Uint128::from(0u64),
            delta: Uint128::from(0u64),
            finders_fee_bps,
            swap_fee_bps,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
    ]
}

// pub fn create_pool_fixtures(
//     router: &mut StargazeApp,
//     infinity_swap: Addr,
//     collection: Addr,
//     creator: Addr,
//     user: Addr,
//     asset_account: Addr,
//     finders_fee_bps: u64,
//     swap_fee_bps: u64,
// ) -> Vec<Pool> {
//     let msgs = get_pool_fixtures(
//         &collection,
//         &Some(asset_account.to_string()),
//         finders_fee_bps,
//         swap_fee_bps,
//     );
//     let mut pools = vec![];
//     for (usize, msg) in msgs.into_iter().enumerate() {
//         let sender = if usize < 7 {
//             creator.clone()
//         } else {
//             user.clone()
//         };
//         pools.push(create_pool(router, infinity_swap.clone(), sender.clone(), msg).unwrap());
//     }
//     pools
// }

// pub fn create_and_activate_pool_fixtures(
//     router: &mut StargazeApp,
//     infinity_swap: Addr,
//     minter: Addr,
//     collection: Addr,
//     creator: Addr,
//     asset_account: Addr,
//     finders_fee_bps: u64,
//     swap_fee_bps: u64,
// ) -> Vec<Pool> {
//     let msgs = get_pool_fixtures(
//         &collection,
//         &Some(asset_account.to_string()),
//         finders_fee_bps,
//         swap_fee_bps,
//     );
//     let mut pools = vec![];
//     for msg in msgs.into_iter() {
//         let pool = create_pool(router, infinity_swap.clone(), creator.clone(), msg).unwrap();
//         if pool.can_buy_nfts() {
//             let deposit_amount = if pool.bonding_curve == BondingCurve::ConstantProduct {
//                 Uint128::from(3_000u64)
//             } else {
//                 pool.spot_price * Uint128::from(10u64)
//             };
//             deposit_tokens(
//                 router,
//                 infinity_swap.clone(),
//                 pool.owner.clone(),
//                 pool.id,
//                 deposit_amount,
//             )
//             .unwrap();
//         }
//         if pool.can_sell_nfts() {
//             let token_id_1 = mint(router, &pool.owner, &minter);
//             approve(router, &pool.owner, &collection, &infinity_swap, token_id_1);
//             let token_id_2 = mint(router, &pool.owner, &minter);
//             approve(router, &pool.owner, &collection, &infinity_swap, token_id_2);
//             deposit_nfts(
//                 router,
//                 infinity_swap.clone(),
//                 pool.owner.clone(),
//                 pool.id,
//                 pool.collection.clone(),
//                 vec![token_id_1, token_id_2].iter().map(|t| t.to_string()).collect(),
//             )
//             .unwrap();
//         }
//         let pool = activate(router, &infinity_swap, &pool.owner, pool.id, true).unwrap();
//         pools.push(pool);
//     }
//     pools
// }
