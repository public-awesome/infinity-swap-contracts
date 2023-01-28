use std::vec;

use crate::instantiate::instantiate;
use crate::msg::InstantiateMsg;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{coins, Addr, Attribute};
use sg_std::NATIVE_DENOM;

const MARKETPLACE: &str = "marketplace";

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
