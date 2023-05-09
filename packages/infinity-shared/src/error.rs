use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum InfinityError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("Not NFT owner: {0} is not the owner of the NFT")]
    NotNftOwner(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Insufficient funds: received {received}, expected {expected}")]
    InsufficientFunds { received: String, expected: String },

    #[error("Deadline passed")]
    DeadlinePassed,
}
