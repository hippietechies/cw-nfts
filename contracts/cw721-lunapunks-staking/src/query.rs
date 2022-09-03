use crate::msg::AllClaimableRewardsResponse;
use crate::msg::AllUndelegatingRewardsResponse;
use crate::msg::GetValidatorResponse;
use crate::msg::RewardsResponse;
use crate::msg::UndelegatingRewardsResponse;
use crate::msg::ClaimableRewardsResponse;
use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdResult};
use std::convert::{TryFrom,TryInto};
use std::marker::PhantomData;
use cw_storage_plus::Bound;

use crate::msg::{QueryMsg};
use crate::state::{StakingContract};

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

impl<'a> StakingContract<'a>
{
    pub fn calc_limit(&self, request: Option<u32>) -> usize {
        request.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize
    }
    pub fn calc_skip(&self, request: Option<u32>, limit: usize) -> usize {
        request.unwrap_or(0) as usize * limit
    }

    pub fn get_rewards(
        &self,
        deps: Deps,
        env: Env,
        token_id: u32,
    ) -> StdResult<RewardsResponse> {
        // get current claim block height of token
        let block_height = self.token_claims.may_load(deps.storage, token_id)?;
        let block_height = block_height.unwrap_or(0u64);
        // with current block height of given token, get undelegating rewards
        // let start = Some(Bound::Exclusive(U64Key::from(block_height).wrapped));
        let start = Some(Bound::Inclusive((block_height, PhantomData)));

        let amount: u128 = self.rewards
        .range(deps.storage, start, None, Order::Ascending)
        .map(|result| result.and_then(|(_, reward)| Ok(reward)).ok().unwrap_or(0u128))
        .sum();

        Ok(RewardsResponse {
            amount: amount.to_string()
        })
    }

    pub fn get_undelegating_rewards(
        &self,
        deps: Deps,
        env: Env,
        token_id: u32,
    ) -> StdResult<UndelegatingRewardsResponse> {
        let undelegating_infos = self.token_undelegating.may_load(deps.storage, token_id.into())?;

        Ok(UndelegatingRewardsResponse {
            undelegating_rewards: undelegating_infos,
        })
    }

    pub fn get_claimable_rewards(
        &self,
        deps: Deps,
        env: Env,
        token_id: u32,
    ) -> StdResult<ClaimableRewardsResponse> {
        let undelegating_infos = self.token_undelegating.may_load(deps.storage, token_id)?;

        let undelegating_sum = match undelegating_infos {
            Some(undelegating_infos) => { undelegating_infos.amount },
            None => { "0".to_string() }
        };

        Ok(ClaimableRewardsResponse {
            claimable_amount: undelegating_sum,
        })
    }

    pub fn get_all_undelegating_rewards(
        &self,
        deps: Deps,
        env: Env,
    ) -> StdResult<AllUndelegatingRewardsResponse> {
        let total_undelegating_reward: u128 = self.token_undelegating
            .range(deps.storage, None, None, Order::Ascending)
            .map(|item|item.and_then(|(_,state)| Ok(state.amount.parse::<u128>().unwrap())).ok().unwrap_or(0u128))
            .sum();

        Ok(AllUndelegatingRewardsResponse {
            total_undelegating_reward: total_undelegating_reward.to_string(),
        })
    }

    pub fn get_all_claimable_rewards(
        &self,
        deps: Deps,
        env: Env,
    ) -> StdResult<AllClaimableRewardsResponse> {
        let total_undelegating_reward: u128 = self.token_undelegating
            .range(deps.storage, None, None, Order::Ascending)
            .map(|item|item.and_then(|(_,state)| {
                if state.mature_at.is_expired(&env.block) {
                    return Ok(state.amount.parse::<u128>().unwrap())
                }
                Ok(0u128)
            }).ok().unwrap_or(0u128))
            .sum();

        Ok(AllClaimableRewardsResponse {
            total_claimable_amount: total_undelegating_reward.to_string(),
        })
    }

    pub fn get_state(
        &self,
        deps: Deps,
    ) -> StdResult<GetValidatorResponse> {
        let state = self.state.load(deps.storage)?;

        Ok(GetValidatorResponse {
            validator: state.validator,
            nft_contract: state.nft_contract.to_string(),
        })
    }

    pub fn query(&self, deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::GetRewards { token_id } =>
                to_binary(&self.get_rewards(deps, env, token_id)?),
            QueryMsg::GetUndelegatingRewards { token_id } =>
                to_binary(&self.get_undelegating_rewards(deps, env, token_id)?),
            QueryMsg::GetClaimableRewards { token_id } =>
                to_binary(&self.get_claimable_rewards(deps, env, token_id)?),
            QueryMsg::GetAllUndelegatingRewards { } =>
                to_binary(&self.get_all_undelegating_rewards(deps, env)?),
            QueryMsg::GetAllClaimableRewards { } =>
                to_binary(&self.get_all_claimable_rewards(deps, env)?),
            QueryMsg::GetValidator { } =>
                to_binary(&self.get_state(deps)?),
        }
    }
}

