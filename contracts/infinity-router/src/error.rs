use cosmwasm_std::{StdError, Uint128};
use cw_utils::PaymentError;
use infinity_shared::InfinityError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("{0}")]
    InfinityError(#[from] InfinityError),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Insufficient funds: expected {expected}, received {received}")]
    InsufficientFunds {
        expected: Uint128,
        received: Uint128,
    },

    #[error("Message can only be called by contract itself")]
    OnlySelf,

    #[error("Deadline passed")]
    DeadlinePassed,

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Unable to match order")]
    UnableToMatchOrder,

    #[error("Swap failed")]
    SwapFailed,
}
