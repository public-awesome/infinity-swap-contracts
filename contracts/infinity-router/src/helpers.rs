use cosmwasm_std::{to_binary, Addr, SubMsg, WasmMsg};
use cw721::Cw721ExecuteMsg;
use sg_std::Response;

pub fn approve_nft(
    collection: &Addr,
    spender: &Addr,
    token_id: &String,
    response: Response,
) -> Response {
    response.add_submessage(SubMsg::new(WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_binary(&Cw721ExecuteMsg::Approve {
            spender: spender.to_string(),
            token_id: token_id.to_string(),
            expires: None,
        })
        .unwrap(),
        funds: vec![],
    }))
}
