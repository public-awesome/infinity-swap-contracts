use anyhow::Error;
use cosmwasm_std::{Addr, Coin};
use cw_multi_test::AppResponse;
use sg_multi_test::StargazeApp;
use sg_std::NATIVE_DENOM;
use std::collections::HashMap;

pub fn get_native_balances(router: &StargazeApp, addresses: &Vec<Addr>) -> HashMap<Addr, Coin> {
    let mut balances: HashMap<Addr, Coin> = HashMap::new();
    for address in addresses {
        let native_balance = router.wrap().query_balance(address, NATIVE_DENOM).unwrap();
        balances.insert(address.clone(), native_balance);
    }
    balances
}
