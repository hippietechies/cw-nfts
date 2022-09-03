use crate::state::UndelegatingInfo;
use cosmwasm_std::Addr;
use cosmwasm_std::Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw721::Expiration;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub nft_contract: String,
    pub validator: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Revest {},
    Release { release_funds: Vec<Coin>},
    UnstakeRewards { token_id: u32 },
    ClaimRewards { token_id: u32 },
    ChangeNftContract { nft_contract: String },
    ChangeValidator { validator: String },
    _BondAllTokens { is_reward: bool },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetRewards { token_id: u32 },
    GetUndelegatingRewards { token_id: u32 },
    GetClaimableRewards { token_id: u32 },
    GetAllUndelegatingRewards { },
    GetAllClaimableRewards { },
    GetValidator { },
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardsResponse {
    pub amount: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct UndelegatingRewardsResponse {
    pub undelegating_rewards: Option<UndelegatingInfo>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct AllUndelegatingRewardsResponse {
    pub total_undelegating_reward: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct GetValidatorResponse {
    pub nft_contract: String,
    pub validator: String,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimableRewardsResponse {
    pub claimable_amount: String,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllClaimableRewardsResponse {
    pub total_claimable_amount: String,
}
