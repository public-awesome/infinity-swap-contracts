use std::vec;

use crate::error::ContractError;
use crate::execute::{execute};
use crate::instantiate::{instantiate};
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::testing::setup::setup_accounts::setup_accounts;
use crate::testing::setup::setup_infinity_pool::{setup_infinity_pool};
use crate::testing::setup::templates::{standard_minter_template};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, Addr, Attribute};
use sg_std::NATIVE_DENOM;
use test_suite::common_setup::contract_boxes::custom_mock_app;

const CREATOR: &str = "creator";
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

// #[test]
// fn create_token_pool() {
//     let mut app = custom_mock_app();
//     let (owner, bidder, creator) = setup_accounts(&mut app).unwrap();
//     let vt = standard_minter_template(1);
//     let collection = vt.collection_response_vec[0].collection.clone().unwrap();

//     let msg = ExecuteMsg::CreatePool {
//         collection: collection.to_string(),
//         pool_type: PoolType::Token,
//         bonding_curve: BondingCurve::Linear,
//         delta: 1,
//         fee: 1,
//         asset_recipient: Addr::unchecked(CREATOR),
//     };

//     let (mut router, creator, bidder) = (vt.router, vt.accts.creator, vt.accts.bidder);
//     let marketplace = setup_marketplace(&mut router, creator.clone()).unwrap();
//     let minter = vt.collection_response_vec[0].minter.clone().unwrap();
//     let collection = vt.collection_response_vec[0].collection.clone().unwrap();
//     let token_id = 1;
//     let start_time = Timestamp::from_nanos(GENESIS_MINT_START_TIME);
//     setup_block_time(&mut router, GENESIS_MINT_START_TIME, None);

//     mint(&mut router, &creator, &minter);
//     approve(&mut router, &creator, &collection, &marketplace, token_id);

//     let mut deps = mock_dependencies();

//     let marketplace_addr = Addr::unchecked(MARKETPLACE);
//     let collection_addr = Addr::unchecked(CO);

//     let mut app = custom_mock_app();
//     let info = mock_info("creator", &coins(1000, NATIVE_DENOM));
//     let infinity_pool = setup_infinity_pool(
//         &mut app,
//         info.sender,
//         marketplace_addr,
//     );

//     let msg = ExecuteMsg::CreatePool { collection: (), pool_type: (), bonding_curve: (), delta: (), fee: (), asset_recipient: () };

//     let msg = InstantiateMsg {
//         denom: NATIVE_DENOM.to_string(),
//         marketplace_addr: marketplace_addr.to_string(),
//     };
//     let info = mock_info("creator", &coins(1000, NATIVE_DENOM));

//     let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
//     let expected = vec![
//         Attribute {
//             key: "action".to_string(),
//             value: "instantiate".to_string(),
//         },
//         Attribute {
//             key: "contract_name".to_string(),
//             value: "crates.io:infinity-pool".to_string(),
//         },
//         Attribute {
//             key: "contract_version".to_string(),
//             value: "0.1.0".to_string(),
//         },
//         Attribute {
//             key: "denom".to_string(),
//             value: NATIVE_DENOM.to_string(),
//         },
//         Attribute {
//             key: "marketplace_addr".to_string(),
//             value: marketplace_addr.to_string(),
//         },
//     ];
//     assert_eq!(res.attributes, expected);
// }

