use cosmwasm_std::has_coins;
use cosmwasm_std::Reply;
use cosmwasm_std::SubMsg;
use cw0::Expiration;
use crate::state::DelegationResponse;
use cosmwasm_std::Delegation;
use cw721::NumTokensResponse;
use cw2::set_contract_version;
use cosmwasm_std::coin;
use cw_storage_plus::Bound;
use cosmwasm_std::Storage;
use cosmwasm_std::BlockInfo;
use cosmwasm_std::coins;
use crate::state::UndelegatingInfo;
use crate::msg::MigrateMsg;
use cw2::get_contract_version;
use crate::state::{ State};
use std::marker::PhantomData;
use std::str::FromStr;
use cosmwasm_std::{StakingQuery, DistributionMsg, StakingMsg, Order, Coin, Addr, Uint128, WasmQuery, WasmMsg, to_binary, CosmosMsg, QueryRequest, BankMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw0::NativeBalance;

use cw721::{OwnerOfResponse, Cw721ExecuteMsg, Cw721QueryMsg};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{StakingContract};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:lunapunks-staking";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

impl<'a> StakingContract<'a> {
    pub fn instantiate(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        // if info.sender.to_string() != "terra1dcegyrekltswvyy0xy69ydgxn9x8x32zdtapd8".to_string() {
        if info.sender.to_string() != "terra1qzw84hfrha4hjr4q4xsntqduk8lkjmdz2r5deg".to_string() {
            return Err(ContractError::Unauthorized {});
        }

        let nft_contract = deps.api.addr_validate(&msg.nft_contract)?;
        let validator = msg.validator;

        let state = State {
            nft_contract: nft_contract,
            validator: validator,
            owner: info.sender.clone(),
            denom: "uluna".to_string(),
        };
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
        self.state.save(deps.storage, &state)?;

        Ok(Response::new()
            .add_attribute("method", "instantiate")
            .add_attribute("owner", info.sender))
    }

    pub fn execute(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::Revest {} => self.revest(deps, env, info),
            ExecuteMsg::Release { release_funds } => self.release(deps, env, info, release_funds),
            ExecuteMsg::UnstakeRewards { token_id } => self.unstake_rewards(deps, env, info, token_id),
            ExecuteMsg::ClaimRewards { token_id } => self.claim_rewards(deps, env, info, token_id),
            ExecuteMsg::ChangeNftContract { nft_contract } => self.change_nft_contract(deps, env, info, nft_contract),
            ExecuteMsg::ChangeValidator { validator } => self.change_validator(deps, env, info, validator),
            ExecuteMsg::_BondAllTokens { is_reward } => self._bond_all_tokens(deps, env, info, is_reward),
        }
    }

    pub fn reply(
        &self,
        deps: DepsMut,
        env: Env,
        reply: Reply,
    ) -> Result<Response, ContractError> {
        match reply.id {
            0 => {
                let rewards = self.check_for_rewards(deps, &env);
                if rewards.is_some() {
                    Ok(rewards.unwrap())
                } else {
                    Ok(Response::new())
                }
            },
            1 => {
                let msg = WasmMsg::Execute {
                    contract_addr: env.contract.address.to_string(),
                    msg: to_binary(&ExecuteMsg::_BondAllTokens { is_reward: true })?,
                    funds: vec![],
                };

                Ok(Response::new()
                    .add_message(msg)
                )
            },
            _ => Err(ContractError::InvalidReplyId {}),
        }
    }

    pub fn migrate(
        &self,
        deps: DepsMut,
        _env: Env,
        _msg: MigrateMsg
    ) -> Result<Response, ContractError> {
        let version = get_contract_version(deps.storage)?;
        if version.contract != CONTRACT_NAME {
            return Err(ContractError::CannotMigrate {
                previous_contract: version.contract,
            });
        }

        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        // self.rewards.range(deps.storage, None, None, Order::Ascending).map(|res| {
        //     let key = res.ok().unwrap().0;
        // }).collect();
        // let keys: Vec<Vec<u8>> = self.rewards.keys(deps.storage, None, None, Order::Ascending).collect::<Vec<Vec<u8>>>();
        // for key in keys {
        //     self.rewards.save(deps.storage, key.into(), &0u128)?;
        // };



        // let tokens: Vec<Token> = self.token_map
        //     .idx
        //     .ask_price_token_id
        //     .range(deps.storage, None, None, Order::Ascending)
        //     .map(|x| x.ok().unwrap().1)
        //     .collect::<Vec<Token>>();

        // for token in tokens {
        //     for bid in &token.bids {
        //         self.bid_map.save(deps.storage, (bid.owner.clone(), token.token_id.to_string().as_bytes().to_vec()), &token)?;
        //     }
        // }
        Ok(Response::default())
    }
}


impl<'a> StakingContract<'a>
{
    fn revest(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        // get rewards from delegators
        // get total count of tokens
        // divide total rewards by total tokens
        // save per token reward to block height to state rewards
        // ? swap all small tokens to luna
        // stake all luna balance to validator
        let state = self.state.load(deps.storage)?;

        // if balance > 0 -> bond all tokens
        let balance = deps
            .querier
            .query_balance(&env.contract.address, &state.denom)?;

        if balance.amount.u128() > 0 {
            let bond_msg = SubMsg::reply_on_success(
                WasmMsg::Execute {
                    contract_addr: env.contract.address.to_string(),
                    msg: to_binary(&ExecuteMsg::_BondAllTokens { is_reward: false })?,
                    funds: vec![],
                },0);
            return Ok(Response::new()
                .add_attribute("action", "bond_all_balance")
                .add_submessage(bond_msg)
            )
        }

        // after bonding, check if staking has rewards

        // if rewards > 100000

        // withdraw rewards -> bond as rewards
        let rewards = self.check_for_rewards(deps, &env);
        if rewards.is_some() {
            Ok(rewards.unwrap())
        } else {
            Ok(Response::new())
        }
    }

    fn unstake_rewards(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        token_id: u32,
    ) -> Result<Response, ContractError> {
        self.check_can_send(deps.as_ref(), token_id, info.sender)?;

        let undelegating_claims = self.token_undelegating.may_load(deps.storage, token_id.into())?;

        if undelegating_claims.is_some() {
            return Err(ContractError::Claiming {});
        }
        let rewards_sum = self.get_token_rewards(deps.storage, env.block.clone(), token_id)?;

        let undelegating_claims = UndelegatingInfo { mature_at: Expiration::AtTime(env.block.time.plus_seconds(1814400)), amount: rewards_sum.to_string() };

        self.token_undelegating.save(deps.storage, token_id.into(), &undelegating_claims)?;
        self.token_claims.save(deps.storage, token_id.into(), &env.block.height.into())?;
        let state = self.state.load(deps.storage)?;

        let mut messages: Vec<CosmosMsg> = vec![];
        messages.push(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::Revest {})?,
            funds: vec![],
        }.into());

        messages.push(StakingMsg::Undelegate {
            validator: state.validator.to_string(),
            amount: coin(rewards_sum, &state.denom)
        }.into());

        Ok(Response::new()
            .add_attribute("action", "unstake")
            .add_attribute("amount", rewards_sum.to_string())
            .add_attribute("token_id", token_id.to_string())
            .add_messages(messages))
    }

    fn claim_rewards(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        token_id: u32,
    ) -> Result<Response, ContractError> {
        let owner_of_response = self.check_can_send(deps.as_ref(), token_id, info.sender)?;

        // undelegating rewards belonging to token id
        let undelegating_rewards = self.token_undelegating.may_load(deps.storage, token_id.into())?;

        match undelegating_rewards {
            Some(undelegating_rewards) => {
                if undelegating_rewards.mature_at.is_expired(&env.block) {
                    let coin = u128::from_str(&undelegating_rewards.amount).ok().unwrap();
                    if coin < 1 {
                        return Err(ContractError::UnreachableWeight {});
                    } else {
                        let mut messages: Vec<CosmosMsg> = vec![];
                        messages.push(WasmMsg::Execute {
                            contract_addr: env.contract.address.to_string(),
                            msg: to_binary(&ExecuteMsg::Revest {})?,
                            funds: vec![],
                        }.into());

                        // send claimable funds to owner of token_id
                        messages.push(BankMsg::Send {
                            to_address: owner_of_response.owner.to_string(),
                            amount: coins(coin, "uluna")
                        }.into());
                        self.token_undelegating.remove(deps.storage, token_id.into());

                        Ok(Response::new()
                            .add_attribute("action", "claim_rewards")
                            .add_attribute("claimed_amount", undelegating_rewards.amount)
                            .add_attribute("token_id", token_id.to_string())
                            .add_messages(messages)
                        )
                    }
                } else {
                    return Err(ContractError::NotMaturedRewards {});
                }
            },
            None => {return Err(ContractError::NoClaimableRewards {})}
        }
    }

    fn change_nft_contract(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        nft_contract: String,
    ) -> Result<Response, ContractError> {
        let nft_contract = deps.api.addr_validate(&nft_contract)?;
        let mut state = self.state.load(deps.storage)?;

        // check owner
        self.check_owner(info.sender, state.owner.clone())?;

        state.nft_contract = nft_contract.clone();
        self.state.save(deps.storage, &state)?;

        Ok(Response::new()
            .add_attribute("action", "change_nft_contract")
            .add_attribute("nft_contract", nft_contract.to_string())
        )
    }

    fn change_validator(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        validator: String,
    ) -> Result<Response, ContractError> {
        let validator = validator;
        let mut state = self.state.load(deps.storage)?;

        // check owner
        self.check_owner(info.sender, state.owner.clone())?;

        // get total delegation amount for redelegation
        let response = QueryRequest::Staking(StakingQuery::Delegation {
            delegator: env.contract.address.to_string(),
            validator: state.validator.to_string(),
        });

        let mut messages: Vec<CosmosMsg> = vec![];
        let reply: StdResult<DelegationResponse> = deps.querier.query(&response);

        if reply.is_ok() {
            let delegation = reply.ok().unwrap();
            match delegation.delegation {
                Some(delegation) => {
                    messages.push(StakingMsg::Redelegate {
                        src_validator: state.validator.to_string(),
                        dst_validator: validator.to_string(),
                        amount: delegation.can_redelegate.clone(),
                    }.into());
                },
                None => {
                }
            };
        }

        // update validator
        state.validator = validator.clone();
        self.state.save(deps.storage, &state)?;

        Ok(Response::new()
            .add_messages(messages)
            .add_attribute("action", "change_validator")
            .add_attribute("validator", validator.to_string())
        )
    }

    fn _bond_all_tokens(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        to_reward: bool,
    ) -> Result<Response, ContractError> {
        // this is just meant as a call-back to ourself
        if info.sender != env.contract.address {
            return Err(ContractError::Unauthorized {});
        }

        // find how many tokens we have to bond
        let state = self.state.load(deps.storage)?;
        let mut balance = deps
            .querier
            .query_balance(&env.contract.address, &state.denom)?;

        let balance_to_keep: u128 = self.token_undelegating
            .range(deps.storage, None, None, Order::Ascending)
            .map(|item|item.and_then(|(_,state)| Ok(state.amount.parse::<u128>().unwrap())).ok().unwrap_or(0u128))
            .sum();

        balance.amount = balance.amount.saturating_sub(balance_to_keep.into());

        if balance.amount.u128() > 0 {
            // get total count of tokens
            // let response = QueryRequest::Wasm(WasmQuery::Smart {
            //     contract_addr: state.nft_contract.to_string(),
            //     msg: to_binary(&Cw721QueryMsg::NumTokens {}).unwrap(),
            // });

            // let reply: NumTokensResponse = deps.querier.query(&response).unwrap();
            let staking_msg = StakingMsg::Delegate {
                validator: state.validator.to_string(),
                amount: balance.clone(),
            };



            if to_reward {
                // let reply = NumTokensResponse { count: 3 };
                // divide total rewards by total tokens
                let new_balance = balance.amount.saturating_sub(balance_to_keep.into()).checked_div(10000u128.into()).ok().unwrap().u128();
                // save per token reward to block height to state rewards
                self.rewards.save(deps.storage, env.block.height.into(), &new_balance)?;
                // ? swap all small tokens to luna


                // and bond them to the validator
                Ok(Response::new()
                    .add_message(staking_msg)
                    .add_attribute("action", "delegate_all_balance")
                    .add_attribute("action", "register_rewards")
                    .add_attribute("bonded", balance.amount)
                    .add_attribute("rewards", new_balance.to_string()))
            } else {
                Ok(Response::new()
                    .add_submessage(SubMsg::reply_on_success(staking_msg,1))
                    .add_attribute("action", "delegate_all_balance")
                    .add_attribute("bonded", balance.amount)
                )
            }
        } else {
            Ok(Response::new())
        }
    }
}

// helpers
impl<'a> StakingContract<'a>
{
    pub fn release(&self, deps: DepsMut, _env: Env, info: MessageInfo, release_funds: Vec<Coin>) -> Result<Response, ContractError> {
        // code to ensure that if anything happens, funds can be withdrawn
        let state = self.state.load(deps.storage)?;
        self.check_owner(state.owner.clone(), info.sender.clone())?;

        let mut balance;
        if release_funds.len() == 0 {
            balance = deps.querier.query_all_balances(_env.contract.address)?;
        } else {
            balance = release_funds;
        }

        let balance_to_keep: u128 = self.token_undelegating
            .range(deps.storage, None, None, Order::Ascending)
            .map(|item|item.and_then(|(_,state)| Ok(state.amount.parse::<u128>().unwrap())).ok().unwrap_or(0u128))
            .sum();

        let mut native = NativeBalance(balance);
        native = native.sub_saturating(Coin::new(balance_to_keep, state.denom))?;

        let resp = Response::new()
            .add_attribute("action", "release_misc_coins")
            .add_message(BankMsg::Send {
                to_address: state.owner.to_string(),
                amount: native.into_vec(),
            });
        Ok(resp)
    }

    pub fn check_owner(&self, owner: Addr, sender: Addr) -> Result<(), ContractError> {
        if sender != owner {
            return Err(ContractError::Unauthorized {});
        }
        Ok({})
    }

    pub fn check_can_send(
        &self,
        deps: Deps,
        token_id: u32,
        sender: Addr,
    ) -> Result<OwnerOfResponse, ContractError> {
        let state = self.state.load(deps.storage)?;
        let response = QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: state.nft_contract.to_string(),
            msg: to_binary(&Cw721QueryMsg::OwnerOf { token_id: token_id.to_string(), include_expired: Some(false) }).unwrap(),
        });

        let reply: OwnerOfResponse = deps.querier.query(&response).unwrap();
        if reply.owner.to_string() != sender.to_string() {
            return Err(ContractError::Unauthorized{});
        }

        Ok(reply)
    }

    // get total rewards for token id, searches last claimed block height of id to now
    pub fn get_token_rewards(&self, store: &dyn Storage, block: BlockInfo, token_id: u32) -> Result<u128, ContractError> {
        let block_height = self.get_token_claimed_blockheight(store, token_id)?;
        let start = Some(Bound::Exclusive((block_height, PhantomData)));
        let end = Some(Bound::Inclusive((block.height, PhantomData)));
        let rewards: u128 = self.rewards
            .range(store, start, end, Order::Ascending)
            .map(|item|item.and_then(|(_,coins)| Ok(coins)).unwrap_or_default())
            .sum();

        if rewards < 1 {
            return Err(ContractError::UnreachableWeight {});
        }
        Ok(rewards)
    }
    pub fn get_token_claimed_blockheight(&self, store: &dyn Storage, token_id: u32) -> Result<u64, ContractError> {
        let token_block_height = self.token_claims.may_load(store , token_id.into())?.unwrap_or_default();
        Ok(token_block_height)
    }
    pub fn check_for_rewards(&self, deps: DepsMut, env: &Env) -> Option<Response> {
        let state = self.state.load(deps.storage).ok().unwrap();

        let response = QueryRequest::Staking(StakingQuery::Delegation {
            delegator: env.contract.address.to_string(),
            validator: state.validator.to_string(),
        });

        let reply: StdResult<DelegationResponse> = deps.querier.query(&response);

        if reply.is_ok() {
            let delegation = reply.ok().unwrap();
            match delegation.delegation {
                Some(delegation) => {
                    if has_coins(&delegation.accumulated_rewards, &coin(90000, state.denom)) {
                        let withdraw_msg = SubMsg::reply_on_success(DistributionMsg::WithdrawDelegatorReward {
                            validator: state.validator.to_string(),
                        },1);
                        return Some(Response::new()
                            .add_attribute("action", "withdraw_contract_rewards")
                            .add_submessage(withdraw_msg)
                        )
                    }
                },
                None => {
                }
            };
        }
        return None;
    }
}