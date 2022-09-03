use crate::state::StakingContractResponse;
use cosmwasm_std::{to_binary, Binary, BlockInfo, Deps, Env, Order, StdError, StdResult, Addr};

use cw0::maybe_addr;
use cw721::{
    AllNftInfoResponse, ApprovalsResponse, ContractInfoResponse, Cw721Query,
    Expiration, NumTokensResponse, OwnerOfResponse, TokensResponse, NftInfoResponse, OperatorsResponse, ApprovalResponse
};
use cw721_base::state::Approval;
use cw_storage_plus::Bound;

use base64::encode;
use crate::msg::{LunaPunkQueryMsg};
use crate::state::{Cw721ExtendedContract, Extension };
use std::convert::TryInto;
use std::marker::PhantomData;

use cw721_base::msg::QueryMsg as CW721QueryMsg;

const DEFAULT_LIMIT: u32 = 10;
const DEFAULT_PAGE: u32 = 0;
const MAX_LIMIT: u32 = 30;


// pub fn staking_contract(deps: Deps) -> StdResult<StakingContractResponse> {
//     let contract = Cw721ExtendedContract::default();

//     let staking_addr = contract.staking_contract.load(deps.storage)?;
//     Ok(StakingContractResponse {
//         contract: staking_addr.to_string(),
//     })
// }

pub fn owner_tokens(
    deps: Deps,
    owner: String,
    start_after: Option<String>,
) -> StdResult<NumTokensResponse> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let start = start_after.map(Bound::exclusive);
    let contract = Cw721ExtendedContract::default();

    let count: u64 = contract
        .tokens
        .idx
        .owner
        .prefix(owner_addr)
        .keys(deps.storage, start, None, Order::Ascending)
        .count() as u64;

    Ok(NumTokensResponse {
        count,
    })
}


pub fn tokens(
    deps: Deps,
    owner: String,
    start_after: Option<String>,
    page: Option<u32>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let contract = Cw721ExtendedContract::default();
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let skip = page.unwrap_or(DEFAULT_PAGE) as usize * limit;

    let mut start: Option<Bound<Vec<u8>>> = None;
    if start_after.is_some() {
        start = Some(Bound::Exclusive((start_after.unwrap().parse::<u64>().unwrap().to_be_bytes().to_vec(), PhantomData)));
    }
    let owner_addr = deps.api.addr_validate(&owner)?;
    let tokens: Vec<String> = contract
        .tokens
        .idx
        .owner
        .prefix(owner_addr)
        .keys(deps.storage, start, None, Order::Ascending)
        .skip(skip)
        .take(limit)
        .map(|x| x.map(|y| u64::from_be_bytes(y.try_into().unwrap()).to_string()))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(TokensResponse { tokens })
}

pub fn all_tokens(
    deps: Deps,
    start_after: Option<String>,
    page: Option<u32>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let contract = Cw721ExtendedContract::default();

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let skip = page.unwrap_or(DEFAULT_PAGE) as usize * limit;
    // let start = start_after.map(Bound::exclusive);
    let mut start: Option<Bound<Vec<u8>>> = None;
    if start_after.is_some() {
        // let test = start_after.map(|x| x.parse::<u64>()).map(|x| x.unwrap().to_be_bytes().to_vec()).unwrap();
        // let start = Some(Bound::ExclusiveRaw(test));
        // let start = Some(Bound::ExclusiveRaw(start_after.unwrap().parse::<u64>().unwrap().to_be_bytes().to_vec()));
        start = Some(Bound::Exclusive((start_after.unwrap().parse::<u64>().unwrap().to_be_bytes().to_vec(), PhantomData)));
    }

    let tokens: StdResult<Vec<String>> = contract
        .tokens
        .range(deps.storage, start, None, Order::Ascending)
        .skip(skip)
        .take(limit)
        .map(|item| item.map(|(k, _)| u64::from_be_bytes(k.try_into().unwrap()).to_string()))
        .collect();

    Ok(TokensResponse { tokens: tokens? })
}




// impl<'a> Cw721Query<Extension> for Cw721ExtendedContract<'a>
// {
//     fn contract_info(&self, deps: Deps) -> StdResult<ContractInfoResponse> {
//         self.contract_info.load(deps.storage)
//     }

//     fn num_tokens(&self, deps: Deps) -> StdResult<NumTokensResponse> {
//         let count = self.token_count(deps.storage)?;
//         Ok(NumTokensResponse { count })
//     }

//     fn nft_info(&self, deps: Deps, token_id: String) -> StdResult<NftInfoResponse<Extension>> {
//         let info = self.tokens.load(deps.storage, convert_id_string_to_bytes(token_id))?;

//         let mut extension = info.extension.unwrap();
//         let mut image = extension.image_data.unwrap();
//         image.insert_str(5usize, "xmlns='http://www.w3.org/2000/svg' ");
//         image = encode(image.clone());
//         image.insert_str(0usize, "data:image/svg+xml;base64,");
//         extension.image = Some(image);
//         extension.image_data = None;

//         Ok(NftInfoResponse {
//             token_uri: info.token_uri,
//             extension: Some(extension),
//         })
//     }

//     fn owner_of(
//         &self,
//         deps: Deps,
//         env: Env,
//         token_id: String,
//         include_expired: bool,
//     ) -> StdResult<OwnerOfResponse> {
//         let info = self.tokens.load(deps.storage, convert_id_string_to_bytes(token_id))?;
//         Ok(OwnerOfResponse {
//             owner: info.owner.to_string(),
//             approvals: humanize_approvals(&env.block, info.approvals, include_expired),
//         })
//     }

//     fn approval(
//         &self,
//         deps: Deps,
//         env: Env,
//         token_id: String,
//         spender: String,
//         include_expired: bool,
//     ) -> StdResult<ApprovalResponse> {
//         let token = self.tokens.load(deps.storage, convert_id_string_to_bytes(token_id))?;

//         // token owner has absolute approval
//         if token.owner == spender {
//             let approval = cw721::Approval {
//                 spender: token.owner.to_string(),
//                 expires: Expiration::Never {},
//             };
//             return Ok(ApprovalResponse { approval });
//         }

//         let filtered: Vec<_> = token
//             .approvals
//             .into_iter()
//             .filter(|t| t.spender == spender)
//             .filter(|t| include_expired || !t.is_expired(&env.block))
//             .map(|a| cw721::Approval {
//                 spender: a.spender.into_string(),
//                 expires: a.expires,
//             })
//             .collect();

//         if filtered.is_empty() {
//             return Err(StdError::not_found("Approval not found"));
//         }
//         // we expect only one item
//         let approval = filtered[0].clone();

//         Ok(ApprovalResponse { approval })
//     }

//     /// approvals returns all approvals owner given access to
//     fn approvals(
//         &self,
//         deps: Deps,
//         env: Env,
//         token_id: String,
//         include_expired: bool,
//     ) -> StdResult<ApprovalsResponse> {
//         let token = self.tokens.load(deps.storage, convert_id_string_to_bytes(token_id))?;
//         let approvals: Vec<_> = token
//             .approvals
//             .into_iter()
//             .filter(|t| include_expired || !t.is_expired(&env.block))
//             .map(|a| cw721::Approval {
//                 spender: a.spender.into_string(),
//                 expires: a.expires,
//             })
//             .collect();

//         Ok(ApprovalsResponse { approvals })
//     }

//     /// operators returns all operators owner given access to
//     fn operators(
//         &self,
//         deps: Deps,
//         env: Env,
//         owner: String,
//         include_expired: bool,
//         start_after: Option<String>,
//         page: Option<u32>,
//     ) -> StdResult<OperatorsResponse> {
//         let limit = (DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
//         let skip = page.unwrap_or(DEFAULT_PAGE) as usize * limit;
//         let start_addr = maybe_addr(deps.api, start_after)?;
//         let start = start_addr.map(|addr| Bound::Exclusive(addr.to_string().as_bytes().to_vec()));

//         let owner_addr = deps.api.addr_validate(&owner)?;
//         let res: StdResult<Vec<_>> = self
//             .operators
//             .prefix(&owner_addr)
//             .range(deps.storage, start, None, Order::Ascending)
//             .filter(|r| {
//                 include_expired || r.is_err() || !r.as_ref().unwrap().1.is_expired(&env.block)
//             })
//             .skip(skip)
//             .take(limit)
//             .map(parse_approval)
//             .collect();
//         Ok(OperatorsResponse { operators: res? })
//     }


//     fn all_tokens(
//         &self,
//         deps: Deps,
//         start_after: Option<String>,
//         limit: Option<u32>,
//     ) -> StdResult<TokensResponse> {
//         let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
//         // let start = start_after.map(Bound::exclusive);
//         let mut start: Option<Bound> = None;
//         if start_after.is_some() {
//             // let test = start_after.map(|x| x.parse::<u64>()).map(|x| x.unwrap().to_be_bytes().to_vec()).unwrap();
//             // let start = Some(Bound::ExclusiveRaw(test));
//             // let start = Some(Bound::ExclusiveRaw(start_after.unwrap().parse::<u64>().unwrap().to_be_bytes().to_vec()));
//             start = Some(Bound::Exclusive(start_after.unwrap().parse::<u64>().unwrap().to_be_bytes().to_vec()));
//         }

//         let tokens: StdResult<Vec<String>> = self
//             .tokens
//             .range(deps.storage, start, None, Order::Ascending)
//             .take(limit)
//             .map(|item| item.map(|(k, _)| u64::from_be_bytes(k.try_into().unwrap()).to_string()))
//             .collect();

//         Ok(TokensResponse { tokens: tokens? })
//     }

//     fn all_nft_info(
//         &self,
//         deps: Deps,
//         env: Env,
//         token_id: String,
//         include_expired: bool,
//     ) -> StdResult<AllNftInfoResponse<Extension>> {
//         let pk = convert_id_string_to_bytes(token_id);

//         let info = self.tokens.load(deps.storage, pk)?;

//         let mut extension = info.extension.unwrap();
//         let mut image = extension.image_data.unwrap();
//         image.insert_str(5usize, "xmlns='http://www.w3.org/2000/svg' ");
//         image = encode(image.clone());
//         image.insert_str(0usize, "data:image/svg+xml;base64,");
//         extension.image = Some(image);
//         extension.image_data = None;

//         Ok(AllNftInfoResponse {
//             access: OwnerOfResponse {
//                 owner: info.owner.to_string(),
//                 approvals: humanize_approvals(&env.block, info.approvals, include_expired),
//             },
//             info: NftInfoResponse {
//                 token_uri: info.token_uri,
//                 extension: Some(extension),
//             },
//         })
//     }
// }

// impl<'a> Cw721ExtendedContract<'a>
// {
//     /// operators returns all operators owner given access to
//     fn operators(
//         &self,
//         deps: Deps,
//         env: Env,
//         owner: String,
//         include_expired: bool,
//         start_after: Option<String>,
//         page: Option<u32>,
//         limit: Option<u32>,
//     ) -> StdResult<OperatorsResponse> {
//         let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
//         let skip = page.unwrap_or(DEFAULT_PAGE) as usize * limit;
//         let start_addr = maybe_addr(deps.api, start_after)?;
//         let start = start_addr.map(|addr| Bound::Exclusive(addr.to_string().as_bytes().to_vec()));

//         let owner_addr = deps.api.addr_validate(&owner)?;
//         let res: StdResult<Vec<_>> = self
//             .operators
//             .prefix(&owner_addr)
//             .range(deps.storage, start, None, Order::Ascending)
//             .filter(|r| {
//                 include_expired || r.is_err() || !r.as_ref().unwrap().1.is_expired(&env.block)
//             })
//             .skip(skip)
//             .take(limit)
//             .map(parse_approval)
//             .collect();
//         Ok(OperatorsResponse { operators: res? })
//     }

//     fn owner_tokens(
//         &self,
//         deps: Deps,
//         owner: String,
//         start_after: Option<String>,
//     ) -> StdResult<NumTokensResponse> {
//         let owner_addr = deps.api.addr_validate(&owner)?;
//         let start = start_after.map(Bound::exclusive);
//         let count: u64 = self
//             .tokens
//             .idx
//             .owner
//             .prefix(owner_addr)
//             .keys(deps.storage, start, None, Order::Ascending)
//             .count() as u64;

//         Ok(NumTokensResponse {
//             count,
//         })
//     }


//     fn tokens(
//         &self,
//         deps: Deps,
//         owner: String,
//         start_after: Option<String>,
//         page: Option<u32>,
//         limit: Option<u32>,
//     ) -> StdResult<TokensResponse> {
//         let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
//         let skip = page.unwrap_or(DEFAULT_PAGE) as usize * limit;

//         // let start = Some(Bound::exclusive_int(start_after.unwrap().parse::<u64>().unwrap()));
//         // let start = Some(Bound::exclusive_int(start_after.unwrap().parse::<u64>().unwrap().to_be_bytes().to_vec()));
//         // let start = Some(Bound(start_after.unwrap().parse::<u64>()));
//         let mut start: Option<Bound> = None;
//         if start_after.is_some() {
//             start = Some(Bound::Exclusive(start_after.unwrap().parse::<u64>().unwrap().to_be_bytes().to_vec()));
//         }
//         let owner_addr = deps.api.addr_validate(&owner)?;
//         let tokens: Vec<String> = self
//             .tokens
//             .idx
//             .owner
//             .prefix(owner_addr)
//             .keys(deps.storage, start, None, Order::Ascending)
//             .skip(skip)
//             .take(limit)
//             .map(|x| x.map(|y| u64::from_be_bytes(y.try_into().unwrap()).to_string()))
//             .collect::<StdResult<Vec<_>>>()?;

//         Ok(TokensResponse { tokens })
//     }

//     fn all_tokens(
//         &self,
//         deps: Deps,
//         start_after: Option<String>,
//         page: Option<u32>,
//         limit: Option<u32>,
//     ) -> StdResult<TokensResponse> {
//         let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
//         let skip = page.unwrap_or(DEFAULT_PAGE) as usize * limit;
//         // let start = start_after.map(Bound::exclusive);
//         let mut start: Option<Bound> = None;
//         if start_after.is_some() {
//             // let test = start_after.map(|x| x.parse::<u64>()).map(|x| x.unwrap().to_be_bytes().to_vec()).unwrap();
//             // let start = Some(Bound::ExclusiveRaw(test));
//             // let start = Some(Bound::ExclusiveRaw(start_after.unwrap().parse::<u64>().unwrap().to_be_bytes().to_vec()));
//             start = Some(Bound::Exclusive(start_after.unwrap().parse::<u64>().unwrap().to_be_bytes().to_vec()));
//         }

//         let tokens: StdResult<Vec<String>> = self
//             .tokens
//             .range(deps.storage, start, None, Order::Ascending)
//             .skip(skip)
//             .take(limit)
//             .map(|item| item.map(|(k, _)| u64::from_be_bytes(k.try_into().unwrap()).to_string()))
//             .collect();

//         Ok(TokensResponse { tokens: tokens? })
//     }

//     pub fn minter(&self, deps: Deps) -> StdResult<MinterResponse> {
//         let minter_addr = self.minter.load(deps.storage)?;
//         Ok(MinterResponse {
//             minter: minter_addr.to_string(),
//         })
//     }
//     pub fn staking_contract(&self, deps: Deps) -> StdResult<StakingContractResponse> {
//         let staking_addr = self.staking_contract.load(deps.storage)?;
//         Ok(StakingContractResponse {
//             contract: staking_addr.to_string(),
//         })
//     }

//     pub fn query(&self, deps: Deps, env: Env, msg: LunaPunkQueryMsg) -> StdResult<Binary> {
//         match msg {
//             LunaPunkQueryMsg::Minter {} => to_binary(&self.minter(deps)?),
//             LunaPunkQueryMsg::ContractInfo {} => to_binary(&self.contract_info(deps)?),
//             LunaPunkQueryMsg::NftInfo { token_id } => to_binary(&self.nft_info(deps, token_id)?),
//             LunaPunkQueryMsg::AllNftInfo {
//                 token_id,
//                 include_expired,
//             } => to_binary(&self.all_nft_info(
//                 deps,
//                 env,
//                 token_id,
//                 include_expired.unwrap_or(false),
//             )?),
//             LunaPunkQueryMsg::ApprovedForAll {
//                 owner,
//                 include_expired,
//                 start_after,
//                 limit,
//             } => to_binary(&self.approvals(
//                 deps,
//                 env,
//                 owner,
//                 include_expired.unwrap_or(false),
//             )?),
//             LunaPunkQueryMsg::NumTokens {} => to_binary(&self.num_tokens(deps)?),
//             LunaPunkQueryMsg::StakingContract {} => to_binary(&self.staking_contract(deps)?),
//             LunaPunkQueryMsg::Tokens {
//                 owner,
//                 start_after,
//                 skip,
//                 limit,
//             } => to_binary(&self.tokens(deps, owner, start_after, skip, limit)?),
//             LunaPunkQueryMsg::AllTokens { start_after, skip, limit } => {
//                 to_binary(&self.all_tokens(deps, start_after, skip, limit)?)
//             },
//             LunaPunkQueryMsg::OwnerTokens { owner, start_after } => {
//                 to_binary(&self.owner_tokens(deps, owner, start_after)?)
//             },
//             _ => Cw721ExtendedContract::default().query(deps, env, msg.into())
//         }
//     }
// }


fn calc_limit(request: Option<u32>) -> usize {
    request.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize
}
fn calc_skip(request: Option<u32>, limit: usize) -> usize {
    request.unwrap_or(0) as usize * limit
}

fn parse_approval(item: StdResult<(Addr, Expiration)>) -> StdResult<cw721::Approval> {
    item.and_then(|(k, expires)| {
        let spender = k.to_string();
        Ok(cw721::Approval { spender, expires })
    })
}

fn humanize_approvals(
    block: &BlockInfo,
    approvals: Vec<Approval>,
    include_expired: bool,
) -> Vec<cw721::Approval> {
    approvals
        .iter()
        .filter(|apr| include_expired || !apr.is_expired(block))
        .map(humanize_approval)
        .collect()
}

fn humanize_approval(approval: &Approval) -> cw721::Approval {
    cw721::Approval {
        spender: approval.spender.to_string(),
        expires: approval.expires,
    }
}

pub fn convert_id_string_to_bytes(token_id: String) -> Vec<u8> {
    print!("token_id:{:?}", token_id);
    token_id.parse::<u64>().unwrap().to_be_bytes().to_vec()
}
