use std::vec;

use crate::msg::PoolNftSwap;
use crate::msg::QueryMsg::SimDirectSwapNftsForTokens;
use crate::msg::QueryMsg::SimDirectSwapTokensForSpecificNfts;
use crate::msg::QueryMsg::SimSwapNftsForTokens;
use crate::msg::QueryMsg::SimSwapTokensForAnyNfts;
use crate::msg::QueryMsg::SimSwapTokensForSpecificNfts;
use crate::msg::{self};
use crate::msg::{NftSwap, SwapParams};
use crate::state::Pool;
use cosmwasm_std::Timestamp;
use cosmwasm_std::{Addr, Uint128};
use sg_std::GENESIS_MINT_START_TIME;

pub fn get_sim_swap_message(
    pool: Pool,
    token_id_1: u32,
    token_amount: u128,
    robust: bool,
    user2: Addr,
    finder: Option<String>,
) -> msg::QueryMsg {
    SimDirectSwapNftsForTokens {
        pool_id: pool.id,
        nfts_to_swap: vec![NftSwap {
            nft_token_id: token_id_1.to_string(),
            token_amount: Uint128::new(token_amount),
        }],
        sender: user2.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust,
            asset_recipient: None,
            finder,
        },
    }
}

pub fn get_sim_swap_nfts_for_tokens_msg(
    collection: &Addr,
    token_id_1: u32,
    token_amount: u128,
    robust: bool,
    user2: Addr,
    finder: Option<String>,
) -> msg::QueryMsg {
    SimSwapNftsForTokens {
        collection: collection.to_string(),
        nfts_to_swap: vec![NftSwap {
            nft_token_id: token_id_1.to_string(),
            token_amount: Uint128::new(token_amount),
        }],
        sender: user2.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust,
            asset_recipient: None,
            finder,
        },
    }
}

pub fn get_sim_direct_swap_tokens_for_specific_nfts_msg(
    pool: Pool,
    token_id_1: u32,
    token_amount: u128,
    robust: bool,
    user2: Addr,
    finder: Option<String>,
) -> msg::QueryMsg {
    SimDirectSwapTokensForSpecificNfts {
        pool_id: pool.id,
        nfts_to_swap_for: vec![NftSwap {
            nft_token_id: token_id_1.to_string(),
            token_amount: Uint128::new(token_amount),
        }],
        sender: user2.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust,
            asset_recipient: None,
            finder,
        },
    }
}

pub fn get_swap_tokens_for_specific_nfts_msg(
    pool: Pool,
    collection: Addr,
    token_id_1: u32,
    token_amount: u128,
    robust: bool,
    user2: Addr,
    finder: Option<String>,
) -> msg::QueryMsg {
    SimSwapTokensForSpecificNfts {
        pool_nfts_to_swap_for: vec![PoolNftSwap {
            pool_id: pool.id,
            nft_swaps: vec![NftSwap {
                nft_token_id: token_id_1.to_string(),
                token_amount: Uint128::new(token_amount),
            }],
        }],
        sender: user2.to_string(),
        collection: collection.to_string(),
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust,
            asset_recipient: None,
            finder,
        },
    }
}

pub fn get_swap_tokens_for_any_nfts_msg(
    collection: Addr,
    max_expected_token_input: Vec<Uint128>,
    robust: bool,
    user2: Addr,
    finder: Option<String>,
) -> msg::QueryMsg {
    SimSwapTokensForAnyNfts {
        sender: user2.to_string(),
        collection: collection.to_string(),
        max_expected_token_input,
        swap_params: SwapParams {
            deadline: Timestamp::from_nanos(GENESIS_MINT_START_TIME).plus_seconds(1000),
            robust,
            asset_recipient: None,
            finder,
        },
    }
}
