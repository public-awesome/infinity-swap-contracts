use anyhow::Error;
use cosmwasm_std::{Addr, Coin};
use cw_multi_test::AppResponse;
use infinity_swap::ContractError;
use sg_multi_test::StargazeApp;
use sg_std::NATIVE_DENOM;
use std::collections::HashMap;

pub fn assert_error(res: Result<AppResponse, Error>, expected: ContractError) {
    assert_eq!(
        res.unwrap_err().source().unwrap().to_string(),
        expected.to_string()
    );
}

pub fn get_native_balances(router: &StargazeApp, addresses: &Vec<Addr>) -> HashMap<Addr, Coin> {
    let mut balances: HashMap<Addr, Coin> = HashMap::new();
    for address in addresses {
        let native_balance = router.wrap().query_balance(address, NATIVE_DENOM).unwrap();
        balances.insert(address.clone(), native_balance);
    }
    balances
}
