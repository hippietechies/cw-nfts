#[cfg(not(feature = "library"))]
pub use crate::msg::{LunaPunkExecuteMsg, LunaPunkQueryMsg, MigrateMsg};
pub use crate::state::{Cw721ExtendedContract,Extension};
use crate::execute::{mint, instantiate as instantiate_luna_punks_contract, release};
use crate::query::{all_tokens, tokens, owner_tokens};

use super::*;

use cosmwasm_std::{entry_point, to_binary};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw721_base::{ContractError, InstantiateMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw4-stake";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  msg: InstantiateMsg,
) -> StdResult<Response> {
  instantiate_luna_punks_contract(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
  pub fn execute(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  msg: LunaPunkExecuteMsg<Extension>,
) -> Result<Response, ContractError> {
  println!("execute:{:?}", msg);
  match msg {
      LunaPunkExecuteMsg::Mint(msg) => mint(deps, env, info, msg),
      LunaPunkExecuteMsg::Release { bids } => release(deps, env, info, bids),
      // _ => {
      //     println!("sdefault");
      //     Ok(Response::new())
      // },
      _ => Cw721ExtendedContract::default().execute(deps, env, info, msg.into()),
  }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: LunaPunkQueryMsg) -> StdResult<Binary> {
  println!("query:{:?}", msg);

  match msg {
      // LunaPunkQueryMsg::StakingContract { } => to_binary(&staking_contract(deps)?),

      LunaPunkQueryMsg::OwnerTokens { owner, start_after } => {
          to_binary(&owner_tokens(deps, owner, start_after)?)
      },
      LunaPunkQueryMsg::Tokens {
          owner, start_after, skip, limit
      } => to_binary(&tokens(deps, owner, start_after, skip, limit)?),
      LunaPunkQueryMsg::AllTokens {
          start_after, skip, limit
      } => to_binary(&all_tokens(deps, start_after, skip, limit)?),
      _ => Cw721ExtendedContract::default().query(deps, env, msg.into()),
  }
}

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
//     let tract = Cw721ExtendedContract::default();//Cw721ExtendedContract::default();//Cw721Contract::<Extension, Empty>::default();
//     tract.migrate(deps, env, msg)
// }

