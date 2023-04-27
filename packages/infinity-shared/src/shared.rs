use cosmwasm_std::{Addr, QuerierWrapper, StdResult};
use sg_marketplace::{
    msg::{ParamsResponse, QueryMsg as MarketplaceQueryMsg},
    state::SudoParams,
};

/// Load the marketplace params
pub fn load_marketplace_params(
    querier: &QuerierWrapper,
    marketplace_addr: &Addr,
) -> StdResult<SudoParams> {
    let marketplace_params: ParamsResponse =
        querier.query_wasm_smart(marketplace_addr, &MarketplaceQueryMsg::Params {})?;
    Ok(marketplace_params.params)
}
