use std::vec;

use crate::error::ContractError;
use crate::execute::{execute};
use crate::instantiate::{instantiate};
use crate::msg::{ExecuteMsg, InstantiateMsg};
// use crate::query::{query_ask_count, query_asks_by_seller, query_bids_by_bidder};
// use crate::state::{ask_key, asks, bid_key, bids, Ask, Bid, SaleType};
// use crate::testing::setup::setup_marketplace::{
//     BID_REMOVAL_REWARD_BPS, MAX_EXPIRY, MAX_FINDERS_FEE_BPS, MIN_EXPIRY, TRADING_FEE_BPS,
// };
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coin, coins, Addr, DepsMut, Timestamp, Uint128, Attribute};
// use cw_utils::Duration;
use sg_std::NATIVE_DENOM;

const CREATOR: &str = "creator";
const COLLECTION: &str = "collection";
const TOKEN_ID: u32 = 123;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies();

    let marketplace_addr = Addr::unchecked("marketplace");

    let msg = InstantiateMsg {
        denom: NATIVE_DENOM.to_string(),
        marketplace_addr: marketplace_addr.to_string(),
    };
    let info = mock_info("creator", &coins(1000, NATIVE_DENOM));

    // we can just call .unwrap() to assert this was a success
    let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let expected = vec![
        Attribute {
            key: "action".to_string(),
            value: "instantiate".to_string(),
        },
        Attribute {
            key: "contract_name".to_string(),
            value: "crates.io:infinity-pool".to_string(),
        },
        Attribute {
            key: "contract_version".to_string(),
            value: "0.1.0".to_string(),
        },
        Attribute {
            key: "denom".to_string(),
            value: NATIVE_DENOM.to_string(),
        },
        Attribute {
            key: "marketplace_addr".to_string(),
            value: marketplace_addr.to_string(),
        },
    ];
    assert_eq!(res.attributes, expected);
}

