use cosmwasm_std::Uint128;
use infinity_pool::msg::ExecuteMsg;
use infinity_pool::state::BondingCurve;

pub fn get_pool_fixtures(
    collection: &str,
    asset_account: &Option<String>,
    finders_fee_bps: u64,
    swap_fee_bps: u64,
) -> Vec<ExecuteMsg> {
    vec![
        ExecuteMsg::CreateTokenPool {
            collection: collection.to_string(),
            asset_recipient: asset_account.clone(),
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(100u64),
            delta: Uint128::from(10u64),
            finders_fee_bps,
        },
        ExecuteMsg::CreateTokenPool {
            collection: collection.to_string(),
            asset_recipient: asset_account.clone(),
            bonding_curve: BondingCurve::Exponential,
            spot_price: Uint128::from(200u64),
            delta: Uint128::from(200u64),
            finders_fee_bps,
        },
        ExecuteMsg::CreateNftPool {
            collection: collection.to_string(),
            asset_recipient: asset_account.clone(),
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(300u64),
            delta: Uint128::from(30u64),
            finders_fee_bps,
        },
        ExecuteMsg::CreateNftPool {
            collection: collection.to_string(),
            asset_recipient: asset_account.clone(),
            bonding_curve: BondingCurve::Exponential,
            spot_price: Uint128::from(400u64),
            delta: Uint128::from(400u64),
            finders_fee_bps,
        },
        ExecuteMsg::CreateTradePool {
            collection: collection.to_string(),
            asset_recipient: asset_account.clone(),
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(500u64),
            delta: Uint128::from(50u64),
            finders_fee_bps,
            swap_fee_bps,
            reinvest_nfts: true,
            reinvest_tokens: true,
        },
        ExecuteMsg::CreateTradePool {
            collection: collection.to_string(),
            asset_recipient: asset_account.clone(),
            bonding_curve: BondingCurve::Exponential,
            spot_price: Uint128::from(600u64),
            delta: Uint128::from(600u64),
            finders_fee_bps,
            swap_fee_bps,
            reinvest_nfts: true,
            reinvest_tokens: true,
        },
        ExecuteMsg::CreateTradePool {
            collection: collection.to_string(),
            asset_recipient: asset_account.clone(),
            bonding_curve: BondingCurve::ConstantProduct,
            spot_price: Uint128::from(0u64),
            delta: Uint128::from(0u64),
            finders_fee_bps,
            swap_fee_bps,
            reinvest_nfts: true,
            reinvest_tokens: true,
        },
        ExecuteMsg::CreateTokenPool {
            collection: collection.to_string(),
            asset_recipient: asset_account.clone(),
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(800u64),
            delta: Uint128::from(80u64),
            finders_fee_bps,
        },
        ExecuteMsg::CreateTokenPool {
            collection: collection.to_string(),
            asset_recipient: asset_account.clone(),
            bonding_curve: BondingCurve::Exponential,
            spot_price: Uint128::from(900u64),
            delta: Uint128::from(900u64),
            finders_fee_bps,
        },
        ExecuteMsg::CreateNftPool {
            collection: collection.to_string(),
            asset_recipient: asset_account.clone(),
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(1000u64),
            delta: Uint128::from(100u64),
            finders_fee_bps,
        },
        ExecuteMsg::CreateNftPool {
            collection: collection.to_string(),
            asset_recipient: asset_account.clone(),
            bonding_curve: BondingCurve::Exponential,
            spot_price: Uint128::from(1100u64),
            delta: Uint128::from(1100u64),
            finders_fee_bps,
        },
        ExecuteMsg::CreateTradePool {
            collection: collection.to_string(),
            asset_recipient: asset_account.clone(),
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(1200u64),
            delta: Uint128::from(120u64),
            finders_fee_bps,
            swap_fee_bps,
            reinvest_nfts: false,
            reinvest_tokens: false,
        },
        ExecuteMsg::CreateTradePool {
            collection: collection.to_string(),
            asset_recipient: asset_account.clone(),
            bonding_curve: BondingCurve::Exponential,
            spot_price: Uint128::from(1300u64),
            delta: Uint128::from(1300u64),
            finders_fee_bps,
            swap_fee_bps,
            reinvest_nfts: false,
            reinvest_tokens: false,
        },
        ExecuteMsg::CreateTradePool {
            collection: collection.to_string(),
            asset_recipient: asset_account.clone(),
            bonding_curve: BondingCurve::ConstantProduct,
            spot_price: Uint128::from(0u64),
            delta: Uint128::from(0u64),
            finders_fee_bps,
            swap_fee_bps,
            reinvest_nfts: false,
            reinvest_tokens: false,
        },
    ]
}
