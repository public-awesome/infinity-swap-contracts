use crate::helpers::pool_functions::create_pool;
use crate::helpers::utils::{assert_error, get_native_balance};
use crate::setup::setup_infinity_contracts::{setup_infinity_test, InfinityTestSetup};

use cosmwasm_std::{Addr, Uint128};
use cw_multi_test::Executor;
use infinity_pool::msg::{ExecuteMsg as InfinityPoolExecuteMsg, PoolInfo};
use infinity_pool::state::{BondingCurve, PoolType};
use infinity_pool::ContractError;
use test_suite::common_setup::msg::MinterTemplateResponse;

#[test]
fn try_withdraw_tokens_token_pool() {
    let InfinityTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts,
            },
        marketplace,
        infinity_index,
        infinity_pool_code_id,
        ..
    } = setup_infinity_test(100).unwrap();

    let collection_resp = &collection_response_vec[0];
    let collection = collection_resp.collection.clone().unwrap();

    let deposit_amount = Uint128::from(10_000_000u128);

    let infinity_pool = create_pool(
        &mut router,
        infinity_pool_code_id,
        &accts.owner,
        marketplace.to_string(),
        infinity_index.to_string(),
        PoolInfo {
            collection: collection.to_string(),
            owner: accts.owner.to_string(),
            asset_recipient: None,
            pool_type: PoolType::Token,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(10_000u128),
            delta: Uint128::from(10u128),
            finders_fee_bps: 0u64,
            swap_fee_bps: 0u64,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        deposit_amount,
    );

    let withdraw_amount = Uint128::from(1_000_000u128);

    // Only owner can withdraw tokens
    let response = router.execute_contract(
        accts.bidder.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::WithdrawTokens {
            amount: withdraw_amount,
            asset_recipient: None,
        },
        &[],
    );
    assert_error(
        response,
        ContractError::Unauthorized("sender is not the owner of the pool".to_string()),
    );

    // Owner cannot withdraw more tokens than are in the pool
    let response = router.execute_contract(
        accts.owner.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::WithdrawTokens {
            amount: deposit_amount + Uint128::from(1u128),
            asset_recipient: None,
        },
        &[],
    );
    assert_error(response, ContractError::InvalidInput("amount exceeds total tokens".to_string()));

    // Owner can withdraw tokens
    let balance_before = get_native_balance(&router, accts.owner.clone());
    let response = router.execute_contract(
        accts.owner.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::WithdrawTokens {
            amount: withdraw_amount,
            asset_recipient: None,
        },
        &[],
    );
    assert!(response.is_ok());
    let balance_after = get_native_balance(&router, accts.owner.clone());

    // Check that the owner's account was credited
    assert_eq!(balance_before, balance_after - withdraw_amount);

    // Check that the contract was debited
    let contract_balance = get_native_balance(&router, infinity_pool.clone());
    assert_eq!(contract_balance, deposit_amount - withdraw_amount);

    // Owner can withdraw all tokens
    let recipient_address = Addr::unchecked("recipient");
    let response = router.execute_contract(
        accts.owner.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::WithdrawAllTokens {
            asset_recipient: Some(recipient_address.to_string()),
        },
        &[],
    );
    assert!(response.is_ok());

    // Check that asset_recipient address received the tokens
    let balance_after = get_native_balance(&router, recipient_address.clone());
    assert_eq!(balance_after, deposit_amount - withdraw_amount);

    // Check that the contract was depited
    let contract_balance = get_native_balance(&router, infinity_pool.clone());
    assert_eq!(contract_balance, Uint128::zero());
}

#[test]
fn try_withdraw_tokens_nft_pool() {
    let InfinityTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts,
            },
        marketplace,
        infinity_index,
        infinity_pool_code_id,
        ..
    } = setup_infinity_test(100).unwrap();

    let collection_resp = &collection_response_vec[0];
    let collection = collection_resp.collection.clone().unwrap();

    let deposit_amount = Uint128::from(10_000_000u128);

    let infinity_pool = create_pool(
        &mut router,
        infinity_pool_code_id,
        &accts.owner,
        marketplace.to_string(),
        infinity_index.to_string(),
        PoolInfo {
            collection: collection.to_string(),
            owner: accts.owner.to_string(),
            asset_recipient: None,
            pool_type: PoolType::Nft,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(10_000u128),
            delta: Uint128::from(10u128),
            finders_fee_bps: 0u64,
            swap_fee_bps: 0u64,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        deposit_amount,
    );

    let withdraw_amount = Uint128::from(1_000_000u128);

    // Only owner can withdraw tokens
    let response = router.execute_contract(
        accts.bidder.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::WithdrawTokens {
            amount: withdraw_amount,
            asset_recipient: None,
        },
        &[],
    );
    assert_error(
        response,
        ContractError::Unauthorized("sender is not the owner of the pool".to_string()),
    );

    // Owner cannot withdraw more tokens than are in the pool
    let response = router.execute_contract(
        accts.owner.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::WithdrawTokens {
            amount: deposit_amount + Uint128::from(1u128),
            asset_recipient: None,
        },
        &[],
    );
    assert_error(response, ContractError::InvalidInput("amount exceeds total tokens".to_string()));

    // Owner can withdraw tokens
    let balance_before = get_native_balance(&router, accts.owner.clone());
    let response = router.execute_contract(
        accts.owner.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::WithdrawTokens {
            amount: withdraw_amount,
            asset_recipient: None,
        },
        &[],
    );
    assert!(response.is_ok());
    let balance_after = get_native_balance(&router, accts.owner.clone());

    // Check that the owner's account was credited
    assert_eq!(balance_before, balance_after - withdraw_amount);

    // Check that the contract was debited
    let contract_balance = get_native_balance(&router, infinity_pool.clone());
    assert_eq!(contract_balance, deposit_amount - withdraw_amount);

    // Owner can withdraw all tokens
    let recipient_address = Addr::unchecked("recipient");
    let response = router.execute_contract(
        accts.owner.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::WithdrawAllTokens {
            asset_recipient: Some(recipient_address.to_string()),
        },
        &[],
    );
    assert!(response.is_ok());

    // Check that asset_recipient address received the tokens
    let balance_after = get_native_balance(&router, recipient_address.clone());
    assert_eq!(balance_after, deposit_amount - withdraw_amount);

    // Check that the contract was depited
    let contract_balance = get_native_balance(&router, infinity_pool.clone());
    assert_eq!(contract_balance, Uint128::zero());
}

#[test]
fn try_withdraw_tokens_trade_pool() {
    let InfinityTestSetup {
        vending_template:
            MinterTemplateResponse {
                collection_response_vec,
                mut router,
                accts,
            },
        marketplace,
        infinity_index,
        infinity_pool_code_id,
        ..
    } = setup_infinity_test(100).unwrap();

    let collection_resp = &collection_response_vec[0];
    let collection = collection_resp.collection.clone().unwrap();

    let deposit_amount = Uint128::from(10_000_000u128);

    let infinity_pool = create_pool(
        &mut router,
        infinity_pool_code_id,
        &accts.owner,
        marketplace.to_string(),
        infinity_index.to_string(),
        PoolInfo {
            collection: collection.to_string(),
            owner: accts.owner.to_string(),
            asset_recipient: None,
            pool_type: PoolType::Trade,
            bonding_curve: BondingCurve::Linear,
            spot_price: Uint128::from(10_000u128),
            delta: Uint128::from(10u128),
            finders_fee_bps: 0u64,
            swap_fee_bps: 0u64,
            reinvest_tokens: false,
            reinvest_nfts: false,
        },
        deposit_amount,
    );

    let withdraw_amount = Uint128::from(1_000_000u128);

    // Only owner can withdraw tokens
    let response = router.execute_contract(
        accts.bidder.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::WithdrawTokens {
            amount: withdraw_amount,
            asset_recipient: None,
        },
        &[],
    );
    assert_error(
        response,
        ContractError::Unauthorized("sender is not the owner of the pool".to_string()),
    );

    // Owner cannot withdraw more tokens than are in the pool
    let response = router.execute_contract(
        accts.owner.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::WithdrawTokens {
            amount: deposit_amount + Uint128::from(1u128),
            asset_recipient: None,
        },
        &[],
    );
    assert_error(response, ContractError::InvalidInput("amount exceeds total tokens".to_string()));

    // Owner can withdraw tokens
    let balance_before = get_native_balance(&router, accts.owner.clone());
    let response = router.execute_contract(
        accts.owner.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::WithdrawTokens {
            amount: withdraw_amount,
            asset_recipient: None,
        },
        &[],
    );
    assert!(response.is_ok());
    let balance_after = get_native_balance(&router, accts.owner.clone());

    // Check that the owner's account was credited
    assert_eq!(balance_before, balance_after - withdraw_amount);

    // Check that the contract was debited
    let contract_balance = get_native_balance(&router, infinity_pool.clone());
    assert_eq!(contract_balance, deposit_amount - withdraw_amount);

    // Owner can withdraw all tokens
    let recipient_address = Addr::unchecked("recipient");
    let response = router.execute_contract(
        accts.owner.clone(),
        infinity_pool.clone(),
        &InfinityPoolExecuteMsg::WithdrawAllTokens {
            asset_recipient: Some(recipient_address.to_string()),
        },
        &[],
    );
    assert!(response.is_ok());

    // Check that asset_recipient address received the tokens
    let balance_after = get_native_balance(&router, recipient_address.clone());
    assert_eq!(balance_after, deposit_amount - withdraw_amount);

    // Check that the contract was depited
    let contract_balance = get_native_balance(&router, infinity_pool.clone());
    assert_eq!(contract_balance, Uint128::zero());
}
