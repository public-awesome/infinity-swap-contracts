use super::setup_contracts::contract_infinity_pool;
use cosmwasm_std::Addr;
use cw_multi_test::Executor;
use sg_marketplace::ContractError;
use sg_multi_test::StargazeApp;
use sg_std::NATIVE_DENOM;

pub fn setup_infinity_pool(
    router: &mut StargazeApp,
    sender: Addr,
    marketplace_addr: Addr,
) -> Result<Addr, ContractError> {
    let infinity_pool_id = router.store_code(contract_infinity_pool());
    let msg = crate::msg::InstantiateMsg {
        denom: NATIVE_DENOM.to_string(),
        marketplace_addr: marketplace_addr.to_string(),
        developer: None,
    };
    let infinity_pool = router
        .instantiate_contract(infinity_pool_id, sender, &msg, &[], "Infinity Pool", None)
        .unwrap();
    Ok(infinity_pool)
}
