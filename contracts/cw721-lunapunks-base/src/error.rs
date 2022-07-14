use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum LunaPunksContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("token_id already claimed")]
    Claimed {},

    #[error("Address already minted")]
    Minted {},

    #[error("Cannot set approval that is already expired")]
    Expired {},

    #[error("Transaction has to be properly funded")]
    Unfunded {},

    #[error("Address is unknown in tx")]
    UnknownAddress {},

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },
}
