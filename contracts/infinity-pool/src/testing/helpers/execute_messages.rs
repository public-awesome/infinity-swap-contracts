use cosmwasm_std::{Addr, Timestamp, Uint128};
use sg_std::GENESIS_MINT_START_TIME;

use crate::msg::ExecuteMsg::DirectSwapNftsForTokens;
use crate::msg::{NftSwap, SwapParams};
use crate::state::Pool;

pub fn get_direct_swap_nfts_for_tokens_msg(
    pool: Pool,
    token_id_1: u32,
    token_amount: u128,
    robust: bool,
    finder: Option<String>,
) -> crate::msg::ExecuteMsg {
    DirectSwapNftsForTokens {
        pool_id: pool.id,
        nfts_to_swap: vec![NftSwap {
            nft_token_id: token_id_1.to_string(),
            token_amount: Uint128::new(token_amount),
        }],
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust,
            asset_recipient: None,
            finder,
        },
    }
}
