use cosmwasm_std::{Addr, QuerierWrapper, StdResult};
use infinity_global::{
    msg::{GlobalConfigResponse, QueryMsg as InfinityGlobalQueryMsg},
    state::GlobalConfig,
};

/// Load the infinity global config
pub fn load_global_config(
    querier: &QuerierWrapper,
    infinity_global: &Addr,
) -> StdResult<GlobalConfig> {
    let global_config: GlobalConfigResponse =
        querier.query_wasm_smart(infinity_global, &InfinityGlobalQueryMsg::GlobalConfig {})?;
    Ok(global_config.config)
}
