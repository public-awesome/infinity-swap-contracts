use anyhow::Error;
use cosmwasm_std::{Addr, Coin, Uint128};
use cw_multi_test::AppResponse;
use sg_multi_test::StargazeApp;
use sg_std::NATIVE_DENOM;
use std::collections::HashMap;

pub fn assert_error(response: Result<AppResponse, Error>, expected: String) {
    assert_eq!(response.unwrap_err().source().unwrap().to_string(), expected);
}

pub fn _assert_event(response: Result<AppResponse, Error>, ty: &str) {
    assert!(response.unwrap().events.iter().any(|event| event.ty == ty));
}

pub fn _get_native_balances(router: &StargazeApp, addresses: &Vec<Addr>) -> HashMap<Addr, Coin> {
    let mut balances: HashMap<Addr, Coin> = HashMap::new();
    for address in addresses {
        let native_balance = router.wrap().query_balance(address, NATIVE_DENOM).unwrap();
        balances.insert(address.clone(), native_balance);
    }
    balances
}

pub fn _get_native_balance(router: &StargazeApp, address: Addr) -> Uint128 {
    _get_native_balances(router, &vec![address.clone()]).get(&address).unwrap().amount
}
