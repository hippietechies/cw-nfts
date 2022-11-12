use cw721_base::Cw721Contract;
use cw721_metadata_onchain::Cw721MetadataContract;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use cosmwasm_std::{Addr, BlockInfo, StdResult, Storage, Empty, Coin};

use cw721::{OwnerOfResponse, NftInfoResponse, ContractInfoResponse, Expiration};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};

pub type Cw721ExtendedContract<'a> = Cw721MetadataContract<'a>;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct BagOfCoins {
    /// When the Approval expires (maybe Expiration::never)
    pub owner: Addr,
    pub bag: Vec<Coin>,
    pub expires: Expiration,
}

/// Shows Address string of staking contract
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct StakingContractResponse {
    pub contract: String,
}

