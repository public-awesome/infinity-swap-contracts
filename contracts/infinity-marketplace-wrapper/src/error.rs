use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Invalid ask: {0}")]
    InvalidAsk(String),

    #[error("Invalid bid: {0}")]
    InvalidBid(String),

    #[error("Price mismatch: {0}")]
    PriceMismatch(String),
}
