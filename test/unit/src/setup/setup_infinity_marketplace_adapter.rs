use super::setup_contracts::contract_infinity_marketplace_adapter;
use cosmwasm_std::Addr;
use cw_multi_test::Executor;
use sg_marketplace::ContractError;
use sg_multi_test::StargazeApp;

pub fn setup_infinity_marketplace_adapter(
    router: &mut StargazeApp,
    sender: Addr,
    marketplace_addr: Addr,
) -> Result<Addr, ContractError> {
    let infinity_marketplace_adapter_id =
        router.store_code(contract_infinity_marketplace_adapter());
    let msg = infinity_marketplace_adapter::msg::InstantiateMsg {
        marketplace: marketplace_addr.to_string(),
    };
    let infinity_marketplace_adapter = router
        .instantiate_contract(
            infinity_marketplace_adapter_id,
            sender,
            &msg,
            &[],
            "Infinity Marketplace Adapter",
            None,
        )
        .unwrap();
    Ok(infinity_marketplace_adapter)
}
