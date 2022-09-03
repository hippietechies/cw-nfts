use cosmwasm_std::FullDelegation;
use cw0::Expiration;
use cw_storage_plus::Map;
use cosmwasm_std::Uint128;
use cw_storage_plus::UniqueIndex;
use cosmwasm_std::Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};

use cw_storage_plus::{Index, IndexList, IndexedMap, Item};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub nft_contract: Addr,
    pub validator: String,
    pub owner: Addr,
    pub denom: String,

    //pub tokens: IndexedMap<&'a str, TokenInfo<Extension>, TokenIndexes<'a, Extension>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UndelegatingInfo {
    pub mature_at: Expiration,
    pub amount: String,
}

pub struct StakingContract<'a>
{
    pub state: Item<'a, State>,
    // u64 -> block_height
    // u128 -> coin amount
    pub rewards: Map<'a, u64, u128>, //store how much coins per blockheight
    // u32 -> token_id
    // u64 -> block_height
    pub token_claims: Map<'a, u32, u64>, //store for which tokenid has withdrawn at which blockheight
    // u32 -> token_id
    // u64 -> coin_amount
    pub token_undelegating: Map<'a, u32, UndelegatingInfo>, //store undelegating tokenid and amount withdrawable
}


impl Default for StakingContract<'static>
{
    fn default() -> Self {
        Self::new(
            "state",
            "rewards",
            "token_claims",
            "token_undelegating",
        )
    }
}

impl<'a> StakingContract<'a>
{
    fn new(
        state_key: &'a str,
        rewards_key: &'a str,
        token_claims_key: &'a str,
        token_undelegating_key: &'a str,
    ) -> Self {
        Self {
            state: Item::new(state_key),
            rewards: Map::new(rewards_key),
            token_claims: Map::new(token_claims_key),
            token_undelegating: Map::new(token_undelegating_key),
        }
    }
}

/// get_coins returns coin of denom
fn get_coins(coins: &[Coin], denom: &str) -> u128 {
    coins
        .iter()
        .find(|c| c.denom == denom)
        .map(|m| m.amount)
        .unwrap_or(Uint128::new(0)).u128()
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TokenMarketInfo {
    pub token_id: u32,
}

/// DelegationResponse is data format returned from StakingRequest::Delegation query
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DelegationResponse {
    pub delegation: Option<FullDelegation>,
}
