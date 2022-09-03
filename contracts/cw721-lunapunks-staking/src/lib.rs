mod error;
mod execute;
pub mod msg;
mod query;
pub mod state;

pub use crate::error::ContractError;
pub use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
pub use crate::state::StakingContract;

#[cfg(not(feature = "library"))]
pub mod entry {
    use crate::msg::MigrateMsg;
use cosmwasm_std::Reply;
use super::*;

    use cosmwasm_std::entry_point;
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

    // This makes a conscious choice on the various generics used by the contract
    #[entry_point]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        let tract = StakingContract::default();
        tract.instantiate(deps, env, info, msg)
    }

    #[entry_point]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        let tract = StakingContract::default();
        tract.execute(deps, env, info, msg)
    }

    #[entry_point]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        let tract = StakingContract::default();
        tract.query(deps, env, msg)
    }

    #[entry_point]
    pub fn reply(
        deps: DepsMut,
        env: Env,
        reply: Reply,
    ) -> Result<Response, ContractError> {
        let tract = StakingContract::default();
        tract.reply(deps, env, reply)
    }

    #[entry_point]
    pub fn migrate(
        deps: DepsMut,
        env: Env,
        msg: MigrateMsg
    ) -> Result<Response, ContractError> {
        let tract = StakingContract::default();
        tract.migrate(deps, env, msg)
    }
}
