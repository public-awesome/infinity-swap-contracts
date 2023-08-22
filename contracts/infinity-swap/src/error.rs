use cosmwasm_std::{Coin, StdError, Uint128};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("InsufficientFunds: expected {expected}")]
    InsufficientFunds { expected: Coin },

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Invalid pair: {0}")]
    InvalidPair(String),
    // #[error("Pair not found: {0}")]
    // PairNotFound(String),

    // #[error("NFT not found: token_id {0}")]
    // NftNotFound(String),

    // #[error("No quote for pair: {0}")]
    // NoQuoteForPair(String),

    // #[error("UnpaidListingFee: {0}")]
    // UnpaidListingFee(Uint128),

    // #[error("Insufficient funds: {0}")]
    // InsufficientFunds(String),

    // #[error("Unable to remove pair: {0}")]
    // UnableToRemovePair(String),

    // #[error("Invalid swap params: {0}")]
    // InvalidSwapParams(String),

    // #[error("Internal error: {0}")]
    // InternalError(String),

    // #[error("Deadline passed")]
    // DeadlinePassed,

    // #[error("Swap error: {0}")]
    // SwapError(String),

    // #[error("Invalid property key error: {0}")]
    // InvalidPropertyKeyError(String),
}
