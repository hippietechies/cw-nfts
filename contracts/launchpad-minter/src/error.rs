use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("password '{0}', hash '{1}'")]
    Janan(String,String),

    #[error("Insufficient value assigned for mint transaction")]
    Insufficient {},

    #[error("Public mint is still on the way")]
    Minttime {},

    #[error("Unable to mint over total capped supply")]
    Overmint {},

    #[error("Unable to mint over personal total capped supply")]
    Overmintpersonal {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
