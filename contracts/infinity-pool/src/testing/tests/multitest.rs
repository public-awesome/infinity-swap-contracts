use std::fmt::Error;
use std::vec;

use crate::error::ContractError;
use crate::execute::execute;
use crate::instantiate::instantiate;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{BondingCurve, PoolType};
use crate::testing::helpers::nft_functions::{approve, mint, mint_for};
use crate::testing::helpers::pool_functions::create_pool;
use crate::testing::helpers::utils::assert_error;
use crate::testing::setup::setup_accounts::{setup_accounts, setup_second_bidder_account};
use crate::testing::setup::setup_infinity_pool::setup_infinity_pool;
use crate::testing::setup::setup_marketplace::setup_marketplace;
use crate::testing::setup::templates::standard_minter_template;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, Addr, Attribute, Uint128};
use cw_multi_test::Executor;
use sg_std::{GENESIS_MINT_START_TIME, NATIVE_DENOM};
use test_suite::common_setup::contract_boxes::custom_mock_app;
use test_suite::common_setup::setup_accounts_and_block::setup_block_time;

const CREATOR: &str = "creator";
const ASSET_ACCOUNT: &str = "asset";
const MARKETPLACE: &str = "marketplace";
const COLLECTION: &str = "collection";
const TOKEN_ID: u32 = 123;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();

    let marketplace_addr = Addr::unchecked(MARKETPLACE);

    let msg = InstantiateMsg {
        denom: NATIVE_DENOM.to_string(),
        marketplace_addr: marketplace_addr.to_string(),
    };
    let info = mock_info("creator", &coins(1000, NATIVE_DENOM));

    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    let expected = vec![
        Attribute {
            key: "denom".to_string(),
            value: NATIVE_DENOM.to_string(),
        },
        Attribute {
            key: "marketplace_addr".to_string(),
            value: marketplace_addr.to_string(),
        },
    ];
    assert_eq!(res.attributes[3..5], expected);
}
