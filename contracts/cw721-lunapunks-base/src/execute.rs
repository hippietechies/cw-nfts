#[cfg(not(feature = "library"))]
use cosmwasm_std::CosmosMsg;
use cw721_base::state::TokenInfo;
use cw721_base::{MintMsg, ContractError, InstantiateMsg};
use crate::msg::{StakingMsg, SuccessMsg};
use crate::query::convert_id_string_to_bytes;
use cosmwasm_std::Storage;

use cosmwasm_std::{WasmMsg, to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Empty};

use cw2::{set_contract_version, get_contract_version};
use cw721::{ContractInfoResponse, Cw721ReceiveMsg, Expiration};
use cw721_base::msg::ExecuteMsg as CW721ExecuteMsg;

use crate::msg::{LunaPunkExecuteMsg, MigrateMsg};
use crate::state::{Cw721ExtendedContract};
use cw721_metadata_onchain::{Extension, Metadata, Trait};

use base64::encode;
use std::convert::TryFrom;
use std::collections::HashMap;
use bech32::Bech32;
use seahash::hash;
use std::convert::TryInto;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw721-base-mint";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response<Empty>> {
    let contract = Cw721ExtendedContract::default();

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let info = ContractInfoResponse {
        name: msg.name.to_string(),
        symbol: msg.symbol,
    };
    contract.contract_info.save(deps.storage, &info)?;
    let owner = deps.api.addr_validate(_info.sender.as_str())?;
    contract.owner.save(deps.storage, &owner)?;
    let minter = deps.api.addr_validate(&msg.minter)?;
    contract.minter.save(deps.storage, &minter)?;

    // let staking_addr = deps.api.addr_validate(&"terra14ayqtv6ck5w0u8slpgjp7wkvve9j066aqzgksn".to_string())?;
    // contract.staking_contract.save(deps.storage, &staking_addr)?;

    // if msg.name == "LunaPunks" {
        let addresses: Vec<String> = vec![
            // "terra11zlxflh77eecxm7yxd4hdlj6c4tamazv9cykxr0".to_string(),
            // "terra11z7y0lw7a45qq0pscysnfvtpzlh4gutqqylgk7p".to_string(),
        ];

        let mut iter = addresses.iter();
        for index in 0..addresses.len() {
            let response = generate_image(iter.next().unwrap().to_string(), _env.block.time.seconds().to_string());
            let token = TokenInfo {
                owner: _info.sender.clone(),
                approvals: vec![],
                token_uri: None,
                extension: Some(Metadata {
                    description: Some("On Chain Luna Punks, only 1 single randomly generated Unique Luna Punk per Terra address!".to_string()),
                    name: Some(format!("LunaPunks: #{}", index.to_string())),
                    image_data: Some(response.0),
                    attributes: Some(response.1),
                    image: None,
                    external_url: Some("https://lunapunks.io/".to_string()),
                    background_color: None,
                    animation_url: None,
                    youtube_url: None,
                }),
            };
            let token_id = convert_id_string_to_bytes(index.to_string());

            contract.tokens
                .update(deps.storage, token_id, |old| match old {
                    Some(_) => Err(ContractError::Claimed {}),
                    None => Ok(token),
                }).ok();
            contract.increment_tokens(deps.storage)?;
        }
    // }
    Ok(Response::default())
}

pub fn mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MintMsg<Extension>,
) -> Result<Response<Empty>, ContractError> {
    let contract = Cw721ExtendedContract::default();

    let minter = contract.minter.load(deps.storage)?;

    if info.sender != minter {
        return Err(ContractError::Unauthorized {});
    }

    let count = contract.token_count(deps.storage)? + 1;
    let token_id = count.to_string();

    let address = generate_address(msg.owner.as_str(), _env.block.time.nanos().wrapping_add(count)).to_string().unwrap();
    // create the token
    let response = generate_image(address, _env.block.time.seconds().to_string());

    let mut image_date: String = "data:image/svg+xml;base64,".to_string();
    image_date.push_str(&encode(response.0));

    let token = TokenInfo {
        owner: deps.api.addr_validate(&msg.owner)?,
        approvals: vec![],
        token_uri: msg.token_uri,
        // bids: HashMap::new(),
        extension: Some(Metadata {
            description: Some("On Chain Luna Punks, only 1 single randomly generated Unique Luna Punk per Terra address!".to_string()),
            name: Some(format!("LunaPunks: #{}", token_id.clone())),
            image_data: Some(image_date),
            attributes: Some(response.1),
            image: None,
            external_url: Some("https://lunapunks.io/".to_string()),
            background_color: None,
            animation_url: None,
            youtube_url: None,
        }),
    };


    // let mut extension = token.extension.unwrap();
    // let mut image = extension.image_data.unwrap();
    // image.insert_str(5usize, "xmlns='http://www.w3.org/2000/svg' ");
    // image = encode(image.clone());
    // image.insert_str(0usize, "data:image/svg+xml;base64,");
    // extension.image = Some(image);
    // extension.image_data = None;
    // token.extension = Some(extension);
    let token_id_pk = convert_id_string_to_bytes(token_id.clone());

    contract.tokens
    .update(deps.storage, token_id_pk, |old| match old {
        Some(_) => Err(ContractError::Claimed {}),
        None => Ok(token),
    })?;

    contract.increment_tokens(deps.storage)?;
    // let staking_contract = &self.staking_contract.load(deps.storage)?;

    Ok(Response::new()
        // .add_message(self.get_revest_msg(staking_contract.to_string())?)
        .add_attribute("action", "mint")
        .add_attribute("minter", msg.owner)
        .add_attribute("token_id", token_id))
}


pub fn release(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    bids: Vec<Coin>,
) -> Result<Response, ContractError> {
    let contract = Cw721ExtendedContract::default();

    let owner = &contract.owner.load(deps.storage)?;

    if owner != &info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let resp = Response::new()
        .add_attribute("action", "release")
        .add_message(BankMsg::Send {
            to_address: owner.to_string(),
            amount: bids,
        });
    Ok(resp)
}

// impl<'a> Cw721ExtendedContract<'a>
// {

//     pub fn execute(
//         &self,
//         deps: DepsMut,
//         env: Env,
//         info: MessageInfo,
//         msg: LunaPunkExecuteMsg<Extension>,
//     ) -> Result<Response<Empty>, ContractError> {
//         match msg {
//             // LunaPunkExecuteMsg::Approve { spender, token_id, expires } => {
//             //     // self.approve(deps, env, info, spender, token_id, expires)
//             //     Cw721ExtendedContract::default().execute(deps, env, info, msg.into())
//             // },
//             LunaPunkExecuteMsg::Release { bids } => self.release(deps, env, info, bids),
//             LunaPunkExecuteMsg::Mint(msg) => self.mint(deps, env, info, msg),
//             LunaPunkExecuteMsg::TransferNft {
//                 recipient,
//                 token_id,
//             } => self.transfer_nft(deps, env, info, recipient, token_id),
//             LunaPunkExecuteMsg::SendNft {
//                 contract,
//                 token_id,
//                 msg,
//             } => self.send_nft(deps, env, info, contract, token_id, msg),
//             _ => {
//                 println!("hello janan");
//                 Cw721ExtendedContract::default().execute(deps, env, info, msg.into())
//             }

//             // ExecuteMsg::Burn { token_id } => self.burn(deps, env, info, token_id),
//         }
//     }

pub fn migrate(
    deps: DepsMut,
    _env: Env,
    _msg: MigrateMsg
) -> Result<Response, ContractError> {
    let version = get_contract_version(deps.storage)?;
    // if version.contract != CONTRACT_NAME {
    //     return Err(ContractError::Std() {
    //         previous_contract: version.contract,
    //     });
    // }

    // let staking_addr = deps.api.addr_validate(&"terra14ayqtv6ck5w0u8slpgjp7wkvve9j066aqzgksn".to_string())?;

    // self.staking_contract.save(deps.storage, &staking_addr)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}
// }

// impl<'a> Cw721ExtendedContract<'a>
// {


//     fn transfer_nft(
//         &self,
//         deps: DepsMut,
//         env: Env,
//         info: MessageInfo,
//         recipient: String,
//         token_id: String,
//     ) -> Result<Response<Empty>, ContractError> {
//         self._transfer_nft(deps, &env, &info, &recipient, &token_id)?;

//         Ok(Response::new()
//             .add_attribute("action", "transfer_nft")
//             .add_attribute("sender", info.sender)
//             .add_attribute("recipient", recipient)
//             .add_attribute("token_id", token_id))
//     }

//     fn send_nft(
//         &self,
//         deps: DepsMut,
//         env: Env,
//         info: MessageInfo,
//         contract: String,
//         token_id: String,
//         msg: Binary,
//     ) -> Result<Response<Empty>, ContractError> {
//         // Transfer token
//         // let staking_contract = &self.staking_contract.load(deps.storage)?;
//         self._transfer_nft(deps, &env, &info, &contract, &token_id)?;

//         let send = Cw721ReceiveMsg {
//             sender: info.sender.to_string(),
//             token_id: token_id.clone(),
//             msg,
//         };

//         let mut messages: Vec<CosmosMsg> = vec![];
//         messages.push(send.into_cosmos_msg(contract.clone())?);
//         // messages.push(self.get_revest_msg(staking_contract.to_string())?.into());
//         // Send message
//         Ok(Response::new()
//             .add_messages(messages)
//             .add_attribute("action", "send_nft")
//             .add_attribute("sender", info.sender)
//             .add_attribute("recipient", contract)
//             .add_attribute("token_id", token_id))
//     }

//     // fn approve(
//     //     &self,
//     //     deps: DepsMut,
//     //     env: Env,
//     //     info: MessageInfo,
//     //     spender: String,
//     //     token_id: String,
//     //     expires: Option<Expiration>,
//     // ) -> Result<Response<Empty>, ContractError> {
//     //     self._update_approvals(deps, &env, &info, &spender, &token_id, true, expires)?;

//     //     Ok(Response::new()
//     //         .add_attribute("action", "approve")
//     //         .add_attribute("sender", info.sender)
//     //         .add_attribute("spender", spender)
//     //         .add_attribute("token_id", token_id))
//     // }

//     fn revoke(
//         &self,
//         deps: DepsMut,
//         env: Env,
//         info: MessageInfo,
//         spender: String,
//         token_id: String,
//     ) -> Result<Response<Empty>, ContractError> {
//         self._update_approvals(deps, &env, &info, &spender, &token_id, false, None)?;

//         Ok(Response::new()
//             .add_attribute("action", "revoke")
//             .add_attribute("sender", info.sender)
//             .add_attribute("spender", spender)
//             .add_attribute("token_id", token_id))
//     }

//     // fn approve_all(
//     //     &self,
//     //     deps: DepsMut,
//     //     env: Env,
//     //     info: MessageInfo,
//     //     operator: String,
//     //     expires: Option<Expiration>,
//     // ) -> Result<Response<Empty>, ContractError> {
//     //     // reject expired data as invalid
//     //     let expires = expires.unwrap_or_default();
//     //     if expires.is_expired(&env.block) {
//     //         return Err(ContractError::Expired {});
//     //     }

//     //     // set the operator for us
//     //     let operator_addr = deps.api.addr_validate(&operator)?;
//     //     self.operators
//     //         .save(deps.storage, (&info.sender, &operator_addr), &expires)?;

//     //     Ok(Response::new()
//     //         .add_attribute("action", "approve_all")
//     //         .add_attribute("sender", info.sender)
//     //         .add_attribute("operator", operator))
//     // }

//     fn revoke_all(
//         &self,
//         deps: DepsMut,
//         _env: Env,
//         info: MessageInfo,
//         operator: String,
//     ) -> Result<Response<Empty>, ContractError> {
//         let operator_addr = deps.api.addr_validate(&operator)?;
//         self.operators
//             .remove(deps.storage, (&info.sender, &operator_addr));

//         Ok(Response::new()
//             .add_attribute("action", "revoke_all")
//             .add_attribute("sender", info.sender)
//             .add_attribute("operator", operator))
//     }

//     // fn burn(
//     //     &self,
//     //     deps: DepsMut,
//     //     env: Env,
//     //     info: MessageInfo,
//     //     token_id: String,
//     // ) -> Result<Response<Empty>, ContractError> {
//     //     let token = self.tokens.load(deps.storage, &token_id)?;
//     //     self.check_can_send(deps.as_ref(), &env, &info, &token)?;

//     //     self.tokens.remove(deps.storage, &token_id)?;
//     //     self.decrement_tokens(deps.storage)?;

//     //     Ok(Response::new()
//     //         .add_attribute("action", "burn")
//     //         .add_attribute("sender", info.sender)
//     //         .add_attribute("token_id", token_id))
//     // }
// }
// // helpers
// impl<'a> Cw721ExtendedContract<'a>
// {
//     pub fn _transfer_nft(
//         &self,
//         deps: DepsMut,
//         env: &Env,
//         info: &MessageInfo,
//         recipient: &str,
//         token_id: &str,
//     ) -> Result<TokenInfo<Extension>, ContractError> {
//         let token_id = convert_id_string_to_bytes(token_id.to_string());
//         let mut token = self.tokens.load(deps.storage, token_id.to_vec())?;
//         // ensure we have permissions
//         self.check_can_send(deps.as_ref(), env, info, &token)?;
//         // set owner and remove existing approvals
//         token.owner = deps.api.addr_validate(recipient)?;
//         token.approvals = vec![];
//         self.tokens.save(deps.storage, token_id, &token)?;
//         Ok(token)
//     }

//     #[allow(clippy::too_many_arguments)]
//     pub fn _update_approvals(
//         &self,
//         deps: DepsMut,
//         env: &Env,
//         info: &MessageInfo,
//         spender: &str,
//         token_id: &str,
//         // if add == false, remove. if add == true, remove then set with this expiration
//         add: bool,
//         expires: Option<Expiration>,
//     ) -> Result<TokenInfo<Extension>, ContractError> {
//         let token_id = convert_id_string_to_bytes(token_id.to_string());
//         let mut token = self.tokens.load(deps.storage, token_id.to_vec())?;
//         // ensure we have permissions
//         self.check_can_approve(deps.as_ref(), env, info, &token)?;

//         // update the approval list (remove any for the same spender before adding)
//         let spender_addr = deps.api.addr_validate(spender)?;
//         token.approvals = token
//             .approvals
//             .into_iter()
//             .filter(|apr| apr.spender != spender_addr)
//             .collect();

//         // only difference between approve and revoke
//         if add {
//             // reject expired data as invalid
//             let expires = expires.unwrap_or_default();
//             if expires.is_expired(&env.block) {
//                 return Err(ContractError::Expired {});
//             }
//             let approval = Approval {
//                 spender: spender_addr,
//                 expires,
//             };
//             token.approvals.push(approval);
//         }

//         self.tokens.save(deps.storage, token_id, &token)?;

//         Ok(token)
//     }

//     /// returns true iff the sender can execute approve or reject on the contract
//     pub fn check_can_approve(
//         &self,
//         deps: Deps,
//         env: &Env,
//         info: &MessageInfo,
//         token: &TokenInfo<Extension>,
//     ) -> Result<(), ContractError> {
//         // owner can approve
//         if token.owner == info.sender {
//             return Ok(());
//         }
//         // operator can approve
//         let op = self
//             .operators
//             .may_load(deps.storage, (&token.owner, &info.sender))?;
//         match op {
//             Some(ex) => {
//                 if ex.is_expired(&env.block) {
//                     Err(ContractError::Unauthorized {})
//                 } else {
//                     Ok(())
//                 }
//             }
//             None => Err(ContractError::Unauthorized {}),
//         }
//     }

//     /// returns true iff the sender can transfer ownership of the token
//     fn check_can_send(
//         &self,
//         deps: Deps,
//         env: &Env,
//         info: &MessageInfo,
//         token: &TokenInfo<Extension>,
//     ) -> Result<(), ContractError> {
//         // owner can send
//         if token.owner == info.sender {
//             return Ok(());
//         }

//         // any non-expired token approval can send
//         if token
//             .approvals
//             .iter()
//             .any(|apr| apr.spender == info.sender && !apr.is_expired(&env.block))
//         {
//             return Ok(());
//         }

//         // operator can send
//         let op = self
//             .operators
//             .may_load(deps.storage, (&token.owner, &info.sender))?;
//         match op {
//             Some(ex) => {
//                 if ex.is_expired(&env.block) {
//                     Err(ContractError::Unauthorized {})
//                 } else {
//                     Ok(())
//                 }
//             }
//             None => Err(ContractError::Unauthorized {}),
//         }
//     }

//     fn decrement_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
//         let val = self.token_count(storage)? - 1;
//         self.token_count.save(storage, &val)?;
//         Ok(val)
//     }

//     pub fn get_revest_msg(&self, staking_contract: String) -> StdResult<WasmMsg> {
//         let revest_msg = WasmMsg::Execute {
//             contract_addr: staking_contract,
//             msg: to_binary(&StakingMsg::Revest {})?,
//             funds: Vec::new(),
//         };
//         Ok(revest_msg)
//     }
// }

fn generate_image(
    sender: String,
    timestamp: String
) -> (String, Vec<Trait>) {
    let address = Bech32::from_string(sender).unwrap();
    let is_male = check_male(&address);
    let attribute_count: u8 = check_attribute_count(&address, &is_male);

    let mut ratio: Vec<u8> = get_ratio(is_male);
    let mut ratio_map: Vec<&str> = get_ratio_map(is_male);
    // let ratio_index: Vec<u8> = get_ratio_index(is_male);

    let mut compensate_counter: u8 = 0;
    let mut assets_map: HashMap<&str, &str> = HashMap::new();

    let a: u16 = address.data[4].into();
    let b: u16 = address.data[5].into();
    let c: u16 = a*32 + b;

    let mut traits: Vec<Trait> = vec![];
    let base_asset_name = get_asset_name(is_male, "Base", c);
    traits.push( Trait {
        display_type: Some("date".to_string()),
        trait_type: "Birthday".to_string(),
        value: timestamp,
    });
    traits.push( Trait {
        display_type: None,
        trait_type: "Gender".to_string(),
        value: if is_male { "Male".to_string() } else { "Female".to_string() }
    });
    assets_map.insert("Base", base_asset_name);
    for i in 0..attribute_count {
        let index = usize::from(i);
        let attribute_numeral: u8 = TryFrom::try_from(u16::from(address.data[6 + index]) * (31u16 - u16::from(compensate_counter)) / 31u16).unwrap();
        let attribute_index = get_random_attribute(attribute_numeral, &mut ratio);

        compensate_counter += ratio[attribute_index];
        ratio.remove(attribute_index);

        let a: u16 = address.data[30-(index*2)].into();
        let b: u16 = address.data[31-(index*2)].into();
        let c: u16 = a*32 + b;

        let selected_attribute = &ratio_map[attribute_index].clone();
        ratio_map.remove(attribute_index);
        assets_map.insert(selected_attribute, get_asset_name(is_male, selected_attribute, c));
    }

    // let mut svg: String = "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'>".to_string();
    // svg.push_str("<style> :root { --color-brand-green: rgb(255,111,111); --color-brand-fade: rgb(45, 85, 176); --color-brand-blue: rgb(45, 85, 176); }");
    // svg.push_str(".gradient__brand-green { stop-color: var(--color-brand-green); }");
    // svg.push_str(".gradient__brand-fade { stop-color: var(--color-brand-fade); }");
    // svg.push_str(".gradient__brand-blue { stop-color: var(--color-brand-blue); } </style>");
    // svg.push_str("<defs><linearGradient id='grad1' x1='0%' y1='0%' x2='100%' y2='0%'>");
    // svg.push_str("<stop offset='0' class='gradient__brand-blue' style='stop-opacity:1' />");

    // svg.push_str("<stop offset='0' class='gradient__brand-blue' style='stop-opacity:1' ><animate attributeName='offset' values='0;0.8;0' dur='39s' repeatCount='indefinite' /></stop>");
    // svg.push_str("<stop offset='0.2' class='gradient__brand-green' style='stop-opacity:0.9' ><animate attributeName='offset' values='0.2;1;0.2' dur='39s' repeatCount='indefinite' /></stop>");
    // svg.push_str("<stop offset='0.4' class='gradient__brand-blue' style='stop-opacity:1' ><animate attributeName='offset' values='0.4;1.2;0.4' dur='39s' repeatCount='indefinite' /></stop>");

    // // svg.push_str("<stop offset='0.2' class='gradient__brand-green' style='stop-opacity:1' ><animate attributeName='offset' values='0.2;0.4;0.2' dur='9s' repeatCount='indefinite' /></stop>");
    // // svg.push_str("<stop offset='63%' class='gradient__brand-fade' style='stop-opacity:1'><animate attributeName='offset' values='0.3;0.6;0.3' dur='9s' repeatCount='indefinite' /></stop>");
    // svg.push_str("<stop offset='1' class='gradient__brand-blue' style='stop-opacity:1'/>");
    // svg.push_str("</linearGradient></defs>");

    let mut svg: String = "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24'>".to_string();
    svg.push_str("<rect width='24' height='24' x='0' y='0' fill='rgba(244,32,39)' fill-opacity='1' />");
    svg.push_str("<rect width='24' height='22' x='0' y='1' fill='rgb(242,225,91)' fill-opacity='0.1' />");
    svg.push_str("<rect width='24' height='20' x='0' y='2' fill='rgb(242,225,91)' fill-opacity='0.1' />");
    svg.push_str("<rect width='24' height='18' x='0' y='3' fill='rgb(242,225,91)' fill-opacity='0.1' />");
    svg.push_str("<rect width='24' height='16' x='0' y='4' fill='rgb(242,225,91)' fill-opacity='0.1' />");
    svg.push_str("<rect width='24' height='14' x='0' y='5' fill='rgb(242,225,91)' fill-opacity='0.1' />");
    svg.push_str("<rect width='24' height='12' x='0' y='6' fill='rgb(242,225,91)' fill-opacity='0.1' />");
    svg.push_str("<rect width='24' height='10' x='0' y='7' fill='rgb(242,225,91)' fill-opacity='0.2' />");
    svg.push_str("<rect width='24' height='8' x='0' y='8' fill='rgb(242,225,91)' fill-opacity='0.2' />");
    svg.push_str("<rect width='24' height='6' x='0' y='9' fill='rgb(242,225,91)' fill-opacity='0.3' />");
    svg.push_str("<rect width='24' height='4' x='0' y='10' fill='rgb(242,225,91)' fill-opacity='0.3' />");
    svg.push_str("<rect width='24' height='2' x='0' y='11' fill='rgb(242,225,91)' fill-opacity='0.8' />");

    for asset_layering in get_attribute_layering() {
        let asset_name = assets_map.get(asset_layering);
        if asset_name != None {

            traits.push( Trait {
                display_type: None,
                trait_type: asset_layering.to_string(),
                value: asset_name.unwrap().to_string(),
            });
            let asset_pixels: Vec<(u8,u8,u8)> = get_asset_pixels(is_male, asset_name.unwrap());
            for val in asset_pixels.iter() {
                svg.push_str(
                    format!("<rect width='1' height='1' x='{}' y='{}' fill='rgb{:?}' fill-opacity='{}' />",
                    val.0.to_string(), val.1.to_string(),
                    get_color(val.2).0,
                    format!("{}",get_color(val.2).1)).as_str());
            }
        }
    }
    svg.push_str("</svg>");
    return (svg, traits);
}

fn get_asset_pixels(is_male:bool, asset_name: &str) -> Vec<(u8,u8,u8)> {
    if is_male {
        return get_male_asset(&asset_name);
    } else {
        return get_female_asset(&asset_name);
    }
}


fn get_color(color_index: u8) -> ((u8,u8,u8), &'static str) {
    if color_index == 0 { return ((0, 0, 0), "1"); }
    else if color_index == 1 { return ((255, 246, 142), "1"); }
    else if color_index == 2 { return ((174, 139, 97), "1"); }
    else if color_index == 3 { return ((113, 63, 29), "1"); }
    else if color_index == 4 { return ((234, 217, 217), "1"); }
    else if color_index == 5 { return ((200, 251, 251), "1"); }
    else if color_index == 6 { return ((125, 162, 105), "1"); }
    else if color_index == 7 { return ((219, 177, 128), "1"); }
    else if color_index == 8 { return ((166, 110, 44), "1"); }
    else if color_index == 9 { return ((40, 177, 67), "1"); }
    else if color_index == 10 { return ((226, 38, 38), "1"); }
    else if color_index == 11 { return ((201, 201, 201), "1"); }
    else if color_index == 12 { return ((255, 255, 255), "1"); }
    else if color_index == 13 { return ((133, 111, 86), "1"); }
    else if color_index == 14 { return ((129, 25, 183), "1"); }
    else if color_index == 15 { return ((53, 36, 16), "1"); }
    else if color_index == 16 { return ((81, 54, 12), "1"); }
    else if color_index == 17 { return ((85, 85, 85), "1"); }
    else if color_index == 18 { return ((113, 12, 199), "1"); }
    else if color_index == 19 { return ((230, 87, 0), "1"); }
    else if color_index == 20 { return ((121, 75, 17), "1"); }
    else if color_index == 21 { return ((255, 142, 190), "1"); }
    else if color_index == 22 { return ((61, 47, 30), "1"); }
    else if color_index == 23 { return ((240, 240, 240), "1"); }
    else if color_index == 24 { return ((26, 67, 200), "1"); }
    else if color_index == 25 { return ((34, 144, 0), "1"); }
    else if color_index == 26 { return ((22, 55, 164), "1"); }
    else if color_index == 27 { return ((45, 107, 98), "1"); }
    else if color_index == 28 { return ((202, 78, 17), "1"); }
    else if color_index == 29 { return ((180, 180, 180), "1"); }
    else if color_index == 30 { return ((128, 219, 218), "1"); }
    else if color_index == 31 { return ((153, 124, 89), "1"); }
    else if color_index == 32 { return ((76, 76, 76), "1"); }
    else if color_index == 33 { return ((147, 55, 9), "1"); }
    else if color_index == 34 { return ((38, 49, 74), "1"); }
    else if color_index == 35 { return ((94, 76, 55), "1"); }
    else if color_index == 36 { return ((152, 136, 128), "1"); }
    else if color_index == 37 { return ((133, 81, 20), "1"); }
    else if color_index == 38 { return ((81, 81, 81), "1"); }
    else if color_index == 39 { return ((104, 70, 31), "1"); }
    else if color_index == 40 { return ((228, 235, 23), "1"); }
    else if color_index == 41 { return ((185, 185, 185), "0.5"); }
    else if color_index == 42 { return ((0, 85, 128), "1"); }
    else if color_index == 43 { return ((26, 110, 213), "1"); }
    else if color_index == 44 { return ((28, 26, 0), "1"); }
    else if color_index == 45 { return ((83, 76, 0), "1"); }
    else if color_index == 46 { return ((134, 88, 30), "1"); }
    else if color_index == 47 { return ((0, 96, 195), "1"); }
    else if color_index == 48 { return ((94, 114, 83), "1"); }
    else if color_index == 49 { return ((40, 88, 177), "1"); }
    else if color_index == 50 { return ((44, 81, 149), "1"); }
    else if color_index == 51 { return ((44, 149, 65), "1"); }
    else if color_index == 52 { return ((50, 141, 253), "1"); }
    else if color_index == 53 { return ((253, 50, 50), "1"); }
    else if color_index == 54 { return ((105, 12, 69), "1"); }
    else if color_index == 55 { return ((140, 13, 91), "1"); }
    else if color_index == 56 { return ((173, 33, 96), "1"); }
    else if color_index == 57 { return ((141, 141, 141), "1"); }
    else if color_index == 58 { return ((177, 177, 177), "1"); }
    else if color_index == 59 { return ((221, 221, 221), "0.5"); }
    else if color_index == 60 { return ((80, 47, 5), "1"); }
    else if color_index == 61 { return ((220, 29, 29), "1"); }
    else if color_index == 62 { return ((214, 4, 4), "1"); }
    else if color_index == 63 { return ((53, 53, 53), "1"); }
    else if color_index == 64 { return ((198, 198, 198), "1"); }
    else if color_index == 65 { return ((89, 89, 89), "1"); }
    else if color_index == 66 { return ((155, 224, 224), "1"); }
    else if color_index == 67 { return ((255, 186, 0), "1"); }
    else if color_index == 68 { return ((164, 133, 96), "1"); }
    else if color_index == 69 { return ((92, 57, 15), "1"); }
    else if color_index == 70 { return ((199, 117, 20), "1"); }
    else if color_index == 71 { return ((167, 124, 71), "1"); }
    else if color_index == 72 { return ((137, 122, 112), "1"); }
    else if color_index == 73 { return ((214, 0, 0), "1"); }
    else if color_index == 74 { return ((165, 141, 141), "1"); }
    else if color_index == 75 { return ((217, 175, 127), "1"); }
    else if color_index == 76 { return ((106, 86, 63), "1"); }
    else if color_index == 77 { return ((60, 195, 0), "1"); }
    else if color_index == 78 { return ((173, 126, 89), "1"); }
    else if color_index == 79 { return ((41, 62, 100), "1"); }
    else if color_index == 80 { return ((41, 100, 52), "1"); }
    else if color_index == 81 { return ((231, 203, 169), "1"); }
    else if color_index == 82 { return ((20, 44, 124), "1"); }
    else if color_index == 83 { return ((104, 60, 8), "1"); }
    else if color_index == 84 { return ((255, 201, 38), "1"); }
    else if color_index == 85 { return ((223, 223, 223), "1"); }
    else if color_index == 86 { return ((1, 0, 0), "1"); }
    else if color_index == 87 { return ((94, 87, 87), "1"); }
    else if color_index == 88 { return ((255, 142, 102), "0.75"); }
    else if color_index == 89 { return ((255, 217, 38), "1"); }
    else if color_index == 90 { return ((215, 215, 215), "1"); }
    else if color_index == 91 { return ((89, 101, 112), "1"); }
    else if color_index == 92 { return ((153, 132, 117), "1"); }
    else if color_index == 93 { return ((178, 97, 220), "1"); }
    else if color_index == 94 { return ((42, 42, 42), "1"); }
    else if color_index == 95 { return ((133, 86, 30), "1"); }
    else if color_index == 96 { return ((201, 178, 178), "1"); }
    else if color_index == 97 { return ((210, 157, 96), "1"); }
    else if color_index == 98 { return ((135, 89, 36), "1"); }
    else if color_index == 99 { return ((88, 43, 16), "1"); }
    else if color_index == 100 { return ((0, 0, 0), "1"); }
    else if color_index == 101 { return ((236, 255, 255), "1"); }
    else if color_index == 102 { return ((117, 189, 189), "1"); }
    else if color_index == 103 { return ((78, 59, 38), "1"); }
    else if color_index == 104 { return ((169, 140, 107), "1"); }
    else if color_index == 105 { return ((155, 188, 136), "1"); }
    else if color_index == 106 { return ((255, 0, 0), "1"); }
    else if color_index == 107 { return ((60, 86, 89), "1"); }
    else if color_index == 108 { return ((60, 104, 39), "1"); }
    else if color_index == 109 { return ((143, 15, 105), "1"); }
    else if color_index == 110 { return ((170, 123, 84), "0.75"); }
    else if color_index == 111 { return ((86, 38, 0), "1"); }
    else if color_index == 112 { return ((105, 47, 8), "1"); }
    else if color_index == 113 { return ((40, 27, 9), "1"); }
    else if color_index == 114 { return ((233, 216, 215), "1"); }
    else if color_index == 115 { return ((139, 83, 44), "1"); }
    else if color_index == 116 { return ((196, 33, 16), "1"); }
    else if color_index == 117 { return ((205, 0, 203), "1"); }
    else if color_index == 118 { return ((194, 137, 70), "1"); }
    else if color_index == 119 { return ((99, 99, 99), "1"); }
    else if color_index == 120 { return ((100, 88, 73), "1"); }
    else if color_index == 121 { return ((226, 91, 38), "1"); }
    else if color_index == 122 { return ((0, 64, 255), "1"); }
    else if color_index == 123 { return ((219, 177, 130), "1"); }
    else if color_index == 124 { return ((184, 162, 134), "1"); }
    else if color_index == 125 { return ((114, 55, 10), "1"); }
    else if color_index == 126 { return ((72, 93, 93), "1"); }
    else if color_index == 127 { return ((72, 111, 43), "1"); }
    else if color_index == 128 { return ((155, 22, 109), "1"); }
    else if color_index == 129 { return ((114, 55, 9), "1"); }
    else if color_index == 130 { return ((154, 142, 139), "1"); }
    else if color_index == 131 { return ((255, 216, 0), "1"); }
    else if color_index == 132 { return ((6, 6, 6), "1"); }
    else if color_index == 133 { return ((1, 1, 1), "1"); }
    else if color_index == 134 { return ((118, 95, 67), "1"); }
    else if color_index == 135 { return ((255, 42, 0), "1"); }
    else if color_index == 136 { return ((182, 159, 130), "1"); }
    else if color_index == 137 { return ((185, 185, 185), "0.25"); }
    else if color_index == 138 { return ((214,251,200),"1");}
    else if color_index == 139 { return ((152,187,139),"1");}
    else if color_index == 140 { return ((191,227,177),"1");}
    else if color_index == 141 { return ((242,254,238),"1");}
    else if color_index == 142 { return ((250,205,218),"1");}
    else if color_index == 143 { return ((175,134,146),"1");}
    else if color_index == 144 { return ((215,172,185),"1");}
    else if color_index == 145 { return ((254,243,246),"1");}
    else if color_index == 146 { return ((243,244,200),"1");}
    else if color_index == 147 { return ((159,160,120),"1");}
    else if color_index == 148 { return ((196,196,154),"1");}
    else if color_index == 149 { return ((254,254,251),"1");}
    else if color_index == 150 { return ((240,240,240),"1");}
    else if color_index == 151 { return ((158,158,158),"1");}
    else if color_index == 152 { return ((204,204,204),"1");}
    else if color_index == 153 { return ((247,247,247),"1");}
    else if color_index == 154 { return ((164,164,164),"1");}
    else if color_index == 155 { return ((100,100,100),"1");}
    else if color_index == 156 { return ((129,129,129),"1");}
    else if color_index == 157 { return ((202,202,202),"1");}
    else if color_index == 158 { return ((65,101,184),"1");}
    else if color_index == 159 { return ((12,68,146),"1");}
    else if color_index == 160 { return ((39,82,162),"1");}
    else if color_index == 161 { return ((215,220,236),"1");}
    else if color_index == 162 { return ((106,135,199),"1");}
    else if color_index == 163 { return ((66,97,158),"1");}
    else if color_index == 164 { return ((92,122,185),"1");}
    else if color_index == 165 { return ((215,220,236),"1");}
    else if color_index == 166 { return ((142,164,213),"1");}
    else if color_index == 167 { return ((102,125,172),"1");}
    else if color_index == 168 { return ((124,146,193),"1");}
    else if color_index == 169 { return ((215,220,236),"1");}
    else if color_index == 170 { return ((183,197,228),"1");}
    else if color_index == 171 { return ((143,156,186),"1");}
    else if color_index == 172 { return ((164,178,208),"1");}
    else if color_index == 173 { return ((215,220,236),"1");}
    else if color_index == 174 { return ((224,230,244),"1");}
    else if color_index == 175 { return ((185,191,204),"1");}
    else if color_index == 176 { return ((210,216,230),"1");}
    else if color_index == 177 { return ((236,240,248),"1");}
    else if color_index == 178 { return ((255,242,242),"1");} // white dot for red alien
    else if color_index == 179 { return ((250, 205, 218),"1");}
    else if color_index == 180 { return ((196, 161, 171),"1");}
    else if color_index == 181 { return ((153, 125, 133),"1");}
    else if color_index == 182 { return ((250, 180, 200),"1");}
    else if color_index == 183 { return ((196, 141, 157),"1");}
    else if color_index == 184 { return ((153, 110, 122),"1");}
    else if color_index == 185 { return ((250, 155, 182),"1");}
    else if color_index == 186 { return ((196, 121, 142),"1");}
    else if color_index == 187 { return ((153, 94, 110),"1");}
    else if color_index == 188 { return ((250, 130, 166),"1");}
    else if color_index == 189 { return ((196, 101, 129),"1");}
    else if color_index == 190 { return ((153, 78, 100),"1");}
    else if color_index == 191 { return ((225, 98, 140),"1");}
    else if color_index == 192 { return ((176, 75, 109),"1");}
    else if color_index == 193 { return ((137, 57, 84),"1");}
    else { return ((255,255,255),"1"); }
}

fn get_ratio(is_male: bool) -> Vec<u8> {
    if is_male {
        return vec![8,5,5,3,3,2,2,2,1,1];
    }
    return vec![8,8,4,4,2,2,2,1,1];
}

fn get_ratio_map(is_male: bool) -> Vec<&'static str> {
    if is_male {
        return vec!["Head","Beard","Glasses","Mouth Accessory","Mouth","Face","Eyes","Neck","Ear","Nose"];
    }
    return vec!["Head","Glasses","Eyes","Mouth Accessory","Neck","Mouth","Face","Ear","Nose"];
}

fn get_attribute_layering() -> Vec<&'static str> {
    return vec!["Base","Face","Head","Ear","Eyes","Nose","Neck","Beard","Mouth","Mouth Accessory","Glasses"];
}

fn get_random_attribute(attribute_numeral: u8, ratio: &mut Vec<u8>) -> usize {
    let mut counter: u8 = 0;
    println!("attribute_numeral: {}", attribute_numeral.to_string());

    for i in 0..ratio.len() {
        counter += ratio[i];
        println!("counter: {}", counter.to_string());
        if attribute_numeral < counter {
            return i;
        }
    }
    return ratio.len() - 1;
}

fn check_attribute_count(address: &Bech32, is_male: &bool) -> u8 {
    let a: u32 = address.data[1].into();
    let b: u32 = address.data[2].into();
    let c: u32 = address.data[3].into();
    let d: u32 = a*32*32 + b*32 + c;

    if *is_male {
        if d < 16384 {
            return 3;
        } else if d < 24576 {
            return 2;
        } else if d < 28672 {
            return 4;
        } else if d < 30720 {
            return 1;
        } else if d < 31744 {
            return 5;
        } else if d < 32256 {
            return 6;
        } else if d < 32512 {
            return 7;
        }else if d < 32640 {
            return 8;
        } else if d < 32734 {
            return 9;
        } else if d < 32760 {
            return 0;
        } else {
            return 10;
        }
    } else {
        if d < 16384 {
            return 3;
        } else if d < 24576 {
            return 2;
        } else if d < 28672 {
            return 4;
        } else if d < 30720 {
            return 1;
        } else if d < 31744 {
            return 5;
        } else if d < 32356 {
            return 6;
        } else if d < 32612 {
            return 7;
        } else if d < 32730 {
            return 8;
        } else if d < 32760 {
            return 0;
        } else {
            return 9;
        }
    }
}

fn check_male(address: &Bech32) -> bool {
    let byte_numeral: u8 = address.data[0].into();
    return byte_numeral % 3 != 0;
}

fn get_asset_name(
    is_male: bool,
    asset_type: &str,
    asset_numeral: u16
) -> &'static str {
    if is_male {
        match asset_type {
            "Base" => {
                if asset_numeral < 92 { return "Green Alien"; }
                else if asset_numeral < 184 { return "Yellow Alien"; }
                else if asset_numeral < 276 { return "White Alien"; }
                else if asset_numeral < 368 { return "Black Alien"; }
                else if asset_numeral < 458 { return "Red 0 Alien"; }
                else if asset_numeral < 548 { return "Red 1 Alien"; }
                else if asset_numeral < 638 { return "Red 2 Alien"; }
                else if asset_numeral < 728 { return "Red 3 Alien"; }
                else if asset_numeral < 819 { return "Red 4 Alien"; }
                else if asset_numeral < 844 { return "Blue 0 Alien"; }
                else if asset_numeral < 869 { return "Blue 1 Alien"; }
                else if asset_numeral < 894 { return "Blue 2 Alien"; }
                else if asset_numeral < 919 { return "Blue 3 Alien"; }
                else if asset_numeral < 944 { return "Blue 4 Alien"; }
                else if asset_numeral < 952 { return "0"; }
                else if asset_numeral < 960 { return "1"; }
                else if asset_numeral < 968 { return "2"; }
                else if asset_numeral < 976 { return "3"; }
                else if asset_numeral < 1008 { return "Ape"; }
                else if asset_numeral < 1023 { return "Zombie"; }
                else { return "Alien"; }
            },
            "Head" => {
                if asset_numeral < 64 { return "Cap Forward"; }
                else if asset_numeral < 128 { return "Cowboy Hat"; }
                else if asset_numeral < 192 { return "Frumpy Hair"; }
                else if asset_numeral < 256 { return "Mohawk Dark"; }
                else if asset_numeral < 320 { return "Mohawk Thin"; }
                else if asset_numeral < 384 { return "Mohawk"; }
                else if asset_numeral < 448 { return "Peak Spike"; }
                else if asset_numeral < 512 { return "Police Cap"; }
                else if asset_numeral < 576 { return "Shaved Head"; }
                else if asset_numeral < 640 { return "Vampire Hair"; }
                else if asset_numeral < 672 { return "Stringy Hair"; }
                else if asset_numeral < 704 { return "Top Hat"; }
                else if asset_numeral < 736 { return "Wild Hair"; }
                else if asset_numeral < 768 { return "Bandana"; }
                else if asset_numeral < 800 { return "Beanie"; }
                else if asset_numeral < 832 { return "Cap"; }
                else if asset_numeral < 864 { return "Crazy Hair"; }
                else if asset_numeral < 896 { return "Do-rag"; }
                else if asset_numeral < 928 { return "Fedora"; }
                else if asset_numeral < 960 { return "Hoodie"; }
                else if asset_numeral < 992 { return "Messy Hair"; }
                else if asset_numeral < 1000 { return "Knitted Cap"; }
                else if asset_numeral < 1008 { return "Headband"; }
                else if asset_numeral < 1016 { return "Purple Hair"; }
                else { return "Clown Hair Green"; }
            },
            "Face" => {
                if asset_numeral < 512 { return "Rosy Cheeks"; }
                else if asset_numeral < 896 { return "Mole"; }
                else { return "Spots"; }
            },
            "Beard" => {
                if asset_numeral < 128 { return "Big Beard"; }
                else if asset_numeral < 258 { return "Handlebars"; }
                else if asset_numeral < 386 { return "Normal Beard Black"; }
                else if asset_numeral < 512 { return "Luxurious Beard"; }
                else if asset_numeral < 576 { return "Chinstrap"; }
                else if asset_numeral < 640 { return "Front Beard Dark"; }
                else if asset_numeral < 704 { return "Front Beard"; }
                else if asset_numeral < 768 { return "GoatU"; }
                else if asset_numeral < 832 { return "Mustache"; }
                else if asset_numeral < 896 { return "Muttonchops"; }
                else if asset_numeral < 960 { return "Normal Beard"; }
                else { return "Shadow Beard"; }
            },
            "Mouth" => {
                if asset_numeral < 512 { return "Frown"; }
                else if asset_numeral < 768 { return "Buck Teeth"; }
                else { return "Smile"; }
            },
            "Mouth Accessory" => {
                if asset_numeral < 384 { return "Vape"; }
                else if asset_numeral < 640 { return "Cigarette"; }
                else if asset_numeral < 896 { return "Pipe"; }
                else { return "Medical Mask"; }
            },
            "Glasses" => {
                if asset_numeral < 128 { return "3D Glasses"; }
                else if asset_numeral < 256 { return "Regular Shades"; }
                else if asset_numeral < 384 { return "VR"; }
                else if asset_numeral < 512 { return "Classic Shades"; }
                else if asset_numeral < 768 { return "Eye Mask"; }
                else if asset_numeral < 832 { return "Eye Patch"; }
                else if asset_numeral < 896 { return "Horned Rim Glasses"; }
                else if asset_numeral < 960 { return "Nerd Glasses"; }
                else if asset_numeral < 992 { return "Big Shades"; }
                else { return "Small Shades"; }
            },
            "Eyes" => {
                if asset_numeral < 512 { return "Clown Eyes Blue"; }
                else { return "Clown Eyes Green"; }
            },
            "Neck" => {
                if asset_numeral < 768 { return "Silver Chain"; }
                else { return "Gold Chain"; }
            },
            "Ear" => { return "Earring"; },
            "Nose" => { return "Clown Nose"; },
            _ => return ""
        }
    } else {
        match asset_type {
            "Base" => {
                if asset_numeral < 92 { return "Green Alien"; }
                else if asset_numeral < 184 { return "Yellow Alien"; }
                else if asset_numeral < 276 { return "White Alien"; }
                else if asset_numeral < 368 { return "Black Alien"; }
                else if asset_numeral < 458 { return "Red 0 Alien"; }
                else if asset_numeral < 548 { return "Red 1 Alien"; }
                else if asset_numeral < 638 { return "Red 2 Alien"; }
                else if asset_numeral < 728 { return "Red 3 Alien"; }
                else if asset_numeral < 819 { return "Red 4 Alien"; }
                else if asset_numeral < 844 { return "Blue 0 Alien"; }
                else if asset_numeral < 869 { return "Blue 1 Alien"; }
                else if asset_numeral < 894 { return "Blue 2 Alien"; }
                else if asset_numeral < 919 { return "Blue 3 Alien"; }
                else if asset_numeral < 944 { return "Blue 4 Alien"; }
                else if asset_numeral < 952 { return "0"; }
                else if asset_numeral < 960 { return "1"; }
                else if asset_numeral < 968 { return "2"; }
                else if asset_numeral < 976 { return "3"; }
                else if asset_numeral < 1008 { return "Ape"; }
                else if asset_numeral < 1023 { return "Zombie"; }
                else { return "Alien"; }
            },
            "Head" => {
                if asset_numeral < 64 { return "Blonde Bob"; }
                else if asset_numeral < 128 { return "Blonde Short"; }
                else if asset_numeral < 192 { return "Dark Hair"; }
                else if asset_numeral < 256 { return "Frumpy Hair"; }
                else if asset_numeral < 320 { return "Mohawk"; }
                else if asset_numeral < 384 { return "Mohawk Dark"; }
                else if asset_numeral < 448 { return "Mohawk Thin"; }
                else if asset_numeral < 512 { return "Orange Side"; }
                else if asset_numeral < 576 { return "Straight Hair Blonde"; }
                else if asset_numeral < 640 { return "Straight Hair Dark"; }
                else if asset_numeral < 704 { return "Straight Hair"; }
                else if asset_numeral < 736 { return "Half Shaved"; }
                else if asset_numeral < 768 { return "Bandana"; }
                else if asset_numeral < 800 { return "Cap"; }
                else if asset_numeral < 832 { return "Messy Hair"; }
                else if asset_numeral < 864 { return "Wild Blonde"; }
                else if asset_numeral < 896 { return "Wild Hair"; }
                else if asset_numeral < 912 { return "Crazy Hair"; }
                else if asset_numeral < 928 { return "Pigtails"; }
                else if asset_numeral < 944 { return "Pink With Hat"; }
                else if asset_numeral < 960 { return "Tassle Hat"; }
                else if asset_numeral < 968 { return "Clown Hair Green"; }
                else if asset_numeral < 976 { return "Headband"; }
                else if asset_numeral < 984 { return "Knitted Cap"; }
                else if asset_numeral < 992 { return "Pilot Helmet"; }
                else if asset_numeral < 1000 { return "Red Mohawk"; }
                else if asset_numeral < 1008 { return "Stringy Hair"; }
                else if asset_numeral < 1016 { return "Tiara"; }
                else { return "Wild White Hair"; }
            },
            "Face" => {
                if asset_numeral < 512 { return "Rosy Cheeks"; }
                else if asset_numeral < 896 { return "Mole"; }
                else { return "Spots"; }
            },
            "Mouth" => {
                if asset_numeral < 512 { return "Hot Lipstick"; }
                else if asset_numeral < 768 { return "Black Lipstick"; }
                else  { return "Purple Lipstick"; }
            },
            "Mouth Accessory" => {
                if asset_numeral < 384 { return "Vape"; }
                else if asset_numeral < 640 { return "Cigarette"; }
                else if asset_numeral < 896 { return "Pipe"; }
                else { return "Medical Mask"; }
            },
            "Glasses" => {
                if asset_numeral < 128 { return "3D Glasses"; }
                else if asset_numeral < 256 { return "Classic Shades"; }
                else if asset_numeral < 384 { return "Regular Shades"; }
                else if asset_numeral < 512 { return "VR"; }
                else if asset_numeral < 768 { return "Eye Mask"; }
                else if asset_numeral < 832 { return "Eye Patch"; }
                else if asset_numeral < 896 { return "Horned Rim Glasses"; }
                else if asset_numeral < 960 { return "Nerd Glasses"; }
                else if asset_numeral < 992 { return "Big Shades"; }
                else { return "Welding Goggles"; }
            },
            "Eyes" => {
                if asset_numeral < 384 { return "Blue Eye Shadow"; }
                else if asset_numeral < 768 { return "Green Eye Shadow"; }
                else if asset_numeral < 896 { return "Purple Eye Shadow"; }
                else if asset_numeral < 960 { return "Clown Eyes Blue"; }
                else { return "Clown Eyes Green"; }
            },
            "Neck" => {
                if asset_numeral < 512 { return "Choker"; }
                else if asset_numeral < 896 { return "Silver Chain"; }
                else { return "Gold Chain"; }
            },
            "Ear" => { return "Earring"; },
            "Nose" => { return "Clown Nose"; },
            _ => return ""
        }
    }
}

fn build_alien_asset(color_map: Vec<u8>) -> Vec<(u8,u8,u8)> {
    return vec![(8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (7, 6, 0), (8, 6, color_map[0]), (9, 6, color_map[0]), (10, 6, color_map[0]), (11, 6, color_map[0]), (12, 6, color_map[0]), (13, 6, color_map[0]), (14, 6, color_map[0]), (15, 6, 0), (6, 7, 0), (7, 7, color_map[0]), (8, 7, color_map[0]), (9, 7, color_map[3]), (10, 7, color_map[0]), (11, 7, color_map[0]), (12, 7, color_map[0]), (13, 7, color_map[0]), (14, 7, color_map[0]), (15, 7, color_map[0]), (16, 7, 0), (6, 8, 0), (7, 8, color_map[0]), (8, 8, color_map[3]), (9, 8, color_map[0]), (10, 8, color_map[0]), (11, 8, color_map[0]), (12, 8, color_map[0]), (13, 8, color_map[0]), (14, 8, color_map[0]), (15, 8, color_map[0]), (16, 8, 0), (6, 9, 0), (7, 9, color_map[0]), (8, 9, color_map[0]), (9, 9, color_map[0]), (10, 9, color_map[0]), (11, 9, color_map[0]), (12, 9, color_map[0]), (13, 9, color_map[0]), (14, 9, color_map[0]), (15, 9, color_map[0]), (16, 9, 0), (6, 10, 0), (7, 10, color_map[0]), (8, 10, color_map[0]), (9, 10, color_map[0]), (10, 10, color_map[0]), (11, 10, color_map[0]), (12, 10, color_map[0]), (13, 10, color_map[0]), (14, 10, color_map[0]), (15, 10, color_map[0]), (16, 10, 0), (5, 11, 0), (6, 11, 0), (7, 11, color_map[0]), (8, 11, color_map[0]), (9, 11, color_map[1]), (10, 11, 0), (11, 11, color_map[0]), (12, 11, color_map[0]), (13, 11, color_map[0]), (14, 11, color_map[1]), (15, 11, 0), (16, 11, 0), (4, 12, 0), (5, 12, color_map[0]), (6, 12, color_map[2]), (7, 12, color_map[0]), (8, 12, color_map[0]), (9, 12, 0), (10, 12, color_map[2]), (11, 12, color_map[0]), (12, 12, color_map[0]), (13, 12, color_map[0]), (14, 12, 0), (15, 12, color_map[2]), (16, 12, 0), (5, 13, 0), (6, 13, color_map[0]), (7, 13, color_map[0]), (8, 13, color_map[0]), (9, 13, color_map[0]), (10, 13, color_map[0]), (11, 13, color_map[0]), (12, 13, color_map[0]), (13, 13, color_map[0]), (14, 13, color_map[0]), (15, 13, color_map[0]), (16, 13, 0), (5, 14, 0), (6, 14, 0), (7, 14, color_map[0]), (8, 14, color_map[0]), (9, 14, color_map[0]), (10, 14, color_map[0]), (11, 14, color_map[0]), (12, 14, color_map[2]), (13, 14, color_map[0]), (14, 14, color_map[0]), (15, 14, color_map[0]), (16, 14, 0), (6, 15, 0), (7, 15, color_map[0]), (8, 15, color_map[0]), (9, 15, color_map[0]), (10, 15, color_map[0]), (11, 15, color_map[0]), (12, 15, color_map[2]), (13, 15, color_map[0]), (14, 15, color_map[0]), (15, 15, color_map[0]), (16, 15, 0), (6, 16, 0), (7, 16, color_map[0]), (8, 16, color_map[0]), (9, 16, color_map[0]), (10, 16, color_map[0]), (11, 16, color_map[0]), (12, 16, color_map[2]), (13, 16, color_map[0]), (14, 16, color_map[0]), (15, 16, color_map[0]), (16, 16, 0), (6, 17, 0), (7, 17, color_map[0]), (8, 17, color_map[0]), (9, 17, color_map[0]), (10, 17, color_map[0]), (11, 17, color_map[0]), (12, 17, color_map[0]), (13, 17, color_map[0]), (14, 17, color_map[0]), (15, 17, color_map[0]), (16, 17, 0), (6, 18, 0), (7, 18, color_map[0]), (8, 18, color_map[0]), (9, 18, color_map[0]), (10, 18, 0), (11, 18, 0), (12, 18, 0), (13, 18, 0), (14, 18, 0), (15, 18, color_map[0]), (16, 18, 0), (6, 19, 0), (7, 19, color_map[0]), (8, 19, color_map[0]), (9, 19, color_map[0]), (10, 19, color_map[0]), (11, 19, color_map[0]), (12, 19, color_map[0]), (13, 19, color_map[0]), (14, 19, color_map[0]), (15, 19, color_map[0]), (16, 19, 0), (6, 20, 0), (7, 20, color_map[0]), (8, 20, color_map[0]), (9, 20, color_map[0]), (10, 20, color_map[0]), (11, 20, color_map[0]), (12, 20, color_map[0]), (13, 20, color_map[0]), (14, 20, color_map[0]), (15, 20, 0), (6, 21, 0), (7, 21, color_map[0]), (8, 21, color_map[0]), (9, 21, color_map[0]), (10, 21, 0), (11, 21, 0), (12, 21, 0), (13, 21, 0), (14, 21, 0), (6, 22, 0), (7, 22, color_map[0]), (8, 22, color_map[0]), (9, 22, color_map[0]), (10, 22, 0), (6, 23, 0), (7, 23, color_map[0]), (8, 23, color_map[0]), (9, 23, color_map[0]), (10, 23, 0)];
}

fn build_female_alien_asset(color_map: Vec<u8>) -> Vec<(u8,u8,u8)> {
    return vec![(9, 7, 0),(10, 7, 0),(11, 7, 0),(12, 7, 0),(13, 7, 0),(14, 7, 0),(8, 8, 0),(9, 8, color_map[0]),(10, 8, color_map[0]),(11, 8, color_map[0]),(12, 8, color_map[0]),(13, 8, color_map[0]),(14, 8, color_map[0]),(15, 8, 0),(7, 9, 0),(8, 9, color_map[0]),(9, 9, color_map[0]),(10, 9, color_map[3]),(11, 9, color_map[0]),(12, 9, color_map[0]),(13, 9, color_map[0]),(14, 9, color_map[0]),(15, 9, color_map[0]),(16, 9, 0),(7, 10, 0),(8, 10, color_map[0]),(9, 10, color_map[3]),(10, 10, color_map[0]),(11, 10, color_map[0]),(12, 10, color_map[0]),(13, 10, color_map[0]),(14, 10, color_map[0]),(15, 10, color_map[0]),(16, 10, 0),(7, 11, 0),(8, 11, color_map[0]),(9, 11, color_map[0]),(10, 11, color_map[0]),(11, 11, color_map[0]),(12, 11, color_map[0]),(13, 11, color_map[0]),(14, 11, color_map[0]),(15, 11, color_map[0]),(16, 11, 0),(6, 12, 0),(7, 12, color_map[1]),(8, 12, color_map[0]),(9, 12, color_map[2]),(10, 12, 0),(11, 12, color_map[0]),(12, 12, color_map[0]),(13, 12, color_map[0]),(14, 12, color_map[2]),(15, 12, 0),(16, 12, 0),(6, 13, 0),(7, 13, color_map[0]),(8, 13, color_map[0]),(9, 13, 0),(10, 13, color_map[1]),(11, 13, color_map[0]),(12, 13, color_map[0]),(13, 13, color_map[0]),(14, 13, 0),(15, 13, color_map[1]),(16, 13, 0),(6, 14, 0),(7, 14, 0),(8, 14, color_map[0]),(9, 14, color_map[0]),(10, 14, color_map[0]),(11, 14, color_map[0]),(12, 14, color_map[0]),(13, 14, color_map[0]),(14, 14, color_map[0]),(15, 14, color_map[0]),(16, 14, 0),(7, 15, 0),(8, 15, color_map[0]),(9, 15, color_map[0]),(10, 15, color_map[0]),(11, 15, color_map[0]),(12, 15, color_map[0]),(13, 15, color_map[0]),(14, 15, color_map[0]),(15, 15, color_map[0]),(16, 15, 0),(7, 16, 0),(8, 16, color_map[0]),(9, 16, color_map[0]),(10, 16, color_map[0]),(11, 16, color_map[0]),(12, 16, 0),(13, 16, color_map[0]),(14, 16, color_map[0]),(15, 16, color_map[0]),(16, 16, 0),(7, 17, 0),(8, 17, color_map[0]),(9, 17, color_map[0]),(10, 17, color_map[0]),(11, 17, color_map[0]),(12, 17, color_map[0]),(13, 17, color_map[0]),(14, 17, color_map[0]),(15, 17, color_map[0]),(16, 17, 0),(7, 18, 0),(8, 18, color_map[0]),(9, 18, color_map[0]),(10, 18, color_map[0]),(11, 18, 0),(12, 18, 0),(13, 18, 0),(14, 18, color_map[0]),(15, 18, color_map[0]),(16, 18, 0),(8, 19, 0),(9, 19, color_map[0]),(10, 19, color_map[0]),(11, 19, color_map[0]),(12, 19, color_map[0]),(13, 19, color_map[0]),(14, 19, color_map[0]),(15, 19, 0),(8, 20, 0),(9, 20, color_map[0]),(10, 20, 0),(11, 20, color_map[0]),(12, 20, color_map[0]),(13, 20, color_map[0]),(14, 20, 0),(8, 21, 0),(9, 21, color_map[0]),(10, 21, color_map[0]),(11, 21, 0),(12, 21, 0),(13, 21, 0),(8, 22, 0),(9, 22, color_map[0]),(10, 22, color_map[0]),(11, 22, color_map[0]),(12, 22, 0),(8, 23, 0),(9, 23, color_map[0]),(10, 23, color_map[0]),(11, 23, color_map[0]),(12, 23, 0)];
}


fn get_male_asset(
    asset_name: &str
) -> Vec<(u8,u8,u8)> {
    match asset_name {
        "Big Beard" => return vec![(9, 16, 8), (10, 16, 8), (11, 16, 8), (12, 16, 8), (13, 16, 8), (14, 16, 8), (15, 16, 8), (7, 17, 8), (8, 17, 8), (9, 17, 8), (10, 17, 8), (11, 17, 8), (12, 17, 8), (13, 17, 8), (14, 17, 8), (15, 17, 8), (16, 17, 8), (17, 17, 0), (5, 18, 0), (6, 18, 8), (7, 18, 8), (8, 18, 8), (9, 18, 8), (10, 18, 8), (14, 18, 8), (15, 18, 8), (16, 18, 8), (17, 18, 0), (5, 19, 0), (6, 19, 8), (7, 19, 8), (8, 19, 8), (9, 19, 8), (10, 19, 8), (11, 19, 8), (12, 19, 8), (13, 19, 8), (14, 19, 8), (15, 19, 8), (16, 19, 8), (17, 19, 0), (5, 20, 0), (6, 20, 8), (7, 20, 8), (8, 20, 8), (9, 20, 8), (10, 20, 8), (11, 20, 8), (12, 20, 8), (13, 20, 8), (14, 20, 8), (15, 20, 8), (16, 20, 8), (17, 20, 0), (6, 21, 0), (7, 21, 0), (8, 21, 8), (9, 21, 8), (10, 21, 8), (11, 21, 8), (12, 21, 8), (13, 21, 8), (14, 21, 8), (15, 21, 8), (16, 21, 8), (17, 21, 0), (6, 22, 0), (8, 22, 0), (9, 22, 0), (10, 22, 0), (11, 22, 8), (12, 22, 8), (13, 22, 8), (14, 22, 8), (15, 22, 8), (16, 22, 0), (6, 23, 0), (10, 23, 0), (11, 23, 0), (12, 23, 0), (13, 23, 0), (14, 23, 0), (15, 23, 0)],
        "Chinstrap" => return vec![(7, 15, 8), (7, 16, 8), (8, 16, 8), (15, 16, 8), (7, 17, 8), (8, 17, 8), (15, 17, 8), (8, 18, 8), (9, 18, 8), (15, 18, 8), (8, 19, 8), (9, 19, 8), (15, 19, 8), (9, 20, 8), (10, 20, 8), (11, 20, 8), (12, 20, 8), (13, 20, 8), (14, 20, 8), (15, 20, 0), (11, 21, 8), (12, 21, 8), (13, 21, 8), (14, 21, 0), (11, 22, 0), (12, 22, 0), (13, 22, 0)],
        "Front Beard Dark" => return vec![(10, 17, 39), (11, 17, 39), (12, 17, 39), (13, 17, 39), (14, 17, 39), (10, 18, 39), (14, 18, 39), (10, 19, 39), (11, 19, 39), (12, 19, 39), (13, 19, 39), (14, 19, 39), (11, 20, 39), (12, 20, 39), (13, 20, 39), (11, 21, 39), (12, 21, 39), (13, 21, 39), (11, 22, 0), (12, 22, 0), (13, 22, 0)],
        "Front Beard" => return vec![(10, 17, 8), (11, 17, 8), (12, 17, 8), (13, 17, 8), (14, 17, 8), (10, 18, 8), (14, 18, 8), (10, 19, 8), (11, 19, 8), (12, 19, 8), (13, 19, 8), (14, 19, 8), (11, 20, 8), (12, 20, 8), (13, 20, 8), (11, 21, 8), (12, 21, 8), (13, 21, 8), (11, 22, 0), (12, 22, 0), (13, 22, 0)],
        "GoatU" => return vec![(11, 20, 8), (12, 20, 8), (13, 20, 8), (11, 21, 8), (12, 21, 8), (13, 21, 8), (11, 22, 0), (12, 22, 8), (13, 22, 0), (12, 23, 0)],
        "Handlebars" => return vec![(10, 17, 118), (11, 17, 8), (12, 17, 8), (13, 17, 8), (14, 17, 118), (10, 18, 8), (14, 18, 8), (10, 19, 8), (14, 19, 8)],
        "Luxurious Beard" => return vec![(6, 14, 0), (6, 15, 0), (7, 15, 0), (16, 15, 0), (6, 16, 0), (7, 16, 0), (8, 16, 0), (9, 16, 0), (15, 16, 0), (16, 16, 0), (6, 17, 0), (7, 17, 0), (8, 17, 0), (9, 17, 0), (10, 17, 0), (11, 17, 0), (12, 17, 0), (13, 17, 0), (14, 17, 0), (15, 17, 0), (16, 17, 0), (6, 18, 0), (7, 18, 0), (8, 18, 0), (9, 18, 0), (10, 18, 0), (11, 18, 112), (12, 18, 112), (13, 18, 112), (14, 18, 0), (15, 18, 0), (16, 18, 0), (7, 19, 0), (8, 19, 0), (9, 19, 0), (10, 19, 0), (11, 19, 0), (12, 19, 0), (13, 19, 0), (14, 19, 0), (15, 19, 0), (16, 19, 0), (8, 20, 0), (9, 20, 0), (10, 20, 0), (11, 20, 0), (12, 20, 0), (13, 20, 0), (14, 20, 0), (15, 20, 0), (8, 21, 0), (9, 21, 0), (10, 21, 0), (11, 21, 0), (12, 21, 0), (13, 21, 0), (14, 21, 0), (15, 21, 0), (9, 22, 0), (10, 22, 0), (11, 22, 0), (12, 22, 0), (13, 22, 0), (14, 22, 0)],
        "Mustache" => return vec![(10, 17, 87), (11, 17, 87), (12, 17, 87), (13, 17, 87), (14, 17, 87)],
        "Muttonchops" => return vec![(7, 15, 8), (7, 16, 8), (8, 16, 8), (15, 16, 8), (7, 17, 8), (8, 17, 8), (9, 17, 8), (15, 17, 8), (8, 18, 8), (9, 18, 8), (15, 18, 8)],
        "Normal Beard Black" => return vec![(6, 15, 0), (7, 15, 0), (16, 15, 0), (6, 16, 0), (7, 16, 0), (8, 16, 0), (15, 16, 0), (16, 16, 0), (6, 17, 0), (7, 17, 0), (8, 17, 0), (9, 17, 0), (10, 17, 0), (11, 17, 0), (12, 17, 0), (13, 17, 0), (14, 17, 0), (15, 17, 0), (16, 17, 0), (7, 18, 0), (8, 18, 0), (9, 18, 0), (10, 18, 0), (11, 18, 46), (12, 18, 46), (13, 18, 46), (14, 18, 0), (15, 18, 0), (16, 18, 0), (8, 19, 0), (9, 19, 0), (10, 19, 0), (11, 19, 0), (12, 19, 0), (13, 19, 0), (14, 19, 0), (15, 19, 0), (16, 19, 0), (9, 20, 0), (10, 20, 0), (11, 20, 0), (12, 20, 0), (13, 20, 0), (14, 20, 0), (15, 20, 0), (10, 21, 0), (11, 21, 0), (12, 21, 0), (13, 21, 0), (14, 21, 0)],
        "Normal Beard" => return vec![(7, 15, 8), (7, 16, 8), (8, 16, 8), (15, 16, 8), (7, 17, 8), (8, 17, 8), (9, 17, 8), (10, 17, 8), (11, 17, 8), (12, 17, 8), (13, 17, 8), (14, 17, 8), (15, 17, 8), (7, 18, 8), (8, 18, 8), (9, 18, 8), (10, 18, 8), (11, 18, 0), (12, 18, 0), (13, 18, 0), (14, 18, 8), (15, 18, 8), (8, 19, 8), (9, 19, 8), (10, 19, 8), (11, 19, 8), (12, 19, 8), (13, 19, 8), (14, 19, 8), (15, 19, 8), (9, 20, 8), (10, 20, 8), (11, 20, 8), (12, 20, 8), (13, 20, 8), (14, 20, 8)],
        "Shadow Beard" => return vec![(7, 15, 31), (7, 16, 31), (8, 16, 31), (15, 16, 31), (7, 17, 31), (8, 17, 31), (9, 17, 31), (10, 17, 31), (11, 17, 31), (12, 17, 31), (13, 17, 31), (14, 17, 31), (15, 17, 31), (7, 18, 31), (8, 18, 31), (9, 18, 31), (10, 18, 31), (11, 18, 113), (12, 18, 113), (13, 18, 113), (14, 18, 31), (15, 18, 31), (8, 19, 31), (9, 19, 31), (10, 19, 31), (11, 19, 31), (12, 19, 31), (13, 19, 31), (14, 19, 31), (15, 19, 31), (9, 20, 31), (10, 20, 31), (11, 20, 31), (12, 20, 31), (13, 20, 31), (14, 20, 31)],
        "Earring" => return vec![(5, 13, 0), (4, 14, 0), (5, 14, 89), (6, 14, 0), (5, 15, 0)],
        "Clown Eyes Blue" => return vec![(9, 10, 49), (14, 10, 49), (9, 11, 50), (10, 11, 50), (14, 11, 50), (15, 11, 50), (9, 12, 0), (10, 12, 79), (14, 12, 0), (15, 12, 79), (9, 13, 49), (14, 13, 49)],
        "Clown Eyes Green" => return vec![(9, 10, 9), (14, 10, 9), (9, 11, 51), (10, 11, 51), (14, 11, 51), (15, 11, 51), (9, 12, 0), (10, 12, 80), (14, 12, 0), (15, 12, 80), (9, 13, 9), (14, 13, 9)],
        "Mole" => return vec![(8, 16, 130)],
        "Rosy Cheeks" => return vec![(9, 15, 88), (10, 15, 88), (15, 15, 88), (9, 16, 88), (15, 16, 88)],
        "Spots" => return vec![(9, 7, 137), (8, 8, 137), (10, 8, 41), (14, 8, 41), (7, 13, 41), (14, 14, 41), (9, 16, 41), (15, 17, 41), (8, 20, 41), (12, 20, 41)],
        "3D Glasses" => return vec![(6, 10, 23), (7, 10, 23), (8, 10, 23), (9, 10, 23), (10, 10, 23), (11, 10, 23), (12, 10, 23), (13, 10, 23), (14, 10, 23), (15, 10, 23), (16, 10, 23), (7, 11, 23), (8, 11, 23), (9, 11, 52), (10, 11, 52), (11, 11, 52), (12, 11, 23), (13, 11, 53), (14, 11, 53), (15, 11, 53), (16, 11, 23), (8, 12, 23), (9, 12, 52), (10, 12, 52), (11, 12, 52), (12, 12, 23), (13, 12, 53), (14, 12, 53), (15, 12, 53), (16, 12, 23), (8, 13, 23), (9, 13, 23), (10, 13, 23), (11, 13, 23), (12, 13, 23), (13, 13, 23), (14, 13, 23), (15, 13, 23), (16, 13, 23)],
        "Big Shades" => return vec![(7, 9, 0), (8, 9, 0), (9, 9, 0), (10, 9, 0), (11, 9, 0), (13, 9, 0), (14, 9, 0), (15, 9, 0), (16, 9, 0), (17, 9, 0), (7, 10, 0), (8, 10, 54), (9, 10, 54), (10, 10, 54), (11, 10, 0), (12, 10, 0), (13, 10, 0), (14, 10, 54), (15, 10, 54), (16, 10, 54), (17, 10, 0), (6, 11, 0), (7, 11, 0), (8, 11, 55), (9, 11, 55), (10, 11, 55), (11, 11, 0), (13, 11, 0), (14, 11, 55), (15, 11, 55), (16, 11, 55), (17, 11, 0), (7, 12, 0), (8, 12, 56), (9, 12, 56), (10, 12, 56), (11, 12, 0), (13, 12, 0), (14, 12, 56), (15, 12, 56), (16, 12, 56), (17, 12, 0), (8, 13, 0), (9, 13, 0), (10, 13, 0), (14, 13, 0), (15, 13, 0), (16, 13, 0)],
        "Classic Shades" => return vec![(6, 10, 0), (7, 10, 0), (8, 10, 0), (9, 10, 0), (10, 10, 0), (11, 10, 0), (12, 10, 0), (13, 10, 0), (14, 10, 0), (15, 10, 0), (16, 10, 0), (8, 11, 0), (9, 11, 69), (10, 11, 69), (11, 11, 0), (13, 11, 0), (14, 11, 69), (15, 11, 69), (16, 11, 0), (8, 12, 0), (9, 12, 70), (10, 12, 70), (11, 12, 0), (13, 12, 0), (14, 12, 70), (15, 12, 70), (16, 12, 0), (9, 13, 0), (10, 13, 0), (14, 13, 0), (15, 13, 0)],
        "Eye Mask" => return vec![(6, 10, 0), (7, 10, 0), (8, 10, 0), (9, 10, 0), (10, 10, 0), (11, 10, 0), (12, 10, 0), (13, 10, 0), (14, 10, 0), (15, 10, 0), (16, 10, 0), (6, 11, 0), (7, 11, 0), (8, 11, 0), (9, 11, 46), (10, 11, 46), (11, 11, 0), (12, 11, 0), (13, 11, 0), (14, 11, 46), (15, 11, 46), (16, 11, 0), (5, 12, 0), (6, 12, 0), (7, 12, 0), (8, 12, 0), (9, 12, 90), (10, 12, 71), (11, 12, 0), (12, 12, 0), (13, 12, 0), (14, 12, 90), (15, 12, 71), (16, 12, 0), (5, 13, 0), (6, 13, 0), (7, 13, 0), (8, 13, 0), (9, 13, 0), (10, 13, 0), (11, 13, 0), (12, 13, 0), (13, 13, 0), (14, 13, 0), (15, 13, 0), (16, 13, 0)],
        "Eye Patch" => return vec![(6, 10, 0), (7, 10, 0), (8, 10, 0), (9, 10, 0), (10, 10, 0), (11, 10, 0), (12, 10, 0), (13, 10, 0), (14, 10, 0), (15, 10, 0), (16, 10, 0), (8, 11, 0), (9, 11, 0), (10, 11, 0), (11, 11, 0), (8, 12, 0), (9, 12, 0), (10, 12, 0), (11, 12, 0), (9, 13, 0), (10, 13, 0)],
        "Horned Rim Glasses" => return vec![(6, 10, 0), (7, 10, 0), (8, 10, 0), (9, 10, 0), (10, 10, 0), (11, 10, 0), (12, 10, 0), (13, 10, 0), (14, 10, 0), (15, 10, 0), (16, 10, 0), (17, 10, 0), (6, 11, 0), (7, 11, 0), (8, 11, 41), (9, 11, 41), (10, 11, 41), (11, 11, 0), (12, 11, 0), (13, 11, 41), (14, 11, 41), (15, 11, 41), (16, 11, 0), (17, 11, 0), (8, 12, 41), (9, 12, 41), (10, 12, 41), (13, 12, 41), (14, 12, 41), (15, 12, 41), (8, 13, 41), (9, 13, 41), (10, 13, 41), (13, 13, 41), (14, 13, 41), (15, 13, 41)],
        "Nerd Glasses" => return vec![(8, 10, 0), (9, 10, 0), (10, 10, 0), (11, 10, 0), (13, 10, 0), (14, 10, 0), (15, 10, 0), (16, 10, 0), (6, 11, 0), (7, 11, 0), (8, 11, 0), (9, 11, 30), (10, 11, 30), (11, 11, 0), (12, 11, 0), (13, 11, 0), (14, 11, 30), (15, 11, 30), (16, 11, 0), (8, 12, 0), (9, 12, 30), (10, 12, 30), (11, 12, 0), (13, 12, 0), (14, 12, 30), (15, 12, 30), (16, 12, 0), (8, 13, 0), (9, 13, 0), (10, 13, 0), (11, 13, 0), (13, 13, 0), (14, 13, 0), (15, 13, 0), (16, 13, 0)],
        "Regular Shades" => return vec![(5, 11, 0), (6, 11, 0), (7, 11, 0), (8, 11, 0), (9, 11, 0), (10, 11, 0), (11, 11, 0), (12, 11, 0), (13, 11, 0), (14, 11, 0), (15, 11, 0), (16, 11, 0), (17, 11, 0), (8, 12, 0), (9, 12, 0), (10, 12, 0), (11, 12, 0), (14, 12, 0), (15, 12, 0), (16, 12, 0), (17, 12, 0), (9, 13, 0), (10, 13, 0), (15, 13, 0), (16, 13, 0)],
        "Small Shades" => return vec![(6, 11, 0), (7, 11, 0), (8, 11, 0), (9, 11, 0), (10, 11, 0), (11, 11, 0), (12, 11, 0), (13, 11, 0), (14, 11, 0), (15, 11, 0), (16, 11, 0), (9, 12, 0), (10, 12, 0), (14, 12, 0), (15, 12, 0), (9, 13, 0), (10, 13, 0), (14, 13, 0), (15, 13, 0)],
        "VR" => return vec![(8, 9, 0), (9, 9, 0), (10, 9, 0), (11, 9, 0), (12, 9, 0), (13, 9, 0), (14, 9, 0), (15, 9, 0), (16, 9, 0), (7, 10, 0), (8, 10, 57), (9, 10, 29), (10, 10, 29), (11, 10, 29), (12, 10, 29), (13, 10, 29), (14, 10, 29), (15, 10, 29), (16, 10, 57), (17, 10, 0), (6, 11, 0), (7, 11, 57), (8, 11, 29), (9, 11, 0), (10, 11, 0), (11, 11, 0), (12, 11, 0), (13, 11, 0), (14, 11, 0), (15, 11, 0), (16, 11, 29), (17, 11, 0), (6, 12, 0), (7, 12, 57), (8, 12, 29), (9, 12, 0), (10, 12, 0), (11, 12, 0), (12, 12, 0), (13, 12, 0), (14, 12, 0), (15, 12, 0), (16, 12, 29), (17, 12, 0), (7, 13, 0), (8, 13, 57), (9, 13, 29), (10, 13, 29), (11, 13, 29), (12, 13, 29), (13, 13, 29), (14, 13, 29), (15, 13, 29), (16, 13, 57), (17, 13, 0), (8, 14, 0), (9, 14, 0), (10, 14, 0), (11, 14, 0), (12, 14, 0), (13, 14, 0), (14, 14, 0), (15, 14, 0), (16, 14, 0)],
        "Bandana" => return vec![(8, 5, 26), (9, 5, 26), (10, 5, 26), (11, 5, 26), (12, 5, 26), (13, 5, 26), (14, 5, 26), (15, 5, 26), (7, 6, 26), (8, 6, 24), (9, 6, 24), (10, 6, 24), (11, 6, 24), (12, 6, 24), (13, 6, 24), (14, 6, 24), (15, 6, 24), (16, 6, 26), (6, 7, 26), (7, 7, 24), (8, 7, 24), (9, 7, 24), (10, 7, 24), (11, 7, 24), (12, 7, 24), (13, 7, 24), (14, 7, 24), (15, 7, 24), (16, 7, 26), (2, 8, 24), (3, 8, 26), (4, 8, 24), (5, 8, 26), (6, 8, 82), (7, 8, 24), (8, 8, 26), (9, 8, 26), (10, 8, 26), (11, 8, 26), (12, 8, 24), (13, 8, 24), (14, 8, 24), (15, 8, 26), (3, 9, 24), (4, 9, 26), (5, 9, 82), (12, 9, 26), (13, 9, 26), (14, 9, 26), (3, 10, 24), (4, 10, 82), (3, 11, 24)],
        "Beanie" => return vec![(9, 3, 47), (10, 3, 47), (11, 3, 47), (12, 3, 47), (13, 3, 47), (11, 4, 0), (8, 5, 47), (9, 5, 47), (10, 5, 40), (11, 5, 40), (12, 5, 40), (13, 5, 62), (14, 5, 62), (7, 6, 47), (8, 6, 47), (9, 6, 47), (10, 6, 40), (11, 6, 40), (12, 6, 40), (13, 6, 62), (14, 6, 62), (15, 6, 62), (6, 7, 47), (7, 7, 47), (8, 7, 47), (9, 7, 40), (10, 7, 40), (11, 7, 40), (12, 7, 40), (13, 7, 40), (14, 7, 62), (15, 7, 62), (16, 7, 62), (6, 8, 47), (7, 8, 47), (8, 8, 40), (9, 8, 40), (10, 8, 40), (11, 8, 40), (12, 8, 40), (13, 8, 40), (14, 8, 40), (15, 8, 62), (16, 8, 62), (8, 9, 77), (9, 9, 77), (10, 9, 77), (11, 9, 77), (12, 9, 77), (13, 9, 77), (14, 9, 77)],
        "Cap Forward" => return vec![(8, 4, 0), (9, 4, 0), (10, 4, 0), (11, 4, 0), (12, 4, 0), (13, 4, 0), (14, 4, 0), (7, 5, 0), (8, 5, 63), (9, 5, 38), (10, 5, 38), (11, 5, 38), (12, 5, 38), (13, 5, 38), (14, 5, 38), (15, 5, 0), (6, 6, 0), (7, 6, 63), (8, 6, 38), (9, 6, 38), (10, 6, 38), (11, 6, 38), (12, 6, 38), (13, 6, 38), (14, 6, 38), (15, 6, 38), (16, 6, 0), (6, 7, 0), (7, 7, 38), (8, 7, 38), (9, 7, 38), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (15, 7, 0), (16, 7, 0), (17, 7, 0), (6, 8, 0), (7, 8, 38), (8, 8, 38), (9, 8, 0), (10, 8, 63), (11, 8, 63), (12, 8, 63), (13, 8, 63), (14, 8, 63), (15, 8, 63), (16, 8, 63), (17, 8, 63), (18, 8, 0), (6, 9, 0), (7, 9, 0), (8, 9, 0), (9, 9, 0), (10, 9, 0), (11, 9, 0), (12, 9, 0), (13, 9, 0), (14, 9, 0), (15, 9, 0), (16, 9, 0), (17, 9, 0), (18, 9, 0)],
        "Cap" => return vec![(8, 4, 14), (9, 4, 14), (10, 4, 14), (11, 4, 14), (12, 4, 14), (13, 4, 14), (14, 4, 14), (7, 5, 14), (8, 5, 14), (9, 5, 14), (10, 5, 14), (11, 5, 14), (12, 5, 14), (13, 5, 93), (14, 5, 14), (15, 5, 14), (6, 6, 14), (7, 6, 14), (8, 6, 14), (9, 6, 14), (10, 6, 14), (11, 6, 14), (12, 6, 14), (13, 6, 14), (14, 6, 93), (15, 6, 14), (6, 7, 14), (7, 7, 14), (8, 7, 14), (9, 7, 14), (10, 7, 14), (11, 7, 14), (12, 7, 14), (13, 7, 14), (14, 7, 14), (15, 7, 14), (16, 7, 14), (17, 7, 14), (18, 7, 14), (6, 8, 14), (7, 8, 14), (8, 8, 14), (9, 8, 14), (10, 8, 14), (11, 8, 14), (12, 8, 14), (13, 8, 14), (14, 8, 14), (15, 8, 14), (16, 8, 14), (17, 8, 14), (18, 8, 14), (19, 8, 14)],
        "Clown Hair Green" => return vec![(7, 4, 9), (8, 4, 9), (14, 4, 9), (15, 4, 9), (6, 5, 9), (7, 5, 9), (15, 5, 9), (16, 5, 9), (5, 6, 9), (6, 6, 9), (7, 6, 9), (15, 6, 9), (16, 6, 9), (17, 6, 9), (4, 7, 9), (5, 7, 9), (6, 7, 9), (16, 7, 9), (17, 7, 9), (18, 7, 9), (4, 8, 9), (5, 8, 9), (6, 8, 9), (16, 8, 9), (17, 8, 9), (18, 8, 9), (2, 9, 9), (3, 9, 9), (4, 9, 9), (5, 9, 9), (6, 9, 9), (16, 9, 9), (17, 9, 9), (18, 9, 9), (19, 9, 9), (20, 9, 9), (2, 10, 9), (3, 10, 9), (4, 10, 9), (5, 10, 9), (6, 10, 9), (16, 10, 9), (17, 10, 9), (18, 10, 9), (19, 10, 9), (20, 10, 9), (3, 11, 9), (4, 11, 9), (5, 11, 9), (6, 11, 9), (16, 11, 9), (17, 11, 9), (18, 11, 9), (19, 11, 9), (4, 12, 9), (5, 12, 9), (16, 12, 9), (17, 12, 9), (18, 12, 9), (5, 13, 9), (17, 13, 9)],
        "Cowboy Hat" => return vec![(8, 3, 20), (9, 3, 20), (13, 3, 20), (14, 3, 20), (7, 4, 20), (8, 4, 20), (9, 4, 20), (10, 4, 20), (11, 4, 20), (12, 4, 20), (13, 4, 20), (14, 4, 20), (15, 4, 20), (7, 5, 20), (8, 5, 20), (9, 5, 20), (10, 5, 20), (11, 5, 20), (12, 5, 20), (13, 5, 20), (14, 5, 20), (15, 5, 20), (7, 6, 20), (8, 6, 20), (9, 6, 20), (10, 6, 20), (11, 6, 20), (12, 6, 20), (13, 6, 20), (14, 6, 20), (15, 6, 20), (2, 7, 20), (6, 7, 60), (7, 7, 60), (8, 7, 60), (9, 7, 60), (10, 7, 60), (11, 7, 60), (12, 7, 60), (13, 7, 60), (14, 7, 60), (15, 7, 60), (16, 7, 60), (20, 7, 20), (2, 8, 20), (3, 8, 20), (4, 8, 20), (5, 8, 20), (6, 8, 20), (7, 8, 20), (8, 8, 20), (9, 8, 20), (10, 8, 20), (11, 8, 20), (12, 8, 20), (13, 8, 20), (14, 8, 20), (15, 8, 20), (16, 8, 20), (17, 8, 20), (18, 8, 20), (19, 8, 20), (20, 8, 20), (3, 9, 20), (4, 9, 20), (5, 9, 20), (6, 9, 20), (7, 9, 20), (8, 9, 20), (9, 9, 20), (10, 9, 20), (11, 9, 20), (12, 9, 20), (13, 9, 20), (14, 9, 20), (15, 9, 20), (16, 9, 20), (17, 9, 20), (18, 9, 20), (19, 9, 20)],
        "Crazy Hair" => return vec![(7, 2, 10), (12, 2, 10), (6, 3, 10), (7, 3, 10), (8, 3, 10), (10, 3, 10), (11, 3, 10), (12, 3, 10), (13, 3, 10), (16, 3, 10), (6, 4, 10), (7, 4, 10), (8, 4, 10), (9, 4, 10), (10, 4, 10), (11, 4, 10), (12, 4, 10), (14, 4, 10), (15, 4, 10), (16, 4, 10), (4, 5, 10), (5, 5, 10), (6, 5, 10), (7, 5, 10), (8, 5, 10), (9, 5, 10), (10, 5, 10), (11, 5, 10), (12, 5, 10), (13, 5, 10), (14, 5, 10), (15, 5, 10), (16, 5, 10), (17, 5, 10), (2, 6, 10), (5, 6, 10), (6, 6, 10), (7, 6, 10), (8, 6, 10), (9, 6, 10), (10, 6, 10), (11, 6, 10), (12, 6, 10), (13, 6, 10), (14, 6, 10), (15, 6, 10), (16, 6, 10), (17, 6, 10), (18, 6, 10), (19, 6, 10), (2, 7, 10), (3, 7, 10), (4, 7, 10), (5, 7, 10), (6, 7, 10), (7, 7, 10), (8, 7, 10), (9, 7, 10), (10, 7, 10), (11, 7, 10), (12, 7, 10), (13, 7, 10), (14, 7, 10), (15, 7, 10), (16, 7, 10), (17, 7, 10), (18, 7, 10), (4, 8, 10), (5, 8, 10), (6, 8, 10), (7, 8, 10), (8, 8, 10), (9, 8, 10), (12, 8, 10), (14, 8, 10), (16, 8, 10), (4, 9, 10), (5, 9, 10), (8, 9, 10), (3, 10, 10), (4, 10, 10), (5, 10, 10), (3, 11, 10), (4, 11, 10), (4, 12, 10), (4, 13, 10), (3, 14, 10)],
        "Do-rag" => return vec![(8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (7, 6, 0), (8, 6, 32), (9, 6, 32), (10, 6, 32), (11, 6, 32), (12, 6, 32), (13, 6, 32), (14, 6, 32), (15, 6, 0), (6, 7, 0), (7, 7, 32), (8, 7, 32), (9, 7, 119), (10, 7, 32), (11, 7, 32), (12, 7, 32), (13, 7, 32), (14, 7, 32), (15, 7, 32), (16, 7, 0), (6, 8, 0), (7, 8, 32), (8, 8, 119), (9, 8, 32), (10, 8, 32), (11, 8, 32), (12, 8, 32), (13, 8, 32), (14, 8, 32), (15, 8, 32), (16, 8, 0), (6, 9, 0), (7, 9, 32), (8, 9, 32), (9, 9, 32), (10, 9, 32), (11, 9, 32), (12, 9, 32), (13, 9, 32), (14, 9, 32), (15, 9, 32), (16, 9, 0)],
        "Fedora" => return vec![(9, 3, 22), (10, 3, 22), (11, 3, 22), (12, 3, 22), (13, 3, 22), (8, 4, 22), (9, 4, 22), (10, 4, 22), (11, 4, 22), (12, 4, 22), (13, 4, 22), (14, 4, 22), (8, 5, 22), (9, 5, 22), (10, 5, 22), (11, 5, 22), (12, 5, 22), (13, 5, 22), (14, 5, 22), (7, 6, 22), (8, 6, 22), (9, 6, 22), (10, 6, 22), (11, 6, 22), (12, 6, 22), (13, 6, 22), (14, 6, 22), (15, 6, 22), (6, 7, 0), (7, 7, 0), (8, 7, 0), (9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (15, 7, 0), (16, 7, 0), (4, 8, 22), (5, 8, 22), (6, 8, 22), (7, 8, 22), (8, 8, 22), (9, 8, 22), (10, 8, 22), (11, 8, 22), (12, 8, 22), (13, 8, 22), (14, 8, 22), (15, 8, 22), (16, 8, 22), (17, 8, 22), (18, 8, 22), (3, 9, 22), (4, 9, 22), (5, 9, 22), (6, 9, 22), (7, 9, 22), (8, 9, 22), (9, 9, 22), (10, 9, 22), (11, 9, 22), (12, 9, 22), (13, 9, 22), (14, 9, 22), (15, 9, 22), (16, 9, 22), (17, 9, 22), (18, 9, 22), (19, 9, 22)],
        "Frumpy Hair" => return vec![(8, 3, 0), (9, 3, 0), (10, 3, 0), (11, 3, 0), (12, 3, 0), (13, 3, 0), (14, 3, 0), (7, 4, 0), (8, 4, 0), (9, 4, 94), (10, 4, 0), (11, 4, 0), (12, 4, 0), (13, 4, 0), (14, 4, 0), (15, 4, 0), (6, 5, 0), (7, 5, 0), (8, 5, 94), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (15, 5, 0), (16, 5, 0), (5, 6, 0), (6, 6, 0), (7, 6, 0), (8, 6, 0), (9, 6, 0), (10, 6, 0), (11, 6, 0), (12, 6, 0), (13, 6, 0), (14, 6, 0), (15, 6, 0), (16, 6, 0), (17, 6, 0), (5, 7, 0), (6, 7, 0), (7, 7, 0), (8, 7, 0), (9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (15, 7, 0), (16, 7, 0), (17, 7, 0), (5, 8, 0), (6, 8, 0), (7, 8, 0), (8, 8, 0), (9, 8, 0), (10, 8, 0), (11, 8, 0), (12, 8, 0), (13, 8, 0), (14, 8, 0), (15, 8, 0), (16, 8, 0), (17, 8, 0), (5, 9, 0), (6, 9, 0), (7, 9, 0), (8, 9, 0), (9, 9, 0), (11, 9, 0), (12, 9, 0), (13, 9, 0), (14, 9, 0), (15, 9, 0), (16, 9, 0), (17, 9, 0), (5, 10, 0), (6, 10, 0), (7, 10, 0), (8, 10, 0), (11, 10, 0), (12, 10, 0), (13, 10, 0), (5, 11, 0), (6, 11, 0), (7, 11, 0), (12, 11, 0), (5, 12, 0), (6, 12, 0), (7, 12, 0), (5, 13, 0), (6, 13, 0), (7, 13, 0)],
        "Headband" => return vec![(7, 7, 12), (8, 7, 12), (9, 7, 12), (10, 7, 12), (11, 7, 12), (12, 7, 12), (13, 7, 12), (14, 7, 12), (15, 7, 12), (7, 8, 43), (8, 8, 43), (9, 8, 43), (10, 8, 43), (11, 8, 43), (12, 8, 43), (13, 8, 43), (14, 8, 43), (15, 8, 43)],
        "Hoodie" => return vec![(10, 3, 0), (11, 3, 0), (12, 3, 0), (8, 4, 0), (9, 4, 0), (10, 4, 17), (11, 4, 17), (12, 4, 17), (13, 4, 0), (14, 4, 0), (7, 5, 0), (8, 5, 17), (9, 5, 17), (10, 5, 17), (11, 5, 17), (12, 5, 17), (13, 5, 17), (14, 5, 17), (15, 5, 0), (6, 6, 0), (7, 6, 17), (8, 6, 17), (9, 6, 17), (10, 6, 17), (11, 6, 17), (12, 6, 17), (13, 6, 17), (14, 6, 17), (15, 6, 17), (16, 6, 0), (5, 7, 0), (6, 7, 17), (7, 7, 17), (8, 7, 17), (9, 7, 17), (10, 7, 17), (11, 7, 17), (12, 7, 17), (13, 7, 17), (14, 7, 17), (15, 7, 17), (16, 7, 17), (17, 7, 0), (4, 8, 0), (5, 8, 17), (6, 8, 17), (7, 8, 17), (8, 8, 0), (9, 8, 0), (10, 8, 0), (11, 8, 0), (12, 8, 0), (13, 8, 0), (14, 8, 0), (15, 8, 17), (16, 8, 17), (17, 8, 17), (18, 8, 0), (4, 9, 0), (5, 9, 17), (6, 9, 17), (7, 9, 0), (15, 9, 0), (16, 9, 17), (17, 9, 17), (18, 9, 0), (4, 10, 0), (5, 10, 17), (6, 10, 0), (16, 10, 0), (17, 10, 17), (18, 10, 0), (3, 11, 0), (4, 11, 17), (5, 11, 17), (6, 11, 0), (16, 11, 0), (17, 11, 17), (18, 11, 17), (19, 11, 0), (3, 12, 0), (4, 12, 17), (5, 12, 0), (16, 12, 0), (17, 12, 0), (18, 12, 17), (19, 12, 0), (3, 13, 0), (4, 13, 17), (5, 13, 0), (16, 13, 0), (17, 13, 0), (18, 13, 17), (19, 13, 0), (2, 14, 0), (3, 14, 17), (4, 14, 0), (5, 14, 0), (6, 14, 0), (16, 14, 0), (17, 14, 0), (18, 14, 0), (19, 14, 17), (20, 14, 0), (2, 15, 0), (3, 15, 17), (4, 15, 0), (5, 15, 0), (6, 15, 0), (16, 15, 0), (17, 15, 0), (18, 15, 0), (19, 15, 17), (20, 15, 0), (2, 16, 0), (3, 16, 17), (4, 16, 0), (5, 16, 0), (6, 16, 0), (16, 16, 0), (17, 16, 0), (18, 16, 0), (19, 16, 17), (20, 16, 0), (2, 17, 0), (3, 17, 17), (4, 17, 0), (5, 17, 0), (6, 17, 0), (16, 17, 0), (17, 17, 0), (18, 17, 0), (19, 17, 17), (20, 17, 0), (2, 18, 0), (3, 18, 17), (4, 18, 0), (5, 18, 0), (6, 18, 0), (16, 18, 0), (17, 18, 0), (18, 18, 0), (19, 18, 17), (20, 18, 0), (3, 19, 0), (4, 19, 17), (5, 19, 0), (6, 19, 0), (16, 19, 0), (17, 19, 0), (18, 19, 17), (19, 19, 0), (3, 20, 0), (4, 20, 17), (5, 20, 0), (6, 20, 0), (15, 20, 0), (16, 20, 0), (17, 20, 17), (18, 20, 0), (4, 21, 0), (5, 21, 17), (6, 21, 0), (10, 21, 0), (11, 21, 0), (12, 21, 0), (13, 21, 0), (14, 21, 0), (15, 21, 0), (16, 21, 17), (17, 21, 0), (4, 22, 0), (5, 22, 17), (6, 22, 0), (10, 22, 0), (11, 22, 0), (12, 22, 0), (13, 22, 17), (14, 22, 17), (15, 22, 17), (16, 22, 0), (4, 23, 0), (5, 23, 17), (6, 23, 0), (10, 23, 0), (11, 23, 0), (12, 23, 17), (13, 23, 0), (14, 23, 0), (15, 23, 0)],
        "Knitted Cap" => return vec![(8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (7, 6, 0), (8, 6, 28), (9, 6, 28), (10, 6, 28), (11, 6, 28), (12, 6, 28), (13, 6, 28), (14, 6, 28), (15, 6, 0), (6, 7, 0), (7, 7, 28), (8, 7, 28), (9, 7, 28), (10, 7, 28), (11, 7, 28), (12, 7, 28), (13, 7, 28), (14, 7, 28), (15, 7, 28), (16, 7, 0), (5, 8, 0), (6, 8, 33), (7, 8, 33), (8, 8, 33), (9, 8, 33), (10, 8, 33), (11, 8, 33), (12, 8, 33), (13, 8, 33), (14, 8, 33), (15, 8, 33), (16, 8, 33), (17, 8, 0), (5, 9, 0), (6, 9, 33), (7, 9, 28), (8, 9, 33), (9, 9, 28), (10, 9, 33), (11, 9, 28), (12, 9, 33), (13, 9, 28), (14, 9, 33), (15, 9, 28), (16, 9, 33), (17, 9, 0)],
        "Messy Hair" => return vec![(8, 4, 0), (9, 4, 0), (10, 4, 0), (11, 4, 0), (12, 4, 0), (13, 4, 0), (15, 4, 0), (7, 5, 0), (8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (15, 5, 0), (6, 6, 0), (7, 6, 0), (8, 6, 0), (9, 6, 0), (10, 6, 0), (11, 6, 0), (12, 6, 0), (13, 6, 0), (14, 6, 0), (15, 6, 0), (16, 6, 0), (17, 6, 0), (5, 7, 0), (6, 7, 0), (7, 7, 0), (8, 7, 0), (9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (15, 7, 0), (16, 7, 0), (6, 8, 0), (7, 8, 0), (8, 8, 0), (9, 8, 0), (10, 8, 0), (11, 8, 0), (12, 8, 0), (14, 8, 0), (15, 8, 0), (16, 8, 0), (5, 9, 0), (6, 9, 0), (7, 9, 0), (9, 9, 0), (12, 9, 0), (15, 9, 0), (16, 9, 0), (17, 9, 0), (6, 10, 0), (8, 10, 0), (12, 10, 0), (16, 10, 0), (6, 11, 0)],
        "Mohawk Dark" => return vec![(12, 1, 0), (13, 1, 0), (11, 2, 0), (12, 2, 0), (13, 2, 0), (10, 3, 0), (11, 3, 0), (12, 3, 0), (13, 3, 0), (9, 4, 0), (10, 4, 0), (11, 4, 0), (12, 4, 0), (13, 4, 0), (8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (11, 6, 0), (12, 6, 0), (12, 7, 0)],
        "Mohawk Thin" => return vec![(11, 1, 0), (10, 2, 0), (11, 2, 17), (12, 2, 0), (10, 3, 0), (11, 3, 17), (12, 3, 0), (10, 4, 0), (11, 4, 17), (12, 4, 0), (8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 17), (12, 5, 0), (13, 5, 0), (14, 5, 0), (10, 6, 0), (11, 6, 17), (12, 6, 0)],
        "Mohawk" => return vec![(12, 1, 0), (13, 1, 0), (11, 2, 0), (12, 2, 8), (13, 2, 0), (10, 3, 0), (11, 3, 8), (12, 3, 8), (13, 3, 0), (9, 4, 0), (10, 4, 95), (11, 4, 8), (12, 4, 8), (13, 4, 0), (8, 5, 0), (9, 5, 95), (10, 5, 8), (11, 5, 8), (12, 5, 8), (13, 5, 0), (14, 5, 0), (11, 6, 8), (12, 6, 8), (12, 7, 8)],
        "Peak Spike" => return vec![(9, 1, 0), (11, 1, 0), (13, 1, 0), (8, 2, 0), (9, 2, 0), (10, 2, 0), (11, 2, 0), (12, 2, 0), (13, 2, 0), (14, 2, 0), (8, 3, 0), (9, 3, 0), (10, 3, 0), (11, 3, 0), (12, 3, 0), (13, 3, 0), (14, 3, 0), (7, 4, 0), (8, 4, 0), (9, 4, 0), (10, 4, 0), (11, 4, 0), (12, 4, 0), (13, 4, 0), (14, 4, 0), (15, 4, 0), (7, 5, 0), (8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (15, 5, 0), (6, 6, 0), (7, 6, 0), (10, 6, 0), (11, 6, 0), (12, 6, 0), (15, 6, 0), (16, 6, 0), (6, 7, 0), (11, 7, 0), (16, 7, 0)],
        "Police Cap" => return vec![(10, 3, 0), (11, 3, 0), (12, 3, 0), (6, 4, 0), (7, 4, 0), (8, 4, 0), (9, 4, 0), (10, 4, 34), (11, 4, 34), (12, 4, 34), (13, 4, 0), (14, 4, 0), (15, 4, 0), (16, 4, 0), (5, 5, 0), (6, 5, 34), (7, 5, 34), (8, 5, 34), (9, 5, 34), (10, 5, 34), (11, 5, 131), (12, 5, 34), (13, 5, 34), (14, 5, 34), (15, 5, 34), (16, 5, 34), (17, 5, 0), (5, 6, 0), (6, 6, 34), (7, 6, 34), (8, 6, 34), (9, 6, 34), (10, 6, 34), (11, 6, 34), (12, 6, 34), (13, 6, 34), (14, 6, 34), (15, 6, 34), (16, 6, 34), (17, 6, 0), (6, 7, 0), (7, 7, 12), (8, 7, 0), (9, 7, 12), (10, 7, 0), (11, 7, 12), (12, 7, 0), (13, 7, 12), (14, 7, 0), (15, 7, 12), (16, 7, 0), (6, 8, 0), (7, 8, 0), (8, 8, 0), (9, 8, 34), (10, 8, 34), (11, 8, 34), (12, 8, 34), (13, 8, 34), (14, 8, 34), (15, 8, 34), (16, 8, 34), (17, 8, 0), (9, 9, 0), (10, 9, 0), (11, 9, 0), (12, 9, 0), (13, 9, 0), (14, 9, 0), (15, 9, 0), (16, 9, 0), (17, 9, 0)],
        "Purple Hair" => return vec![(9, 3, 18), (11, 3, 18), (6, 4, 18), (8, 4, 18), (9, 4, 18), (10, 4, 18), (11, 4, 18), (12, 4, 18), (13, 4, 18), (4, 5, 18), (5, 5, 18), (6, 5, 18), (7, 5, 18), (8, 5, 18), (9, 5, 18), (10, 5, 18), (11, 5, 18), (12, 5, 18), (13, 5, 18), (14, 5, 18), (5, 6, 18), (6, 6, 18), (7, 6, 18), (8, 6, 18), (9, 6, 18), (10, 6, 18), (11, 6, 18), (12, 6, 18), (13, 6, 18), (14, 6, 18), (15, 6, 18), (3, 7, 18), (4, 7, 18), (5, 7, 18), (6, 7, 18), (7, 7, 18), (8, 7, 18), (9, 7, 18), (10, 7, 18), (11, 7, 18), (12, 7, 18), (13, 7, 18), (14, 7, 18), (15, 7, 18), (16, 7, 18), (4, 8, 18), (5, 8, 18), (6, 8, 18), (7, 8, 18), (8, 8, 18), (15, 8, 18), (16, 8, 18), (2, 9, 18), (3, 9, 18), (4, 9, 18), (5, 9, 18), (6, 9, 18), (7, 9, 18), (16, 9, 18), (4, 10, 18), (5, 10, 18), (6, 10, 18), (2, 11, 18), (3, 11, 18), (4, 11, 18), (5, 11, 18), (6, 11, 18), (3, 12, 18), (4, 12, 18), (2, 13, 18), (3, 13, 18), (4, 13, 18), (4, 14, 18), (5, 14, 18), (3, 15, 18), (4, 15, 18), (5, 15, 18), (5, 16, 18), (4, 17, 18), (5, 17, 18), (5, 18, 18)],
        "Shaved Head" => return vec![(8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (7, 6, 0), (8, 6, 35), (9, 6, 35), (10, 6, 35), (11, 6, 35), (12, 6, 35), (13, 6, 35), (14, 6, 35), (15, 6, 0), (6, 7, 0), (7, 7, 35), (8, 7, 35), (9, 7, 120), (10, 7, 35), (11, 7, 35), (12, 7, 35), (13, 7, 35), (14, 7, 35), (15, 7, 35), (16, 7, 0), (6, 8, 0), (7, 8, 35), (8, 8, 120), (9, 8, 35), (10, 8, 35), (11, 8, 35), (12, 8, 35), (13, 8, 35), (14, 8, 35), (15, 8, 35), (16, 8, 0), (6, 9, 0), (7, 9, 35), (15, 9, 35), (16, 9, 0)],
        "Stringy Hair" => return vec![(8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (7, 6, 0), (9, 6, 0), (11, 6, 0), (13, 6, 0), (15, 6, 0), (6, 7, 0), (9, 7, 0), (11, 7, 0), (13, 7, 0), (16, 7, 0), (6, 8, 0), (8, 8, 0), (10, 8, 0), (14, 8, 0), (16, 8, 0), (6, 9, 0), (8, 9, 0), (11, 9, 0), (14, 9, 0), (16, 9, 0)],
        "Top Hat" => return vec![(7, 1, 0), (8, 1, 0), (9, 1, 0), (10, 1, 0), (11, 1, 0), (12, 1, 0), (13, 1, 0), (14, 1, 0), (15, 1, 0), (6, 2, 0), (7, 2, 0), (8, 2, 0), (9, 2, 0), (10, 2, 0), (11, 2, 0), (12, 2, 0), (13, 2, 0), (14, 2, 0), (15, 2, 0), (16, 2, 0), (6, 3, 0), (7, 3, 0), (8, 3, 0), (9, 3, 0), (10, 3, 0), (11, 3, 0), (12, 3, 0), (13, 3, 0), (14, 3, 0), (15, 3, 0), (16, 3, 0), (6, 4, 0), (7, 4, 0), (8, 4, 0), (9, 4, 0), (10, 4, 0), (11, 4, 0), (12, 4, 0), (13, 4, 0), (14, 4, 0), (15, 4, 0), (16, 4, 0), (6, 5, 0), (7, 5, 0), (8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (15, 5, 0), (16, 5, 0), (6, 6, 61), (7, 6, 61), (8, 6, 61), (9, 6, 61), (10, 6, 61), (11, 6, 61), (12, 6, 61), (13, 6, 61), (14, 6, 61), (15, 6, 61), (16, 6, 61), (5, 7, 0), (6, 7, 0), (7, 7, 0), (8, 7, 0), (9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (15, 7, 0), (16, 7, 0), (17, 7, 0), (4, 8, 0), (5, 8, 0), (6, 8, 0), (7, 8, 0), (8, 8, 0), (9, 8, 0), (10, 8, 0), (11, 8, 0), (12, 8, 0), (13, 8, 0), (14, 8, 0), (15, 8, 0), (16, 8, 0), (17, 8, 0), (18, 8, 0), (16, 10, 0), (16, 11, 0), (16, 12, 0), (16, 13, 0), (16, 14, 0), (16, 15, 0), (16, 16, 0), (16, 17, 0), (16, 18, 0), (16, 19, 0)],
        "Vampire Hair" => return vec![(8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (7, 6, 0), (8, 6, 0), (9, 6, 0), (10, 6, 0), (11, 6, 0), (12, 6, 0), (13, 6, 0), (14, 6, 0), (15, 6, 0), (6, 7, 0), (7, 7, 0), (8, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (15, 7, 0), (16, 7, 0), (6, 8, 0), (7, 8, 0), (12, 8, 0), (16, 8, 0), (6, 9, 0), (7, 9, 0), (12, 9, 0), (16, 9, 0), (6, 10, 0), (16, 10, 0), (6, 11, 0), (16, 11, 0)],
        "Wild Hair" => return vec![(6, 3, 0), (7, 3, 0), (12, 3, 0), (13, 3, 0), (14, 3, 0), (15, 3, 0), (16, 3, 0), (3, 4, 0), (6, 4, 0), (7, 4, 0), (8, 4, 0), (9, 4, 0), (10, 4, 0), (11, 4, 0), (12, 4, 0), (13, 4, 0), (14, 4, 0), (15, 4, 0), (18, 4, 0), (19, 4, 0), (3, 5, 0), (4, 5, 0), (5, 5, 0), (6, 5, 0), (7, 5, 0), (8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (15, 5, 0), (16, 5, 0), (17, 5, 0), (18, 5, 0), (3, 6, 0), (4, 6, 0), (5, 6, 0), (6, 6, 0), (7, 6, 0), (8, 6, 0), (9, 6, 0), (10, 6, 0), (11, 6, 0), (12, 6, 0), (13, 6, 0), (14, 6, 0), (15, 6, 0), (16, 6, 0), (17, 6, 0), (18, 6, 0), (19, 6, 0), (4, 7, 0), (5, 7, 0), (6, 7, 0), (7, 7, 0), (8, 7, 0), (9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (15, 7, 0), (16, 7, 0), (17, 7, 0), (18, 7, 0), (3, 8, 0), (4, 8, 0), (5, 8, 0), (6, 8, 0), (7, 8, 0), (8, 8, 0), (9, 8, 0), (10, 8, 0), (11, 8, 0), (14, 8, 0), (15, 8, 0), (16, 8, 0), (17, 8, 0), (18, 8, 0), (2, 9, 0), (3, 9, 0), (4, 9, 0), (5, 9, 0), (6, 9, 0), (7, 9, 0), (8, 9, 0), (11, 9, 0), (12, 9, 0), (15, 9, 0), (16, 9, 0), (17, 9, 0), (4, 10, 0), (5, 10, 0), (6, 10, 0), (7, 10, 0), (16, 10, 0), (17, 10, 0), (18, 10, 0), (3, 11, 0), (4, 11, 0), (5, 11, 0), (6, 11, 0), (7, 11, 0), (16, 11, 0), (17, 11, 0), (18, 11, 0), (3, 12, 0), (5, 12, 0), (6, 12, 0), (7, 12, 0), (16, 12, 0), (17, 12, 0), (4, 13, 0), (5, 13, 0), (6, 13, 0), (7, 13, 0), (16, 13, 0), (17, 13, 0), (4, 14, 0), (5, 14, 0), (6, 14, 0), (16, 14, 0), (17, 14, 0)],
        "Medical Mask" => return vec![(6, 12, 11), (7, 13, 11), (8, 14, 11), (15, 14, 11), (9, 15, 11), (10, 15, 11), (11, 15, 11), (12, 15, 58), (13, 15, 11), (14, 15, 11), (9, 16, 11), (10, 16, 11), (11, 16, 11), (12, 16, 11), (13, 16, 11), (14, 16, 11), (9, 17, 58), (10, 17, 11), (11, 17, 11), (12, 17, 11), (13, 17, 11), (14, 17, 58), (7, 18, 11), (8, 18, 11), (9, 18, 11), (10, 18, 11), (11, 18, 11), (12, 18, 11), (13, 18, 11), (14, 18, 11), (15, 18, 11), (9, 19, 11), (10, 19, 11), (11, 19, 11), (12, 19, 11), (13, 19, 11), (14, 19, 11), (10, 20, 11), (11, 20, 11), (12, 20, 11), (13, 20, 11)],
        "Buck Teeth" => return vec![(11, 18, 12), (12, 18, 0), (13, 18, 12)],
        "Frown" => return vec![(11, 18, 0), (12, 18, 0), (13, 18, 0), (10, 19, 0)],
        "Smile" => return vec![(10, 17, 0), (11, 18, 0), (12, 18, 0), (13, 18, 0)],
        "Cigarette" => return vec![(19, 10, 59), (19, 11, 59), (19, 12, 59), (19, 13, 59), (19, 14, 59), (19, 15, 59), (14, 17, 0), (15, 17, 0), (16, 17, 0), (17, 17, 0), (18, 17, 0), (19, 17, 0), (13, 18, 0), (14, 18, 64), (15, 18, 64), (16, 18, 64), (17, 18, 64), (18, 18, 64), (19, 18, 121), (20, 18, 0), (14, 19, 0), (15, 19, 0), (16, 19, 0), (17, 19, 0), (18, 19, 0), (19, 19, 0)],
        "Pipe" => return vec![(20, 11, 41), (19, 12, 41), (20, 12, 41), (21, 12, 41), (19, 13, 41), (20, 13, 41), (21, 13, 41), (20, 15, 41), (20, 17, 41), (14, 18, 0), (13, 19, 0), (14, 19, 37), (15, 19, 0), (18, 19, 0), (19, 19, 0), (20, 19, 0), (21, 19, 0), (22, 19, 0), (14, 20, 0), (15, 20, 37), (16, 20, 0), (18, 20, 0), (19, 20, 37), (20, 20, 37), (21, 20, 37), (22, 20, 0), (15, 21, 0), (16, 21, 37), (17, 21, 0), (18, 21, 0), (19, 21, 83), (20, 21, 37), (21, 21, 83), (22, 21, 0), (16, 22, 0), (17, 22, 37), (18, 22, 37), (19, 22, 37), (20, 22, 83), (21, 22, 0), (17, 23, 0), (18, 23, 0), (19, 23, 0), (20, 23, 0)],
        "Vape" => return vec![(14, 17, 0), (15, 17, 0), (16, 17, 0), (17, 17, 0), (18, 17, 0), (19, 17, 0), (20, 17, 0), (13, 18, 0), (14, 18, 65), (15, 18, 65), (16, 18, 65), (17, 18, 65), (18, 18, 65), (19, 18, 122), (20, 18, 0), (14, 19, 0), (15, 19, 0), (16, 19, 0), (17, 19, 0), (18, 19, 0), (19, 19, 0), (20, 19, 0)],
        "Gold Chain" => return vec![(7, 20, 84), (8, 21, 84), (9, 22, 84)],
        "Silver Chain" => return vec![(7, 22, 85), (8, 22, 85), (9, 22, 85)],
        "Clown Nose" => return vec![(12, 14, 73), (13, 14, 73), (12, 15, 73), (13, 15, 73)],
        "0" => return vec![(8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (7, 6, 0), (8, 6, 4), (9, 6, 4), (10, 6, 4), (11, 6, 4), (12, 6, 4), (13, 6, 4), (14, 6, 4), (15, 6, 0), (6, 7, 0), (7, 7, 4), (8, 7, 4), (9, 7, 12), (10, 7, 4), (11, 7, 4), (12, 7, 4), (13, 7, 4), (14, 7, 4), (15, 7, 4), (16, 7, 0), (6, 8, 0), (7, 8, 4), (8, 8, 12), (9, 8, 4), (10, 8, 4), (11, 8, 4), (12, 8, 4), (13, 8, 4), (14, 8, 4), (15, 8, 4), (16, 8, 0), (6, 9, 0), (7, 9, 4), (8, 9, 4), (9, 9, 4), (10, 9, 4), (11, 9, 4), (12, 9, 4), (13, 9, 4), (14, 9, 4), (15, 9, 4), (16, 9, 0), (6, 10, 0), (7, 10, 4), (8, 10, 4), (9, 10, 4), (10, 10, 4), (11, 10, 4), (12, 10, 4), (13, 10, 4), (14, 10, 4), (15, 10, 4), (16, 10, 0), (6, 11, 0), (7, 11, 4), (8, 11, 4), (9, 11, 74), (10, 11, 74), (11, 11, 4), (12, 11, 4), (13, 11, 4), (14, 11, 74), (15, 11, 74), (16, 11, 0), (5, 12, 0), (6, 12, 4), (7, 12, 4), (8, 12, 4), (9, 12, 0), (10, 12, 96), (11, 12, 4), (12, 12, 4), (13, 12, 4), (14, 12, 0), (15, 12, 96), (16, 12, 0), (5, 13, 0), (6, 13, 4), (7, 13, 4), (8, 13, 4), (9, 13, 4), (10, 13, 4), (11, 13, 4), (12, 13, 4), (13, 13, 4), (14, 13, 4), (15, 13, 4), (16, 13, 0), (5, 14, 0), (6, 14, 0), (7, 14, 4), (8, 14, 4), (9, 14, 4), (10, 14, 4), (11, 14, 4), (12, 14, 4), (13, 14, 4), (14, 14, 4), (15, 14, 4), (16, 14, 0), (6, 15, 0), (7, 15, 4), (8, 15, 4), (9, 15, 4), (10, 15, 4), (11, 15, 4), (12, 15, 0), (13, 15, 0), (14, 15, 4), (15, 15, 4), (16, 15, 0), (6, 16, 0), (7, 16, 4), (8, 16, 4), (9, 16, 4), (10, 16, 4), (11, 16, 4), (12, 16, 4), (13, 16, 4), (14, 16, 4), (15, 16, 4), (16, 16, 0), (6, 17, 0), (7, 17, 4), (8, 17, 4), (9, 17, 4), (10, 17, 4), (11, 17, 4), (12, 17, 4), (13, 17, 4), (14, 17, 4), (15, 17, 4), (16, 17, 0), (6, 18, 0), (7, 18, 4), (8, 18, 4), (9, 18, 4), (10, 18, 4), (11, 18, 0), (12, 18, 0), (13, 18, 0), (14, 18, 4), (15, 18, 4), (16, 18, 0), (6, 19, 0), (7, 19, 4), (8, 19, 4), (9, 19, 4), (10, 19, 4), (11, 19, 4), (12, 19, 4), (13, 19, 4), (14, 19, 4), (15, 19, 4), (16, 19, 0), (6, 20, 0), (7, 20, 4), (8, 20, 4), (9, 20, 4), (10, 20, 4), (11, 20, 114), (12, 20, 114), (13, 20, 114), (14, 20, 4), (15, 20, 0), (6, 21, 0), (7, 21, 4), (8, 21, 4), (9, 21, 4), (10, 21, 0), (11, 21, 0), (12, 21, 0), (13, 21, 0), (14, 21, 0), (6, 22, 0), (7, 22, 4), (8, 22, 4), (9, 22, 4), (10, 22, 0), (6, 23, 0), (7, 23, 4), (8, 23, 4), (9, 23, 4), (10, 23, 0)],
        "1" => return vec![(8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 132), (12, 5, 0), (13, 5, 0), (14, 5, 0), (7, 6, 0), (8, 6, 7), (9, 6, 7), (10, 6, 75), (11, 6, 75), (12, 6, 75), (13, 6, 75), (14, 6, 7), (15, 6, 0), (6, 7, 0), (7, 7, 7), (8, 7, 7), (9, 7, 81), (10, 7, 7), (11, 7, 7), (12, 7, 7), (13, 7, 7), (14, 7, 7), (15, 7, 7), (16, 7, 0), (6, 8, 0), (7, 8, 7), (8, 8, 81), (9, 8, 7), (10, 8, 7), (11, 8, 7), (12, 8, 7), (13, 8, 7), (14, 8, 7), (15, 8, 7), (16, 8, 0), (6, 9, 0), (7, 9, 7), (8, 9, 7), (9, 9, 7), (10, 9, 7), (11, 9, 7), (12, 9, 7), (13, 9, 7), (14, 9, 7), (15, 9, 7), (16, 9, 0), (6, 10, 0), (7, 10, 7), (8, 10, 7), (9, 10, 7), (10, 10, 7), (11, 10, 7), (12, 10, 7), (13, 10, 7), (14, 10, 7), (15, 10, 7), (16, 10, 0), (6, 11, 0), (7, 11, 7), (8, 11, 7), (9, 11, 8), (10, 11, 8), (11, 11, 7), (12, 11, 7), (13, 11, 7), (14, 11, 8), (15, 11, 8), (16, 11, 0), (5, 12, 0), (6, 12, 7), (7, 12, 7), (8, 12, 7), (9, 12, 0), (10, 12, 97), (11, 12, 7), (12, 12, 7), (13, 12, 7), (14, 12, 0), (15, 12, 97), (16, 12, 0), (5, 13, 0), (6, 13, 7), (7, 13, 7), (8, 13, 7), (9, 13, 7), (10, 13, 7), (11, 13, 7), (12, 13, 7), (13, 13, 7), (14, 13, 7), (15, 13, 7), (16, 13, 0), (5, 14, 0), (6, 14, 0), (7, 14, 7), (8, 14, 7), (9, 14, 7), (10, 14, 7), (11, 14, 7), (12, 14, 7), (13, 14, 7), (14, 14, 7), (15, 14, 7), (16, 14, 0), (6, 15, 0), (7, 15, 7), (8, 15, 7), (9, 15, 7), (10, 15, 7), (11, 15, 7), (12, 15, 0), (13, 15, 0), (14, 15, 7), (15, 15, 7), (16, 15, 0), (6, 16, 0), (7, 16, 7), (8, 16, 7), (9, 16, 7), (10, 16, 7), (11, 16, 7), (12, 16, 7), (13, 16, 7), (14, 16, 7), (15, 16, 7), (16, 16, 0), (6, 17, 0), (7, 17, 7), (8, 17, 7), (9, 17, 7), (10, 17, 7), (11, 17, 7), (12, 17, 7), (13, 17, 7), (14, 17, 75), (15, 17, 75), (16, 17, 0), (6, 18, 0), (7, 18, 7), (8, 18, 7), (9, 18, 7), (10, 18, 7), (11, 18, 0), (12, 18, 0), (13, 18, 0), (14, 18, 123), (15, 18, 123), (16, 18, 133), (6, 19, 0), (7, 19, 7), (8, 19, 7), (9, 19, 7), (10, 19, 7), (11, 19, 7), (12, 19, 7), (13, 19, 7), (14, 19, 75), (15, 19, 75), (16, 19, 0), (6, 20, 0), (7, 20, 7), (8, 20, 7), (9, 20, 7), (10, 20, 7), (11, 20, 7), (12, 20, 7), (13, 20, 7), (14, 20, 7), (15, 20, 0), (6, 21, 0), (7, 21, 7), (8, 21, 7), (9, 21, 7), (10, 21, 0), (11, 21, 0), (12, 21, 0), (13, 21, 0), (14, 21, 0), (6, 22, 0), (7, 22, 7), (8, 22, 7), (9, 22, 7), (10, 22, 0), (6, 23, 0), (7, 23, 7), (8, 23, 7), (9, 23, 7), (10, 23, 0)],
        "2" => return vec![(8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (7, 6, 0), (8, 6, 2), (9, 6, 2), (10, 6, 2), (11, 6, 2), (12, 6, 2), (13, 6, 2), (14, 6, 2), (15, 6, 0), (6, 7, 0), (7, 7, 2), (8, 7, 2), (9, 7, 124), (10, 7, 2), (11, 7, 2), (12, 7, 2), (13, 7, 2), (14, 7, 2), (15, 7, 2), (16, 7, 0), (6, 8, 0), (7, 8, 2), (8, 8, 124), (9, 8, 2), (10, 8, 2), (11, 8, 2), (12, 8, 2), (13, 8, 2), (14, 8, 2), (15, 8, 2), (16, 8, 0), (6, 9, 0), (7, 9, 2), (8, 9, 2), (9, 9, 2), (10, 9, 2), (11, 9, 2), (12, 9, 2), (13, 9, 2), (14, 9, 2), (15, 9, 2), (16, 9, 0), (6, 10, 0), (7, 10, 2), (8, 10, 2), (9, 10, 2), (10, 10, 2), (11, 10, 2), (12, 10, 2), (13, 10, 2), (14, 10, 2), (15, 10, 2), (16, 10, 0), (6, 11, 0), (7, 11, 2), (8, 11, 2), (9, 11, 98), (10, 11, 98), (11, 11, 2), (12, 11, 2), (13, 11, 2), (14, 11, 98), (15, 11, 98), (16, 11, 0), (5, 12, 0), (6, 12, 2), (7, 12, 2), (8, 12, 2), (9, 12, 0), (10, 12, 71), (11, 12, 2), (12, 12, 2), (13, 12, 2), (14, 12, 0), (15, 12, 71), (16, 12, 0), (5, 13, 0), (6, 13, 2), (7, 13, 2), (8, 13, 2), (9, 13, 2), (10, 13, 2), (11, 13, 2), (12, 13, 2), (13, 13, 2), (14, 13, 2), (15, 13, 2), (16, 13, 0), (5, 14, 0), (6, 14, 0), (7, 14, 2), (8, 14, 2), (9, 14, 2), (10, 14, 2), (11, 14, 2), (12, 14, 2), (13, 14, 2), (14, 14, 2), (15, 14, 2), (16, 14, 0), (6, 15, 0), (7, 15, 2), (8, 15, 2), (9, 15, 2), (10, 15, 2), (11, 15, 2), (12, 15, 0), (13, 15, 0), (14, 15, 2), (15, 15, 2), (16, 15, 0), (6, 16, 0), (7, 16, 2), (8, 16, 2), (9, 16, 2), (10, 16, 2), (11, 16, 2), (12, 16, 2), (13, 16, 2), (14, 16, 2), (15, 16, 2), (16, 16, 0), (6, 17, 0), (7, 17, 2), (8, 17, 2), (9, 17, 2), (10, 17, 2), (11, 17, 2), (12, 17, 2), (13, 17, 2), (14, 17, 2), (15, 17, 2), (16, 17, 0), (6, 18, 0), (7, 18, 2), (8, 18, 2), (9, 18, 2), (10, 18, 2), (11, 18, 0), (12, 18, 0), (13, 18, 0), (14, 18, 2), (15, 18, 2), (16, 18, 0), (6, 19, 0), (7, 19, 2), (8, 19, 2), (9, 19, 2), (10, 19, 2), (11, 19, 2), (12, 19, 2), (13, 19, 2), (14, 19, 2), (15, 19, 2), (16, 19, 0), (6, 20, 0), (7, 20, 2), (8, 20, 2), (9, 20, 2), (10, 20, 2), (11, 20, 2), (12, 20, 2), (13, 20, 2), (14, 20, 2), (15, 20, 0), (6, 21, 0), (7, 21, 2), (8, 21, 2), (9, 21, 2), (10, 21, 0), (11, 21, 86), (12, 21, 86), (13, 21, 86), (14, 21, 0), (6, 22, 0), (7, 22, 2), (8, 22, 2), (9, 22, 2), (10, 22, 0), (6, 23, 0), (7, 23, 2), (8, 23, 2), (9, 23, 2), (10, 23, 0)],
        "3" => return vec![(8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (7, 6, 0), (8, 6, 3), (9, 6, 3), (10, 6, 3), (11, 6, 3), (12, 6, 3), (13, 6, 3), (14, 6, 3), (15, 6, 0), (6, 7, 0), (7, 7, 3), (8, 7, 3), (9, 7, 115), (10, 7, 3), (11, 7, 3), (12, 7, 3), (13, 7, 3), (14, 7, 3), (15, 7, 3), (16, 7, 0), (6, 8, 0), (7, 8, 3), (8, 8, 115), (9, 8, 3), (10, 8, 3), (11, 8, 3), (12, 8, 3), (13, 8, 3), (14, 8, 3), (15, 8, 3), (16, 8, 0), (6, 9, 0), (7, 9, 3), (8, 9, 3), (9, 9, 3), (10, 9, 3), (11, 9, 3), (12, 9, 3), (13, 9, 3), (14, 9, 3), (15, 9, 3), (16, 9, 0), (6, 10, 0), (7, 10, 3), (8, 10, 3), (9, 10, 3), (10, 10, 3), (11, 10, 3), (12, 10, 3), (13, 10, 3), (14, 10, 3), (15, 10, 3), (16, 10, 0), (6, 11, 0), (7, 11, 3), (8, 11, 3), (9, 11, 99), (10, 11, 99), (11, 11, 3), (12, 11, 3), (13, 11, 3), (14, 11, 99), (15, 11, 99), (16, 11, 0), (5, 12, 0), (6, 12, 3), (7, 12, 3), (8, 12, 3), (9, 12, 0), (10, 12, 125), (11, 12, 3), (12, 12, 3), (13, 12, 3), (14, 12, 0), (15, 12, 125), (16, 12, 0), (5, 13, 0), (6, 13, 3), (7, 13, 3), (8, 13, 3), (9, 13, 3), (10, 13, 3), (11, 13, 3), (12, 13, 3), (13, 13, 3), (14, 13, 3), (15, 13, 3), (16, 13, 0), (5, 14, 0), (6, 14, 0), (7, 14, 3), (8, 14, 3), (9, 14, 3), (10, 14, 3), (11, 14, 3), (12, 14, 3), (13, 14, 3), (14, 14, 3), (15, 14, 3), (16, 14, 0), (6, 15, 0), (7, 15, 3), (8, 15, 3), (9, 15, 3), (10, 15, 3), (11, 15, 3), (12, 15, 0), (13, 15, 0), (14, 15, 3), (15, 15, 3), (16, 15, 0), (6, 16, 0), (7, 16, 3), (8, 16, 3), (9, 16, 3), (10, 16, 3), (11, 16, 3), (12, 16, 3), (13, 16, 3), (14, 16, 3), (15, 16, 3), (16, 16, 0), (6, 17, 0), (7, 17, 3), (8, 17, 3), (9, 17, 3), (10, 17, 3), (11, 17, 3), (12, 17, 3), (13, 17, 3), (14, 17, 3), (15, 17, 3), (16, 17, 0), (6, 18, 0), (7, 18, 3), (8, 18, 3), (9, 18, 3), (10, 18, 3), (11, 18, 100), (12, 18, 100), (13, 18, 100), (14, 18, 3), (15, 18, 3), (16, 18, 0), (6, 19, 0), (7, 19, 3), (8, 19, 3), (9, 19, 3), (10, 19, 3), (11, 19, 3), (12, 19, 3), (13, 19, 3), (14, 19, 3), (15, 19, 3), (16, 19, 0), (6, 20, 0), (7, 20, 3), (8, 20, 3), (9, 20, 3), (10, 20, 3), (11, 20, 3), (12, 20, 3), (13, 20, 3), (14, 20, 3), (15, 20, 0), (6, 21, 0), (7, 21, 3), (8, 21, 3), (9, 21, 3), (10, 21, 0), (11, 21, 86), (12, 21, 86), (13, 21, 86), (14, 21, 0), (6, 22, 0), (7, 22, 3), (8, 22, 3), (9, 22, 3), (10, 22, 0), (6, 23, 0), (7, 23, 3), (8, 23, 3), (9, 23, 3), (10, 23, 0)],
        "Alien" => return vec![(8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (7, 6, 0), (8, 6, 5), (9, 6, 5), (10, 6, 5), (11, 6, 5), (12, 6, 5), (13, 6, 5), (14, 6, 5), (15, 6, 0), (6, 7, 0), (7, 7, 5), (8, 7, 5), (9, 7, 101), (10, 7, 5), (11, 7, 5), (12, 7, 5), (13, 7, 5), (14, 7, 5), (15, 7, 5), (16, 7, 0), (6, 8, 0), (7, 8, 5), (8, 8, 101), (9, 8, 5), (10, 8, 5), (11, 8, 5), (12, 8, 5), (13, 8, 5), (14, 8, 5), (15, 8, 5), (16, 8, 0), (6, 9, 0), (7, 9, 5), (8, 9, 5), (9, 9, 5), (10, 9, 5), (11, 9, 5), (12, 9, 5), (13, 9, 5), (14, 9, 5), (15, 9, 5), (16, 9, 0), (6, 10, 0), (7, 10, 5), (8, 10, 5), (9, 10, 5), (10, 10, 5), (11, 10, 5), (12, 10, 5), (13, 10, 5), (14, 10, 5), (15, 10, 5), (16, 10, 0), (5, 11, 0), (6, 11, 0), (7, 11, 5), (8, 11, 5), (9, 11, 102), (10, 11, 0), (11, 11, 5), (12, 11, 5), (13, 11, 5), (14, 11, 102), (15, 11, 0), (16, 11, 0), (4, 12, 0), (5, 12, 5), (6, 12, 66), (7, 12, 5), (8, 12, 5), (9, 12, 0), (10, 12, 66), (11, 12, 5), (12, 12, 5), (13, 12, 5), (14, 12, 0), (15, 12, 66), (16, 12, 0), (5, 13, 0), (6, 13, 5), (7, 13, 5), (8, 13, 5), (9, 13, 5), (10, 13, 5), (11, 13, 5), (12, 13, 5), (13, 13, 5), (14, 13, 5), (15, 13, 5), (16, 13, 0), (5, 14, 0), (6, 14, 0), (7, 14, 5), (8, 14, 5), (9, 14, 5), (10, 14, 5), (11, 14, 5), (12, 14, 66), (13, 14, 5), (14, 14, 5), (15, 14, 5), (16, 14, 0), (6, 15, 0), (7, 15, 5), (8, 15, 5), (9, 15, 5), (10, 15, 5), (11, 15, 5), (12, 15, 66), (13, 15, 5), (14, 15, 5), (15, 15, 5), (16, 15, 0), (6, 16, 0), (7, 16, 5), (8, 16, 5), (9, 16, 5), (10, 16, 5), (11, 16, 5), (12, 16, 66), (13, 16, 5), (14, 16, 5), (15, 16, 5), (16, 16, 0), (6, 17, 0), (7, 17, 5), (8, 17, 5), (9, 17, 5), (10, 17, 5), (11, 17, 5), (12, 17, 5), (13, 17, 5), (14, 17, 5), (15, 17, 5), (16, 17, 0), (6, 18, 0), (7, 18, 5), (8, 18, 5), (9, 18, 5), (10, 18, 0), (11, 18, 0), (12, 18, 0), (13, 18, 0), (14, 18, 0), (15, 18, 5), (16, 18, 0), (6, 19, 0), (7, 19, 5), (8, 19, 5), (9, 19, 5), (10, 19, 5), (11, 19, 5), (12, 19, 5), (13, 19, 5), (14, 19, 5), (15, 19, 5), (16, 19, 0), (6, 20, 0), (7, 20, 5), (8, 20, 5), (9, 20, 5), (10, 20, 5), (11, 20, 5), (12, 20, 5), (13, 20, 5), (14, 20, 5), (15, 20, 0), (6, 21, 0), (7, 21, 5), (8, 21, 5), (9, 21, 5), (10, 21, 0), (11, 21, 0), (12, 21, 0), (13, 21, 0), (14, 21, 0), (6, 22, 0), (7, 22, 5), (8, 22, 5), (9, 22, 5), (10, 22, 0), (6, 23, 0), (7, 23, 5), (8, 23, 5), (9, 23, 5), (10, 23, 0)],
        "Green Alien" => return vec![ (8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (7, 6, 0), (8, 6, 138), (9, 6, 138), (10, 6, 138), (11, 6, 138), (12, 6, 138), (13, 6, 138), (14, 6, 138), (15, 6, 0), (6, 7, 0), (7, 7, 138), (8, 7, 138), (9, 7, 141), (10, 7, 138), (11, 7, 138), (12, 7, 138), (13, 7, 138), (14, 7, 138), (15, 7, 138), (16, 7, 0), (6, 8, 0), (7, 8, 138), (8, 8, 141), (9, 8, 138), (10, 8, 138), (11, 8, 138), (12, 8, 138), (13, 8, 138), (14, 8, 138), (15, 8, 138), (16, 8, 0), (6, 9, 0), (7, 9, 138), (8, 9, 138), (9, 9, 138), (10, 9, 138), (11, 9, 138), (12, 9, 138), (13, 9, 138), (14, 9, 138), (15, 9, 138), (16, 9, 0), (6, 10, 0), (7, 10, 138), (8, 10, 138), (9, 10, 138), (10, 10, 138), (11, 10, 138), (12, 10, 138), (13, 10, 138), (14, 10, 138), (15, 10, 138), (16, 10, 0), (5, 11, 0), (6, 11, 0), (7, 11, 138), (8, 11, 138), (9, 11, 139), (10, 11, 0), (11, 11, 138), (12, 11, 138), (13, 11, 138), (14, 11, 139), (15, 11, 0), (16, 11, 0), (4, 12, 0), (5, 12, 138), (6, 12, 140), (7, 12, 138), (8, 12, 138), (9, 12, 0), (10, 12, 140), (11, 12, 138), (12, 12, 138), (13, 12, 138), (14, 12, 0), (15, 12, 140), (16, 12, 0), (5, 13, 0), (6, 13, 138), (7, 13, 138), (8, 13, 138), (9, 13, 138), (10, 13, 138), (11, 13, 138), (12, 13, 138), (13, 13, 138), (14, 13, 138), (15, 13, 138), (16, 13, 0), (5, 14, 0), (6, 14, 0), (7, 14, 138), (8, 14, 138), (9, 14, 138), (10, 14, 138), (11, 14, 138), (12, 14, 140), (13, 14, 138), (14, 14, 138), (15, 14, 138), (16, 14, 0), (6, 15, 0), (7, 15, 138), (8, 15, 138), (9, 15, 138), (10, 15, 138), (11, 15, 138), (12, 15, 140), (13, 15, 138), (14, 15, 138), (15, 15, 138), (16, 15, 0), (6, 16, 0), (7, 16, 138), (8, 16, 138), (9, 16, 138), (10, 16, 138), (11, 16, 138), (12, 16, 140), (13, 16, 138), (14, 16, 138), (15, 16, 138), (16, 16, 0), (6, 17, 0), (7, 17, 138), (8, 17, 138), (9, 17, 138), (10, 17, 138), (11, 17, 138), (12, 17, 138), (13, 17, 138), (14, 17, 138), (15, 17, 138), (16, 17, 0), (6, 18, 0), (7, 18, 138), (8, 18, 138), (9, 18, 138), (10, 18, 0), (11, 18, 0), (12, 18, 0), (13, 18, 0), (14, 18, 0), (15, 18, 138), (16, 18, 0), (6, 19, 0), (7, 19, 138), (8, 19, 138), (9, 19, 138), (10, 19, 138), (11, 19, 138), (12, 19, 138), (13, 19, 138), (14, 19, 138), (15, 19, 138), (16, 19, 0), (6, 20, 0), (7, 20, 138), (8, 20, 138), (9, 20, 138), (10, 20, 138), (11, 20, 138), (12, 20, 138), (13, 20, 138), (14, 20, 138), (15, 20, 0), (6, 21, 0), (7, 21, 138), (8, 21, 138), (9, 21, 138), (10, 21, 0), (11, 21, 0), (12, 21, 0), (13, 21, 0), (14, 21, 0), (6, 22, 0), (7, 22, 138), (8, 22, 138), (9, 22, 138), (10, 22, 0), (6, 23, 0), (7, 23, 138), (8, 23, 138), (9, 23, 138), (10, 23, 0)],
        "Red 0 Alien" => return build_alien_asset(vec![179,181,180,178]),
        "Red 1 Alien" => return build_alien_asset(vec![182,184,183,178]),
        "Red 2 Alien" => return build_alien_asset(vec![185,187,186,178]),
        "Red 3 Alien" => return build_alien_asset(vec![188,190,189,178]),
        "Red 4 Alien" => return build_alien_asset(vec![191,193,192,178]),
        "Yellow Alien" => return build_alien_asset(vec![146,147,148,149]),
        "White Alien" => return build_alien_asset(vec![150,151,152,153]),
        "Black Alien" => return build_alien_asset(vec![154,155,156,157]),
        "Blue 0 Alien" => return build_alien_asset(vec![158,159,160,161]),
        "Blue 1 Alien" => return build_alien_asset(vec![162,163,164,165]),
        "Blue 2 Alien" => return build_alien_asset(vec![166,167,168,169]),
        "Blue 3 Alien" => return build_alien_asset(vec![170,171,172,173]),
        "Blue 4 Alien" => return build_alien_asset(vec![174,175,176,177]),
        "Ape" => return vec![(8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (7, 6, 0), (8, 6, 15), (9, 6, 15), (10, 6, 15), (11, 6, 15), (12, 6, 15), (13, 6, 15), (14, 6, 15), (15, 6, 0), (6, 7, 0), (7, 7, 15), (8, 7, 15), (9, 7, 103), (10, 7, 15), (11, 7, 15), (12, 7, 15), (13, 7, 15), (14, 7, 15), (15, 7, 15), (16, 7, 0), (6, 8, 0), (7, 8, 15), (8, 8, 103), (9, 8, 15), (10, 8, 15), (11, 8, 15), (12, 8, 15), (13, 8, 15), (14, 8, 15), (15, 8, 15), (16, 8, 0), (6, 9, 0), (7, 9, 15), (8, 9, 15), (9, 9, 0), (10, 9, 0), (11, 9, 0), (12, 9, 0), (13, 9, 0), (14, 9, 0), (15, 9, 15), (16, 9, 0), (6, 10, 0), (7, 10, 15), (8, 10, 13), (9, 10, 13), (10, 10, 13), (11, 10, 13), (12, 10, 13), (13, 10, 13), (14, 10, 13), (15, 10, 13), (16, 10, 0), (6, 11, 0), (7, 11, 15), (8, 11, 13), (9, 11, 76), (10, 11, 76), (11, 11, 13), (12, 11, 13), (13, 11, 13), (14, 11, 76), (15, 11, 76), (16, 11, 0), (5, 12, 0), (6, 12, 0), (7, 12, 15), (8, 12, 13), (9, 12, 0), (10, 12, 104), (11, 12, 13), (12, 12, 13), (13, 12, 13), (14, 12, 0), (15, 12, 104), (16, 12, 0), (5, 13, 0), (6, 13, 15), (7, 13, 15), (8, 13, 13), (9, 13, 13), (10, 13, 13), (11, 13, 13), (12, 13, 13), (13, 13, 13), (14, 13, 13), (15, 13, 13), (16, 13, 0), (5, 14, 0), (6, 14, 0), (7, 14, 15), (8, 14, 13), (9, 14, 13), (10, 14, 13), (11, 14, 13), (12, 14, 13), (13, 14, 13), (14, 14, 13), (15, 14, 13), (16, 14, 0), (6, 15, 0), (7, 15, 15), (8, 15, 15), (9, 15, 13), (10, 15, 13), (11, 15, 0), (12, 15, 13), (13, 15, 0), (14, 15, 13), (15, 15, 13), (16, 15, 0), (6, 16, 0), (7, 16, 15), (8, 16, 15), (9, 16, 15), (10, 16, 13), (11, 16, 13), (12, 16, 13), (13, 16, 13), (14, 16, 13), (15, 16, 15), (16, 16, 0), (6, 17, 0), (7, 17, 15), (8, 17, 15), (9, 17, 13), (10, 17, 13), (11, 17, 13), (12, 17, 13), (13, 17, 13), (14, 17, 13), (15, 17, 13), (16, 17, 0), (6, 18, 0), (7, 18, 15), (8, 18, 0), (9, 18, 13), (10, 18, 0), (11, 18, 0), (12, 18, 0), (13, 18, 0), (14, 18, 0), (15, 18, 13), (16, 18, 0), (6, 19, 0), (7, 19, 15), (8, 19, 0), (9, 19, 13), (10, 19, 13), (11, 19, 13), (12, 19, 13), (13, 19, 13), (14, 19, 13), (15, 19, 13), (16, 19, 0), (6, 20, 0), (7, 20, 15), (8, 20, 15), (9, 20, 0), (10, 20, 13), (11, 20, 13), (12, 20, 13), (13, 20, 13), (14, 20, 13), (15, 20, 0), (6, 21, 0), (7, 21, 15), (8, 21, 15), (9, 21, 15), (10, 21, 0), (11, 21, 0), (12, 21, 0), (13, 21, 0), (14, 21, 0), (6, 22, 0), (7, 22, 15), (8, 22, 15), (9, 22, 15), (10, 22, 0), (6, 23, 0), (7, 23, 15), (8, 23, 15), (9, 23, 15), (10, 23, 0)],
        "Zombie" => return vec![(8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (7, 6, 0), (8, 6, 6), (9, 6, 6), (10, 6, 6), (11, 6, 6), (12, 6, 6), (13, 6, 6), (14, 6, 6), (15, 6, 0), (6, 7, 0), (7, 7, 6), (8, 7, 6), (9, 7, 105), (10, 7, 6), (11, 7, 6), (12, 7, 6), (13, 7, 6), (14, 7, 6), (15, 7, 6), (16, 7, 0), (6, 8, 0), (7, 8, 6), (8, 8, 105), (9, 8, 6), (10, 8, 6), (11, 8, 6), (12, 8, 6), (13, 8, 6), (14, 8, 6), (15, 8, 6), (16, 8, 0), (6, 9, 0), (7, 9, 6), (8, 9, 6), (9, 9, 6), (10, 9, 6), (11, 9, 6), (12, 9, 6), (13, 9, 6), (14, 9, 6), (15, 9, 6), (16, 9, 0), (6, 10, 0), (7, 10, 6), (8, 10, 6), (9, 10, 6), (10, 10, 6), (11, 10, 6), (12, 10, 6), (13, 10, 6), (14, 10, 6), (15, 10, 6), (16, 10, 0), (6, 11, 0), (7, 11, 6), (8, 11, 6), (9, 11, 48), (10, 11, 48), (11, 11, 6), (12, 11, 6), (13, 11, 6), (14, 11, 48), (15, 11, 48), (16, 11, 0), (5, 12, 0), (6, 12, 6), (7, 12, 6), (8, 12, 6), (9, 12, 106), (10, 12, 0), (11, 12, 6), (12, 12, 6), (13, 12, 6), (14, 12, 106), (15, 12, 0), (16, 12, 0), (5, 13, 0), (6, 13, 6), (7, 13, 6), (8, 13, 6), (9, 13, 48), (10, 13, 6), (11, 13, 6), (12, 13, 6), (13, 13, 6), (14, 13, 48), (15, 13, 6), (16, 13, 0), (5, 14, 0), (6, 14, 0), (7, 14, 6), (8, 14, 6), (9, 14, 6), (10, 14, 6), (11, 14, 6), (12, 14, 6), (13, 14, 6), (14, 14, 6), (15, 14, 6), (16, 14, 0), (6, 15, 0), (7, 15, 6), (8, 15, 6), (9, 15, 6), (10, 15, 6), (11, 15, 6), (12, 15, 0), (13, 15, 0), (14, 15, 6), (15, 15, 6), (16, 15, 0), (6, 16, 0), (7, 16, 6), (8, 16, 6), (9, 16, 6), (10, 16, 6), (11, 16, 6), (12, 16, 6), (13, 16, 6), (14, 16, 6), (15, 16, 6), (16, 16, 0), (6, 17, 0), (7, 17, 6), (8, 17, 6), (9, 17, 6), (10, 17, 6), (11, 17, 6), (12, 17, 6), (13, 17, 6), (14, 17, 6), (15, 17, 6), (16, 17, 0), (6, 18, 0), (7, 18, 6), (8, 18, 6), (9, 18, 6), (10, 18, 6), (11, 18, 0), (12, 18, 0), (13, 18, 0), (14, 18, 6), (15, 18, 6), (16, 18, 0), (6, 19, 0), (7, 19, 6), (8, 19, 6), (9, 19, 6), (10, 19, 6), (11, 19, 48), (12, 19, 6), (13, 19, 6), (14, 19, 6), (15, 19, 6), (16, 19, 0), (6, 20, 0), (7, 20, 6), (8, 20, 6), (9, 20, 6), (10, 20, 6), (11, 20, 6), (12, 20, 6), (13, 20, 6), (14, 20, 6), (15, 20, 0), (6, 21, 0), (7, 21, 6), (8, 21, 6), (9, 21, 6), (10, 21, 0), (11, 21, 0), (12, 21, 0), (13, 21, 0), (14, 21, 0), (6, 22, 0), (7, 22, 6), (8, 22, 6), (9, 22, 6), (10, 22, 0), (6, 23, 0), (7, 23, 6), (8, 23, 6), (9, 23, 6), (10, 23, 0)],
        _ => return vec![]
    }
}

fn get_female_asset(
    asset_name: &str
) -> Vec<(u8,u8,u8)> {
    match asset_name {
        "Earring" => return vec![(6, 13, 100), (5, 14, 0), (6, 14, 89), (7, 14, 0), (6, 15, 0)],
        "Blue Eye Shadow" => return vec![(9, 12, 107), (10, 12, 107), (14, 12, 107), (15, 12, 107), (10, 13, 126), (15, 13, 126)],
        "Clown Eyes Blue" => return vec![(9, 11, 49), (14, 11, 49), (9, 12, 50), (10, 12, 50), (14, 12, 50), (15, 12, 50), (9, 13, 0), (10, 13, 79), (14, 13, 0), (15, 13, 79), (9, 14, 49), (14, 14, 49)],
        "Clown Eyes Green" => return vec![(9, 11, 9), (14, 11, 9), (9, 12, 51), (10, 12, 51), (14, 12, 51), (15, 12, 51), (9, 13, 0), (10, 13, 80), (14, 13, 0), (15, 13, 80), (9, 14, 9), (14, 14, 9)],
        "Green Eye Shadow" => return vec![(9, 12, 108), (10, 12, 108), (14, 12, 108), (15, 12, 108), (10, 13, 127), (15, 13, 127)],
        "Purple Eye Shadow" => return vec![(9, 12, 109), (10, 12, 109), (14, 12, 109), (15, 12, 109), (9, 13, 0), (10, 13, 128), (14, 13, 0), (15, 13, 128)],
        "Mole" => return vec![(9, 16, 134)],
        "Rosy Cheeks" => return vec![(9, 15, 110), (10, 15, 110), (14, 15, 110), (15, 15, 110)],
        "Spots" => return vec![(9, 9, 137), (14, 9, 41), (11, 10, 41), (7, 13, 41), (14, 14, 41), (9, 16, 41), (15, 17, 41), (12, 20, 41)],
        "3D Glasses" => return vec![(6, 11, 23), (7, 11, 23), (8, 11, 23), (9, 11, 23), (10, 11, 23), (11, 11, 23), (12, 11, 23), (13, 11, 23), (14, 11, 23), (15, 11, 23), (16, 11, 23), (7, 12, 23), (8, 12, 23), (9, 12, 52), (10, 12, 52), (11, 12, 52), (12, 12, 23), (13, 12, 53), (14, 12, 53), (15, 12, 53), (16, 12, 23), (8, 13, 23), (9, 13, 52), (10, 13, 52), (11, 13, 52), (12, 13, 23), (13, 13, 53), (14, 13, 53), (15, 13, 53), (16, 13, 23), (8, 14, 23), (9, 14, 23), (10, 14, 23), (11, 14, 23), (12, 14, 23), (13, 14, 23), (14, 14, 23), (15, 14, 23), (16, 14, 23)],
        "Big Shades" => return vec![(7, 11, 0), (8, 11, 0), (9, 11, 0), (10, 11, 0), (11, 11, 0), (13, 11, 0), (14, 11, 0), (15, 11, 0), (16, 11, 0), (17, 11, 0), (7, 12, 0), (8, 12, 54), (9, 12, 54), (10, 12, 54), (11, 12, 0), (12, 12, 0), (13, 12, 0), (14, 12, 54), (15, 12, 54), (16, 12, 54), (17, 12, 0), (6, 13, 0), (7, 13, 0), (8, 13, 55), (9, 13, 55), (10, 13, 55), (11, 13, 0), (13, 13, 0), (14, 13, 55), (15, 13, 55), (16, 13, 55), (17, 13, 0), (7, 14, 0), (8, 14, 56), (9, 14, 56), (10, 14, 56), (11, 14, 0), (13, 14, 0), (14, 14, 56), (15, 14, 56), (16, 14, 56), (17, 14, 0), (8, 15, 0), (9, 15, 0), (10, 15, 0), (14, 15, 0), (15, 15, 0), (16, 15, 0)],
        "Classic Shades" => return vec![(7, 11, 0), (8, 11, 0), (9, 11, 0), (10, 11, 0), (11, 11, 0), (12, 11, 0), (13, 11, 0), (14, 11, 0), (15, 11, 0), (16, 11, 0), (8, 12, 0), (9, 12, 69), (10, 12, 69), (11, 12, 0), (13, 12, 0), (14, 12, 69), (15, 12, 69), (16, 12, 0), (8, 13, 0), (9, 13, 70), (10, 13, 70), (11, 13, 0), (13, 13, 0), (14, 13, 70), (15, 13, 70), (16, 13, 0), (9, 14, 0), (10, 14, 0), (14, 14, 0), (15, 14, 0)],
        "Eye Mask" => return vec![(6, 11, 0), (7, 11, 0), (8, 11, 0), (9, 11, 0), (10, 11, 0), (11, 11, 0), (12, 11, 0), (13, 11, 0), (14, 11, 0), (15, 11, 0), (16, 11, 0), (6, 12, 0), (7, 12, 0), (8, 12, 0), (9, 12, 46), (10, 12, 46), (11, 12, 0), (12, 12, 0), (13, 12, 0), (14, 12, 46), (15, 12, 46), (16, 12, 0), (5, 13, 0), (6, 13, 0), (7, 13, 0), (8, 13, 0), (9, 13, 90), (10, 13, 71), (11, 13, 0), (12, 13, 0), (13, 13, 0), (14, 13, 90), (15, 13, 71), (16, 13, 0), (5, 14, 0), (6, 14, 0), (7, 14, 0), (8, 14, 0), (9, 14, 0), (10, 14, 0), (11, 14, 0), (12, 14, 0), (13, 14, 0), (14, 14, 0), (15, 14, 0), (16, 14, 0)],
        "Eye Patch" => return vec![(7, 11, 0), (8, 11, 0), (9, 11, 0), (10, 11, 0), (11, 11, 0), (12, 11, 0), (13, 11, 0), (14, 11, 0), (15, 11, 0), (16, 11, 0), (8, 12, 0), (9, 12, 0), (10, 12, 0), (11, 12, 0), (8, 13, 0), (9, 13, 0), (10, 13, 0), (11, 13, 0), (9, 14, 0), (10, 14, 0)],
        "Horned Rim Glasses" => return vec![(7, 11, 0), (8, 11, 0), (9, 11, 0), (10, 11, 0), (11, 11, 0), (12, 11, 0), (13, 11, 0), (14, 11, 0), (15, 11, 0), (16, 11, 0), (7, 12, 0), (8, 12, 41), (9, 12, 41), (10, 12, 41), (11, 12, 0), (12, 12, 0), (13, 12, 41), (14, 12, 41), (15, 12, 41), (16, 12, 0), (8, 13, 41), (9, 13, 41), (10, 13, 41), (13, 13, 41), (14, 13, 41), (15, 13, 41), (8, 14, 41), (9, 14, 41), (10, 14, 41), (13, 14, 41), (14, 14, 41), (15, 14, 41)],
        "Nerd Glasses" => return vec![(8, 11, 0), (9, 11, 0), (10, 11, 0), (11, 11, 0), (13, 11, 0), (14, 11, 0), (15, 11, 0), (16, 11, 0), (7, 12, 0), (8, 12, 0), (9, 12, 30), (10, 12, 30), (11, 12, 0), (12, 12, 0), (13, 12, 0), (14, 12, 30), (15, 12, 30), (16, 12, 0), (8, 13, 0), (9, 13, 30), (10, 13, 30), (11, 13, 0), (13, 13, 0), (14, 13, 30), (15, 13, 30), (16, 13, 0), (9, 14, 0), (10, 14, 0), (14, 14, 0), (15, 14, 0)],
        "Regular Shades" => return vec![(6, 11, 0), (7, 11, 0), (8, 11, 0), (9, 11, 0), (10, 11, 0), (11, 11, 0), (12, 11, 0), (13, 11, 0), (14, 11, 0), (15, 11, 0), (16, 11, 0), (8, 12, 0), (9, 12, 0), (10, 12, 0), (11, 12, 0), (13, 12, 0), (14, 12, 0), (15, 12, 0), (16, 12, 0), (9, 13, 0), (10, 13, 0), (14, 13, 0), (15, 13, 0)],
        "VR" => return vec![(8, 11, 0), (9, 11, 0), (10, 11, 0), (11, 11, 0), (12, 11, 0), (13, 11, 0), (14, 11, 0), (15, 11, 0), (16, 11, 0), (7, 12, 0), (8, 12, 57), (9, 12, 29), (10, 12, 29), (11, 12, 29), (12, 12, 29), (13, 12, 29), (14, 12, 29), (15, 12, 29), (16, 12, 57), (17, 12, 0), (6, 13, 0), (7, 13, 57), (8, 13, 29), (9, 13, 0), (10, 13, 0), (11, 13, 0), (12, 13, 0), (13, 13, 0), (14, 13, 0), (15, 13, 0), (16, 13, 29), (17, 13, 0), (6, 14, 0), (7, 14, 57), (8, 14, 29), (9, 14, 0), (10, 14, 0), (11, 14, 0), (12, 14, 0), (13, 14, 0), (14, 14, 0), (15, 14, 0), (16, 14, 29), (17, 14, 0), (7, 15, 0), (8, 15, 57), (9, 15, 29), (10, 15, 29), (11, 15, 29), (12, 15, 29), (13, 15, 29), (14, 15, 29), (15, 15, 29), (16, 15, 57), (17, 15, 0), (8, 16, 0), (9, 16, 0), (10, 16, 0), (11, 16, 0), (12, 16, 0), (13, 16, 0), (14, 16, 0), (15, 16, 0), (16, 16, 0)],
        "Welding Goggles" => return vec![(8, 7, 44), (9, 7, 45), (10, 7, 45), (11, 7, 44), (12, 7, 44), (13, 7, 44), (14, 7, 45), (15, 7, 45), (16, 7, 44), (7, 8, 44), (8, 8, 45), (9, 8, 30), (10, 8, 30), (11, 8, 45), (12, 8, 44), (13, 8, 45), (14, 8, 30), (15, 8, 30), (16, 8, 45), (7, 9, 44), (8, 9, 45), (9, 9, 30), (10, 9, 30), (11, 9, 45), (12, 9, 44), (13, 9, 45), (14, 9, 30), (15, 9, 30), (16, 9, 45), (7, 10, 44), (8, 10, 44), (9, 10, 45), (10, 10, 45), (11, 10, 44), (13, 10, 44), (14, 10, 45), (15, 10, 45), (16, 10, 44), (7, 11, 44), (16, 11, 44)],
        "Bandana" => return vec![(8, 6, 26), (9, 6, 26), (10, 6, 26), (11, 6, 26), (12, 6, 26), (13, 6, 26), (14, 6, 26), (15, 6, 26), (7, 7, 26), (8, 7, 24), (9, 7, 24), (10, 7, 24), (11, 7, 24), (12, 7, 24), (13, 7, 24), (14, 7, 24), (15, 7, 24), (16, 7, 26), (6, 8, 26), (7, 8, 24), (8, 8, 24), (9, 8, 24), (10, 8, 24), (11, 8, 24), (12, 8, 24), (13, 8, 24), (14, 8, 24), (15, 8, 24), (16, 8, 26), (2, 9, 24), (3, 9, 26), (4, 9, 24), (5, 9, 26), (6, 9, 82), (7, 9, 24), (8, 9, 26), (9, 9, 26), (10, 9, 26), (11, 9, 26), (12, 9, 24), (13, 9, 24), (14, 9, 24), (15, 9, 26), (3, 10, 24), (4, 10, 26), (5, 10, 82), (12, 10, 26), (13, 10, 26), (14, 10, 26), (3, 11, 24), (4, 11, 82), (3, 12, 24)],
        "Blonde Bob" => return vec![(9, 5, 1), (10, 5, 1), (11, 5, 1), (12, 5, 1), (13, 5, 1), (14, 5, 1), (7, 6, 1), (8, 6, 1), (9, 6, 1), (10, 6, 1), (11, 6, 1), (12, 6, 1), (13, 6, 1), (14, 6, 1), (15, 6, 1), (7, 7, 1), (8, 7, 1), (9, 7, 1), (10, 7, 1), (11, 7, 1), (12, 7, 1), (13, 7, 1), (14, 7, 1), (15, 7, 1), (16, 7, 1), (6, 8, 1), (7, 8, 1), (8, 8, 1), (9, 8, 1), (10, 8, 1), (11, 8, 1), (12, 8, 1), (13, 8, 1), (15, 8, 1), (16, 8, 1), (17, 8, 1), (6, 9, 1), (7, 9, 1), (9, 9, 1), (10, 9, 1), (11, 9, 1), (12, 9, 1), (16, 9, 1), (17, 9, 1), (6, 10, 1), (7, 10, 1), (11, 10, 1), (16, 10, 1), (17, 10, 1), (5, 11, 1), (6, 11, 1), (7, 11, 1), (16, 11, 1), (17, 11, 1), (5, 12, 1), (6, 12, 1), (16, 12, 1), (17, 12, 1), (5, 13, 1), (6, 13, 1), (16, 13, 1), (17, 13, 1), (5, 14, 1), (6, 14, 1), (7, 14, 1), (16, 14, 1), (17, 14, 1), (5, 15, 1), (6, 15, 1), (7, 15, 1), (16, 15, 1), (17, 15, 1), (18, 15, 1), (5, 16, 1), (6, 16, 1), (7, 16, 1), (16, 16, 1), (17, 16, 1), (18, 16, 1), (4, 17, 1), (5, 17, 1), (6, 17, 1), (7, 17, 1), (8, 17, 1), (16, 17, 1), (17, 17, 1), (18, 17, 1), (19, 17, 1), (6, 18, 1), (7, 18, 1), (8, 18, 1), (9, 18, 1), (15, 18, 1), (16, 18, 1)],
        "Blonde Short" => return vec![(9, 5, 1), (10, 5, 1), (11, 5, 1), (12, 5, 1), (13, 5, 1), (8, 6, 1), (9, 6, 1), (10, 6, 1), (11, 6, 1), (12, 6, 1), (13, 6, 1), (14, 6, 1), (7, 7, 1), (8, 7, 1), (9, 7, 1), (10, 7, 1), (11, 7, 1), (12, 7, 1), (13, 7, 1), (14, 7, 1), (15, 7, 1), (7, 8, 1), (8, 8, 1), (9, 8, 1), (10, 8, 1), (11, 8, 1), (12, 8, 1), (13, 8, 1), (14, 8, 1), (15, 8, 1), (16, 8, 1), (6, 9, 1), (7, 9, 1), (8, 9, 1), (9, 9, 1), (10, 9, 1), (11, 9, 1), (12, 9, 1), (13, 9, 1), (14, 9, 1), (15, 9, 1), (16, 9, 1), (6, 10, 1), (7, 10, 1), (8, 10, 1), (13, 10, 1), (16, 10, 1), (6, 11, 1), (7, 11, 1), (12, 11, 1), (16, 11, 1), (6, 12, 1), (16, 12, 1), (6, 13, 1), (16, 13, 1), (7, 14, 1), (15, 14, 1)],
        "Cap" => return vec![(9, 5, 14), (10, 5, 14), (11, 5, 14), (12, 5, 14), (13, 5, 14), (14, 5, 14), (15, 5, 14), (8, 6, 14), (9, 6, 14), (10, 6, 14), (11, 6, 14), (12, 6, 14), (13, 6, 14), (14, 6, 93), (15, 6, 14), (16, 6, 14), (7, 7, 14), (8, 7, 14), (9, 7, 14), (10, 7, 14), (11, 7, 14), (12, 7, 14), (13, 7, 14), (14, 7, 14), (15, 7, 93), (16, 7, 14), (7, 8, 14), (8, 8, 14), (9, 8, 14), (10, 8, 14), (11, 8, 14), (12, 8, 14), (13, 8, 14), (14, 8, 14), (15, 8, 14), (16, 8, 14), (17, 8, 14), (18, 8, 14), (19, 8, 14), (7, 9, 14), (8, 9, 14), (9, 9, 14), (10, 9, 14), (11, 9, 14), (12, 9, 14), (13, 9, 14), (14, 9, 14), (15, 9, 14), (16, 9, 14), (17, 9, 14), (18, 9, 14), (19, 9, 14), (20, 9, 14)],
        "Clown Hair Green" => return vec![(10, 3, 9), (11, 3, 9), (12, 3, 9), (7, 4, 9), (8, 4, 9), (9, 4, 9), (10, 4, 9), (11, 4, 9), (12, 4, 9), (13, 4, 9), (14, 4, 9), (15, 4, 9), (6, 5, 9), (7, 5, 9), (8, 5, 9), (9, 5, 9), (10, 5, 9), (11, 5, 9), (12, 5, 9), (13, 5, 9), (14, 5, 9), (15, 5, 9), (16, 5, 9), (5, 6, 9), (6, 6, 9), (7, 6, 9), (8, 6, 9), (9, 6, 9), (10, 6, 9), (11, 6, 9), (12, 6, 9), (13, 6, 9), (14, 6, 9), (15, 6, 9), (16, 6, 9), (17, 6, 9), (4, 7, 9), (5, 7, 9), (6, 7, 9), (7, 7, 9), (8, 7, 9), (9, 7, 9), (10, 7, 9), (11, 7, 9), (12, 7, 9), (13, 7, 9), (14, 7, 9), (15, 7, 9), (16, 7, 9), (17, 7, 9), (18, 7, 9), (4, 8, 9), (5, 8, 9), (6, 8, 9), (7, 8, 9), (8, 8, 9), (9, 8, 9), (10, 8, 9), (11, 8, 9), (12, 8, 9), (13, 8, 9), (14, 8, 9), (15, 8, 9), (16, 8, 9), (17, 8, 9), (18, 8, 9), (3, 9, 9), (4, 9, 9), (5, 9, 9), (6, 9, 9), (7, 9, 9), (8, 9, 9), (9, 9, 9), (14, 9, 9), (15, 9, 9), (16, 9, 9), (17, 9, 9), (18, 9, 9), (19, 9, 9), (3, 10, 9), (4, 10, 9), (5, 10, 9), (6, 10, 9), (7, 10, 9), (16, 10, 9), (17, 10, 9), (18, 10, 9), (19, 10, 9), (3, 11, 9), (4, 11, 9), (5, 11, 9), (6, 11, 9), (16, 11, 9), (17, 11, 9), (18, 11, 9), (19, 11, 9), (4, 12, 9), (5, 12, 9), (16, 12, 9), (17, 12, 9), (18, 12, 9), (4, 13, 9), (5, 13, 9), (17, 13, 9), (18, 13, 9), (4, 14, 9), (5, 14, 9), (18, 14, 9), (5, 15, 9), (6, 15, 9), (17, 15, 9), (6, 16, 9), (17, 16, 9)],
        "Crazy Hair" => return vec![(10, 3, 10), (12, 3, 10), (6, 4, 10), (8, 4, 10), (10, 4, 10), (12, 4, 10), (15, 4, 10), (6, 5, 10), (9, 5, 10), (10, 5, 10), (11, 5, 10), (12, 5, 10), (13, 5, 10), (14, 5, 10), (15, 5, 10), (17, 5, 10), (18, 5, 10), (19, 5, 10), (5, 6, 10), (7, 6, 10), (8, 6, 10), (9, 6, 10), (10, 6, 10), (11, 6, 10), (12, 6, 10), (13, 6, 10), (14, 6, 10), (15, 6, 10), (16, 6, 10), (5, 7, 10), (6, 7, 10), (8, 7, 10), (9, 7, 10), (10, 7, 10), (11, 7, 10), (12, 7, 10), (13, 7, 10), (14, 7, 10), (15, 7, 10), (16, 7, 10), (17, 7, 10), (18, 7, 10), (3, 8, 10), (5, 8, 10), (6, 8, 10), (7, 8, 10), (8, 8, 10), (14, 8, 10), (15, 8, 10), (16, 8, 10), (4, 9, 10), (5, 9, 10), (6, 9, 10), (7, 9, 10), (16, 9, 10), (17, 9, 10), (18, 9, 10), (19, 9, 10), (5, 10, 10), (6, 10, 10), (7, 10, 10), (16, 10, 10), (17, 10, 10), (18, 10, 10), (5, 11, 10), (6, 11, 10), (16, 11, 10), (17, 11, 10), (18, 11, 10), (19, 11, 10), (4, 12, 10), (5, 12, 10), (17, 12, 10), (18, 12, 10), (5, 13, 10), (17, 13, 10), (4, 14, 10), (5, 14, 10), (17, 14, 10), (18, 14, 10), (5, 15, 10), (6, 15, 10), (17, 15, 10), (6, 16, 10), (17, 16, 10)],
        "Dark Hair" => return vec![(9, 6, 0), (10, 6, 0), (11, 6, 0), (12, 6, 0), (13, 6, 0), (14, 6, 0), (8, 7, 0), (9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (15, 7, 0), (7, 8, 0), (8, 8, 0), (9, 8, 0), (10, 8, 0), (11, 8, 0), (12, 8, 0), (13, 8, 0), (14, 8, 0), (15, 8, 0), (16, 8, 0), (6, 9, 0), (7, 9, 0), (8, 9, 0), (9, 9, 0), (10, 9, 0), (11, 9, 0), (12, 9, 0), (13, 9, 0), (14, 9, 0), (15, 9, 0), (16, 9, 0), (17, 9, 0), (6, 10, 0), (7, 10, 0), (8, 10, 0), (9, 10, 0), (10, 10, 0), (11, 10, 0), (12, 10, 0), (13, 10, 0), (14, 10, 0), (15, 10, 0), (16, 10, 0), (17, 10, 0), (6, 11, 0), (7, 11, 0), (8, 11, 0), (11, 11, 0), (16, 11, 0), (17, 11, 0), (18, 11, 0), (5, 12, 0), (6, 12, 0), (7, 12, 0), (16, 12, 0), (17, 12, 0), (18, 12, 0), (5, 13, 0), (6, 13, 0), (7, 13, 0), (16, 13, 0), (17, 13, 0), (18, 13, 0), (5, 14, 0), (6, 14, 0), (7, 14, 0), (16, 14, 0), (17, 14, 0), (18, 14, 0), (4, 15, 0), (5, 15, 0), (6, 15, 0), (7, 15, 0), (16, 15, 0), (17, 15, 0), (18, 15, 0), (4, 16, 0), (5, 16, 0), (6, 16, 0), (7, 16, 0), (16, 16, 0), (17, 16, 0), (18, 16, 0), (4, 17, 0), (5, 17, 0), (6, 17, 0), (7, 17, 0), (16, 17, 0), (17, 17, 0), (18, 17, 0), (19, 17, 0), (3, 18, 0), (4, 18, 0), (5, 18, 0), (6, 18, 0), (7, 18, 0), (8, 18, 0), (15, 18, 0), (16, 18, 0), (17, 18, 0), (18, 18, 0), (19, 18, 0), (6, 19, 0), (7, 19, 0), (8, 19, 0), (15, 19, 0), (16, 19, 0), (17, 19, 0)],
        "Frumpy Hair" => return vec![(8, 4, 0), (9, 4, 0), (10, 4, 0), (11, 4, 0), (12, 4, 0), (13, 4, 0), (14, 4, 0), (7, 5, 0), (8, 5, 0), (9, 5, 94), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (15, 5, 0), (6, 6, 0), (7, 6, 0), (8, 6, 94), (9, 6, 0), (10, 6, 0), (11, 6, 0), (12, 6, 0), (13, 6, 0), (14, 6, 0), (15, 6, 0), (16, 6, 0), (5, 7, 0), (6, 7, 0), (7, 7, 0), (8, 7, 0), (9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (15, 7, 0), (16, 7, 0), (17, 7, 0), (5, 8, 0), (6, 8, 0), (7, 8, 0), (8, 8, 0), (9, 8, 0), (10, 8, 0), (11, 8, 0), (12, 8, 0), (13, 8, 0), (14, 8, 0), (15, 8, 0), (16, 8, 0), (17, 8, 0), (5, 9, 0), (6, 9, 0), (7, 9, 0), (8, 9, 0), (9, 9, 0), (10, 9, 0), (11, 9, 0), (12, 9, 0), (13, 9, 0), (14, 9, 0), (15, 9, 0), (16, 9, 0), (17, 9, 0), (5, 10, 0), (6, 10, 0), (7, 10, 0), (8, 10, 0), (9, 10, 0), (11, 10, 0), (12, 10, 0), (13, 10, 0), (14, 10, 0), (15, 10, 0), (16, 10, 0), (17, 10, 0), (5, 11, 0), (6, 11, 0), (7, 11, 0), (8, 11, 0), (11, 11, 0), (12, 11, 0), (13, 11, 0), (16, 11, 0), (17, 11, 0), (4, 12, 0), (5, 12, 0), (6, 12, 0), (7, 12, 0), (12, 12, 0), (16, 12, 0), (17, 12, 0), (18, 12, 0), (4, 13, 0), (5, 13, 0), (6, 13, 0), (7, 13, 0), (16, 13, 0), (17, 13, 0), (18, 13, 0), (4, 14, 0), (5, 14, 0), (6, 14, 0), (7, 14, 0), (16, 14, 0), (17, 14, 0), (18, 14, 0), (4, 15, 0), (5, 15, 0), (6, 15, 0), (7, 15, 0), (16, 15, 0), (17, 15, 0), (18, 15, 0), (5, 16, 0), (6, 16, 0), (7, 16, 0), (16, 16, 0), (17, 16, 0), (18, 16, 0), (5, 17, 0), (6, 17, 0), (7, 17, 0), (16, 17, 0), (17, 17, 0), (7, 18, 0), (16, 18, 0)],
        "Half Shaved" => return vec![(9, 5, 0), (10, 5, 0), (11, 5, 0), (7, 6, 0), (8, 6, 0), (9, 6, 0), (10, 6, 0), (11, 6, 0), (12, 6, 0), (6, 7, 0), (7, 7, 0), (8, 7, 0), (9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (5, 8, 0), (6, 8, 0), (7, 8, 0), (8, 8, 0), (9, 8, 0), (10, 8, 0), (5, 9, 0), (6, 9, 0), (7, 9, 0), (8, 9, 0), (9, 9, 0), (5, 10, 0), (6, 10, 0), (7, 10, 0), (8, 10, 0), (4, 11, 0), (5, 11, 0), (6, 11, 0), (7, 11, 0), (8, 11, 0), (4, 12, 0), (5, 12, 0), (6, 12, 0), (7, 12, 0), (8, 12, 0), (4, 13, 0), (5, 13, 0), (6, 13, 0), (7, 13, 0), (4, 14, 0), (5, 14, 0), (6, 14, 0), (7, 14, 0), (4, 15, 0), (5, 15, 0), (6, 15, 0), (7, 15, 0), (4, 16, 0), (5, 16, 0), (6, 16, 0), (7, 16, 0), (4, 17, 0), (5, 17, 0), (6, 17, 0), (7, 17, 0), (4, 18, 0), (5, 18, 0), (6, 18, 0), (7, 18, 0), (8, 18, 0), (3, 19, 0), (4, 19, 0), (5, 19, 0), (6, 19, 0), (7, 19, 0), (8, 19, 0), (3, 20, 0), (4, 20, 0), (5, 20, 0), (6, 20, 0), (7, 20, 0), (8, 20, 0), (3, 21, 0), (4, 21, 0), (5, 21, 0), (6, 21, 0), (7, 21, 0), (8, 21, 0), (8, 22, 0), (8, 23, 0)],
        "Headband" => return vec![(8, 9, 12), (9, 9, 12), (10, 9, 12), (11, 9, 12), (12, 9, 12), (13, 9, 12), (14, 9, 12), (15, 9, 12), (8, 10, 43), (9, 10, 43), (10, 10, 43), (11, 10, 43), (12, 10, 43), (13, 10, 43), (14, 10, 43), (15, 10, 43)],
        "Knitted Cap" => return vec![(9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (8, 6, 0), (9, 6, 28), (10, 6, 28), (11, 6, 28), (12, 6, 28), (13, 6, 28), (14, 6, 28), (15, 6, 0), (7, 7, 0), (8, 7, 28), (9, 7, 28), (10, 7, 28), (11, 7, 28), (12, 7, 28), (13, 7, 28), (14, 7, 28), (15, 7, 28), (16, 7, 0), (6, 8, 0), (7, 8, 33), (8, 8, 33), (9, 8, 33), (10, 8, 33), (11, 8, 33), (12, 8, 33), (13, 8, 33), (14, 8, 33), (15, 8, 33), (16, 8, 33), (17, 8, 0), (6, 9, 0), (7, 9, 33), (8, 9, 28), (9, 9, 33), (10, 9, 28), (11, 9, 33), (12, 9, 28), (13, 9, 33), (14, 9, 28), (15, 9, 33), (16, 9, 28), (17, 9, 0)],
        "Messy Hair" => return vec![(8, 4, 0), (9, 4, 0), (10, 4, 0), (11, 4, 0), (12, 4, 0), (13, 4, 0), (15, 4, 0), (7, 5, 0), (8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (15, 5, 0), (6, 6, 0), (7, 6, 0), (8, 6, 0), (9, 6, 0), (10, 6, 0), (11, 6, 0), (12, 6, 0), (13, 6, 0), (14, 6, 0), (15, 6, 0), (16, 6, 0), (17, 6, 0), (5, 7, 0), (6, 7, 0), (7, 7, 0), (8, 7, 0), (9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (15, 7, 0), (16, 7, 0), (6, 8, 0), (7, 8, 0), (8, 8, 0), (9, 8, 0), (10, 8, 0), (11, 8, 0), (12, 8, 0), (14, 8, 0), (15, 8, 0), (16, 8, 0), (5, 9, 0), (6, 9, 0), (7, 9, 0), (9, 9, 0), (12, 9, 0), (15, 9, 0), (16, 9, 0), (17, 9, 0), (7, 10, 0), (8, 10, 0), (12, 10, 0), (16, 10, 0), (7, 11, 0)],
        "Mohawk Dark" => return vec![(13, 3, 0), (14, 3, 0), (12, 4, 0), (13, 4, 0), (14, 4, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (10, 6, 0), (11, 6, 0), (12, 6, 0), (13, 6, 0), (14, 6, 0), (9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (12, 8, 0), (13, 8, 0), (13, 9, 0)],
        "Mohawk Thin" => return vec![(12, 3, 0), (11, 4, 0), (12, 4, 17), (13, 4, 0), (11, 5, 0), (12, 5, 17), (13, 5, 0), (11, 6, 0), (12, 6, 17), (13, 6, 0), (9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 17), (13, 7, 0), (14, 7, 0), (11, 8, 0), (12, 8, 17), (13, 8, 0)],
        "Mohawk" => return vec![(13, 3, 0), (14, 3, 0), (12, 4, 0), (13, 4, 8), (14, 4, 0), (11, 5, 0), (12, 5, 8), (13, 5, 8), (14, 5, 0), (10, 6, 0), (11, 6, 95), (12, 6, 8), (13, 6, 8), (14, 6, 0), (9, 7, 0), (10, 7, 95), (11, 7, 8), (12, 7, 8), (13, 7, 8), (14, 7, 0), (12, 8, 8), (13, 8, 8), (13, 9, 8)],
        "Orange Side" => return vec![(10, 6, 19), (11, 6, 19), (12, 6, 19), (13, 6, 19), (8, 7, 19), (9, 7, 19), (10, 7, 19), (11, 7, 19), (12, 7, 19), (13, 7, 19), (14, 7, 19), (15, 7, 19), (7, 8, 19), (8, 8, 19), (9, 8, 19), (10, 8, 19), (11, 8, 19), (12, 8, 19), (13, 8, 19), (14, 8, 19), (15, 8, 19), (16, 8, 19), (7, 9, 19), (8, 9, 19), (9, 9, 19), (10, 9, 19), (11, 9, 19), (12, 9, 19), (13, 9, 19), (14, 9, 19), (15, 9, 19), (16, 9, 19), (7, 10, 19), (8, 10, 19), (13, 10, 19), (15, 10, 19), (16, 10, 19), (17, 10, 19), (6, 11, 19), (7, 11, 19), (16, 11, 19), (17, 11, 19), (6, 12, 19), (7, 12, 19), (16, 12, 19), (17, 12, 19), (6, 13, 19), (16, 13, 19), (17, 13, 19), (6, 14, 19), (16, 14, 19), (17, 14, 19), (16, 15, 19), (17, 15, 19), (18, 15, 19), (16, 16, 19), (17, 16, 19), (18, 16, 19), (16, 17, 19), (17, 17, 19), (18, 17, 19), (16, 18, 19), (17, 18, 19), (18, 18, 19), (16, 19, 19), (17, 19, 19), (18, 19, 19), (15, 20, 19), (16, 20, 19), (17, 20, 19), (18, 20, 19), (14, 21, 19), (15, 21, 19), (16, 21, 19), (17, 21, 19), (18, 21, 19), (14, 22, 19), (15, 22, 19), (16, 22, 19)],
        "Pigtails" => return vec![(9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (4, 6, 0), (5, 6, 0), (8, 6, 0), (9, 6, 0), (10, 6, 0), (11, 6, 0), (12, 6, 0), (13, 6, 0), (14, 6, 0), (15, 6, 0), (18, 6, 0), (19, 6, 0), (3, 7, 0), (4, 7, 0), (5, 7, 0), (6, 7, 0), (7, 7, 89), (8, 7, 0), (9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (15, 7, 0), (16, 7, 89), (17, 7, 0), (18, 7, 0), (19, 7, 0), (20, 7, 0), (2, 8, 0), (3, 8, 0), (4, 8, 0), (5, 8, 0), (6, 8, 0), (7, 8, 0), (8, 8, 0), (9, 8, 0), (10, 8, 0), (11, 8, 0), (12, 8, 0), (13, 8, 0), (14, 8, 0), (15, 8, 0), (16, 8, 0), (17, 8, 0), (18, 8, 0), (19, 8, 0), (20, 8, 0), (21, 8, 0), (2, 9, 0), (3, 9, 0), (4, 9, 0), (5, 9, 0), (7, 9, 0), (8, 9, 0), (9, 9, 0), (15, 9, 0), (16, 9, 0), (18, 9, 0), (19, 9, 0), (20, 9, 0), (21, 9, 0), (2, 10, 0), (3, 10, 0), (4, 10, 0), (7, 10, 0), (8, 10, 0), (16, 10, 0), (19, 10, 0), (20, 10, 0), (21, 10, 0), (2, 11, 0), (3, 11, 0), (4, 11, 0), (7, 11, 0), (16, 11, 0), (19, 11, 0), (20, 11, 0), (21, 11, 0), (3, 12, 0), (20, 12, 0)],
        "Pilot Helmet" => return vec![(9, 5, 27), (10, 5, 27), (11, 5, 27), (12, 5, 27), (13, 5, 27), (14, 5, 27), (8, 6, 0), (9, 6, 0), (10, 6, 0), (11, 6, 0), (12, 6, 0), (13, 6, 0), (14, 6, 0), (15, 6, 0), (7, 7, 0), (8, 7, 30), (9, 7, 30), (10, 7, 30), (11, 7, 0), (12, 7, 0), (13, 7, 30), (14, 7, 30), (15, 7, 30), (16, 7, 0), (7, 8, 0), (8, 8, 30), (9, 8, 30), (10, 8, 0), (11, 8, 0), (12, 8, 0), (13, 8, 0), (14, 8, 30), (15, 8, 30), (16, 8, 0), (7, 9, 0), (8, 9, 0), (9, 9, 0), (10, 9, 0), (11, 9, 27), (12, 9, 27), (13, 9, 0), (14, 9, 0), (15, 9, 0), (16, 9, 0), (7, 10, 27), (8, 10, 27), (9, 10, 27), (10, 10, 27), (11, 10, 27), (12, 10, 27), (13, 10, 27), (14, 10, 27), (15, 10, 27), (16, 10, 27), (6, 11, 27), (7, 11, 27), (16, 11, 27), (6, 12, 27), (7, 12, 27), (16, 12, 27), (6, 13, 27), (7, 13, 27), (16, 13, 27), (6, 14, 27), (7, 14, 27), (16, 14, 27), (7, 15, 27), (16, 15, 27), (7, 16, 27), (16, 16, 27), (7, 17, 27), (16, 17, 27), (7, 18, 27), (16, 18, 27), (7, 19, 27), (16, 19, 27), (7, 20, 27), (16, 20, 27)],
        "Pink With Hat" => return vec![(9, 6, 0), (10, 6, 0), (11, 6, 0), (12, 6, 0), (13, 6, 0), (14, 6, 0), (8, 7, 0), (9, 7, 0), (10, 7, 0), (11, 7, 12), (12, 7, 0), (13, 7, 0), (14, 7, 0), (15, 7, 0), (7, 8, 0), (8, 8, 0), (9, 8, 0), (10, 8, 0), (11, 8, 0), (12, 8, 12), (13, 8, 0), (14, 8, 0), (15, 8, 0), (16, 8, 0), (7, 9, 0), (8, 9, 0), (9, 9, 0), (10, 9, 0), (11, 9, 0), (12, 9, 0), (13, 9, 0), (14, 9, 0), (15, 9, 0), (16, 9, 0), (7, 10, 21), (8, 10, 21), (9, 10, 21), (10, 10, 21), (11, 10, 21), (12, 10, 21), (13, 10, 21), (14, 10, 21), (15, 10, 21), (16, 10, 21), (6, 11, 21), (7, 11, 21), (8, 11, 21), (11, 11, 21), (12, 11, 21), (13, 11, 21), (16, 11, 21), (17, 11, 21), (6, 12, 21), (7, 12, 21), (12, 12, 21), (16, 12, 21), (17, 12, 21), (5, 13, 21), (6, 13, 21), (16, 13, 21), (17, 13, 21), (18, 13, 21), (5, 14, 21), (6, 14, 21), (7, 14, 21), (16, 14, 21), (17, 14, 21), (18, 14, 21), (5, 15, 21), (6, 15, 21), (7, 15, 21), (16, 15, 21), (17, 15, 21), (18, 15, 21), (5, 16, 21), (6, 16, 21), (7, 16, 21), (16, 16, 21), (17, 16, 21), (18, 16, 21), (5, 17, 21), (6, 17, 21), (7, 17, 21), (16, 17, 21), (17, 17, 21), (18, 17, 21), (6, 18, 21), (7, 18, 21), (16, 18, 21), (17, 18, 21), (6, 19, 21), (7, 19, 21), (8, 19, 21), (15, 19, 21), (16, 19, 21), (17, 19, 21), (7, 20, 21), (8, 20, 21), (15, 20, 21), (16, 20, 21)],
        "Red Mohawk" => return vec![(12, 4, 10), (11, 5, 10), (12, 5, 10), (10, 6, 10), (11, 6, 10), (12, 6, 10), (11, 7, 10), (12, 7, 10)],
        "Straight Hair Blonde" => return vec![(9, 7, 1), (10, 7, 1), (11, 7, 1), (12, 7, 1), (13, 7, 1), (14, 7, 1), (8, 8, 1), (9, 8, 1), (10, 8, 1), (11, 8, 1), (12, 8, 1), (13, 8, 1), (14, 8, 1), (15, 8, 1), (7, 9, 1), (8, 9, 1), (9, 9, 1), (10, 9, 1), (11, 9, 1), (12, 9, 1), (13, 9, 1), (14, 9, 1), (15, 9, 1), (16, 9, 1), (6, 10, 1), (7, 10, 1), (8, 10, 1), (9, 10, 1), (15, 10, 1), (16, 10, 1), (6, 11, 1), (7, 11, 1), (8, 11, 1), (16, 11, 1), (6, 12, 1), (7, 12, 1), (8, 12, 1), (16, 12, 1), (5, 13, 1), (6, 13, 1), (7, 13, 1), (16, 13, 1), (5, 14, 1), (6, 14, 1), (7, 14, 1), (16, 14, 1), (5, 15, 1), (6, 15, 1), (7, 15, 1), (16, 15, 1), (5, 16, 1), (6, 16, 1), (7, 16, 1), (16, 16, 1), (5, 17, 1), (6, 17, 1), (7, 17, 1), (16, 17, 1), (5, 18, 1), (6, 18, 1), (7, 18, 1), (16, 18, 1), (5, 19, 1), (6, 19, 1), (7, 19, 1), (16, 19, 1), (5, 20, 1), (6, 20, 1), (7, 20, 1), (15, 20, 1), (16, 20, 1), (5, 21, 1), (6, 21, 1), (7, 21, 1), (14, 21, 1), (15, 21, 1), (16, 21, 1), (5, 22, 1), (6, 22, 1), (7, 22, 1), (13, 22, 1), (14, 22, 1), (15, 22, 1)],
        "Straight Hair Dark" => return vec![(9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (8, 8, 0), (9, 8, 0), (10, 8, 0), (11, 8, 0), (12, 8, 0), (13, 8, 0), (14, 8, 0), (15, 8, 0), (7, 9, 0), (8, 9, 0), (9, 9, 0), (10, 9, 0), (11, 9, 0), (12, 9, 0), (13, 9, 0), (14, 9, 0), (15, 9, 0), (16, 9, 0), (6, 10, 0), (7, 10, 0), (8, 10, 0), (9, 10, 0), (15, 10, 0), (16, 10, 0), (6, 11, 0), (7, 11, 0), (8, 11, 0), (16, 11, 0), (6, 12, 0), (7, 12, 0), (8, 12, 0), (16, 12, 0), (5, 13, 0), (6, 13, 0), (7, 13, 0), (16, 13, 0), (5, 14, 0), (6, 14, 0), (7, 14, 0), (16, 14, 0), (5, 15, 0), (6, 15, 0), (7, 15, 0), (16, 15, 0), (5, 16, 0), (6, 16, 0), (7, 16, 0), (16, 16, 0), (5, 17, 0), (6, 17, 0), (7, 17, 0), (16, 17, 0), (5, 18, 0), (6, 18, 0), (7, 18, 0), (15, 18, 0), (16, 18, 0), (5, 19, 0), (6, 19, 0), (7, 19, 0), (15, 19, 0), (16, 19, 0), (5, 20, 0), (6, 20, 0), (7, 20, 0), (15, 20, 0), (16, 20, 0), (5, 21, 0), (6, 21, 0), (7, 21, 0), (14, 21, 0), (15, 21, 0), (16, 21, 0), (5, 22, 0), (6, 22, 0), (7, 22, 0), (13, 22, 0), (14, 22, 0), (15, 22, 0)],
        "Straight Hair" => return vec![(9, 7, 16), (10, 7, 16), (11, 7, 16), (12, 7, 16), (13, 7, 16), (14, 7, 16), (8, 8, 16), (9, 8, 16), (10, 8, 16), (11, 8, 16), (12, 8, 16), (13, 8, 16), (14, 8, 16), (15, 8, 16), (7, 9, 16), (8, 9, 16), (9, 9, 16), (10, 9, 16), (11, 9, 16), (12, 9, 16), (13, 9, 16), (14, 9, 16), (15, 9, 16), (16, 9, 16), (6, 10, 16), (7, 10, 16), (8, 10, 16), (9, 10, 16), (15, 10, 16), (16, 10, 16), (6, 11, 16), (7, 11, 16), (8, 11, 16), (16, 11, 16), (6, 12, 16), (7, 12, 16), (8, 12, 16), (16, 12, 16), (5, 13, 16), (6, 13, 16), (7, 13, 16), (16, 13, 16), (5, 14, 16), (6, 14, 16), (7, 14, 16), (16, 14, 16), (5, 15, 16), (6, 15, 16), (7, 15, 16), (16, 15, 16), (5, 16, 16), (6, 16, 16), (7, 16, 16), (16, 16, 16), (5, 17, 16), (6, 17, 16), (7, 17, 16), (16, 17, 16), (5, 18, 16), (6, 18, 16), (7, 18, 16), (15, 18, 16), (16, 18, 16), (5, 19, 16), (6, 19, 16), (7, 19, 16), (15, 19, 16), (16, 19, 16), (5, 20, 16), (6, 20, 16), (7, 20, 16), (15, 20, 16), (16, 20, 16), (5, 21, 16), (6, 21, 16), (7, 21, 16), (14, 21, 16), (15, 21, 16), (16, 21, 16), (5, 22, 16), (6, 22, 16), (7, 22, 16), (13, 22, 16), (14, 22, 16), (15, 22, 16)],
        "Stringy Hair" => return vec![(9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (8, 8, 0), (10, 8, 0), (12, 8, 0), (14, 8, 0), (15, 8, 0), (7, 9, 0), (9, 9, 0), (11, 9, 0), (13, 9, 0), (15, 9, 0), (16, 9, 0), (7, 10, 0), (8, 10, 0), (10, 10, 0), (12, 10, 0), (14, 10, 0), (16, 10, 0)],
        "Tassle Hat" => return vec![(11, 2, 0), (12, 2, 0), (10, 3, 0), (11, 3, 42), (12, 3, 25), (13, 3, 0), (9, 4, 0), (10, 4, 42), (11, 4, 25), (12, 4, 42), (13, 4, 25), (14, 4, 0), (9, 5, 0), (10, 5, 25), (11, 5, 42), (12, 5, 25), (13, 5, 42), (14, 5, 0), (8, 6, 0), (9, 6, 25), (10, 6, 42), (11, 6, 25), (12, 6, 42), (13, 6, 25), (14, 6, 42), (15, 6, 0), (7, 7, 0), (8, 7, 25), (9, 7, 42), (10, 7, 25), (11, 7, 42), (12, 7, 25), (13, 7, 42), (14, 7, 25), (15, 7, 42), (16, 7, 0), (7, 8, 0), (8, 8, 42), (9, 8, 25), (10, 8, 42), (11, 8, 25), (12, 8, 42), (13, 8, 25), (14, 8, 42), (15, 8, 25), (16, 8, 0), (6, 9, 0), (7, 9, 42), (8, 9, 25), (9, 9, 25), (10, 9, 25), (11, 9, 25), (12, 9, 25), (13, 9, 25), (14, 9, 25), (15, 9, 42), (16, 9, 25), (17, 9, 0), (6, 10, 0), (7, 10, 25), (8, 10, 25), (15, 10, 25), (16, 10, 25), (17, 10, 0), (6, 11, 0), (7, 11, 25), (16, 11, 25), (17, 11, 0), (6, 12, 0), (7, 12, 25), (16, 12, 25), (17, 12, 0), (6, 13, 0), (7, 13, 25), (16, 13, 25), (17, 13, 0), (6, 14, 0), (7, 14, 25), (16, 14, 25), (17, 14, 0), (6, 15, 0), (7, 15, 25), (16, 15, 25), (17, 15, 0), (6, 16, 0), (7, 16, 25), (16, 16, 25), (17, 16, 0), (6, 17, 0), (7, 17, 25), (16, 17, 25), (17, 17, 0), (6, 18, 0), (7, 18, 25), (16, 18, 25), (17, 18, 0), (5, 19, 0), (6, 19, 25), (7, 19, 25), (8, 19, 25), (15, 19, 25), (16, 19, 25), (17, 19, 25), (18, 19, 0), (6, 20, 0), (7, 20, 25), (8, 20, 0), (15, 20, 0), (16, 20, 25), (17, 20, 0), (7, 21, 0), (16, 21, 0)],
        "Tiara" => return vec![(9, 8, 67), (10, 8, 67), (11, 8, 67), (13, 8, 67), (14, 8, 67), (9, 9, 12), (12, 9, 67), (11, 10, 67), (12, 10, 135), (13, 10, 67), (12, 11, 67)],
        "Wild Blonde" => return vec![(11, 3, 1), (14, 3, 1), (15, 3, 1), (8, 4, 1), (9, 4, 1), (12, 4, 1), (13, 4, 1), (14, 4, 1), (15, 4, 1), (17, 4, 1), (6, 5, 1), (7, 5, 1), (8, 5, 1), (9, 5, 1), (10, 5, 1), (12, 5, 1), (13, 5, 1), (14, 5, 1), (15, 5, 1), (16, 5, 1), (5, 6, 1), (6, 6, 1), (7, 6, 1), (9, 6, 1), (10, 6, 1), (11, 6, 1), (12, 6, 1), (13, 6, 1), (14, 6, 1), (16, 6, 1), (17, 6, 1), (18, 6, 1), (19, 6, 1), (20, 6, 1), (4, 7, 1), (5, 7, 1), (7, 7, 1), (8, 7, 1), (9, 7, 1), (10, 7, 1), (11, 7, 1), (12, 7, 1), (13, 7, 1), (14, 7, 1), (15, 7, 1), (16, 7, 1), (17, 7, 1), (18, 7, 1), (3, 8, 1), (4, 8, 1), (5, 8, 1), (6, 8, 1), (7, 8, 1), (8, 8, 1), (9, 8, 1), (10, 8, 1), (11, 8, 1), (12, 8, 1), (13, 8, 1), (14, 8, 1), (15, 8, 1), (16, 8, 1), (17, 8, 1), (5, 9, 1), (6, 9, 1), (7, 9, 1), (8, 9, 1), (9, 9, 1), (14, 9, 1), (15, 9, 1), (16, 9, 1), (17, 9, 1), (18, 9, 1), (19, 9, 1), (4, 10, 1), (5, 10, 1), (6, 10, 1), (7, 10, 1), (10, 10, 1), (14, 10, 1), (16, 10, 1), (17, 10, 1), (18, 10, 1), (20, 10, 1), (3, 11, 1), (4, 11, 1), (5, 11, 1), (17, 11, 1), (18, 11, 1), (19, 11, 1), (4, 12, 1), (5, 12, 1), (17, 12, 1), (18, 12, 1), (19, 12, 1), (20, 12, 1), (3, 13, 1), (4, 13, 1), (5, 13, 1), (17, 13, 1), (18, 13, 1), (19, 13, 1), (20, 13, 1), (4, 14, 1), (5, 14, 1), (17, 14, 1), (18, 14, 1), (19, 14, 1), (3, 15, 1), (4, 15, 1), (5, 15, 1), (6, 15, 1), (17, 15, 1), (19, 15, 1), (4, 16, 1), (6, 16, 1), (7, 16, 1), (16, 16, 1), (18, 16, 1), (4, 17, 1), (5, 17, 1), (6, 17, 1), (18, 17, 1), (19, 17, 1), (5, 18, 1)],
        "Wild Hair" => return vec![(14, 2, 0), (6, 3, 0), (9, 3, 0), (13, 3, 0), (16, 3, 0), (5, 4, 0), (7, 4, 0), (9, 4, 0), (10, 4, 0), (12, 4, 0), (13, 4, 0), (14, 4, 0), (15, 4, 0), (16, 4, 0), (19, 4, 0), (3, 5, 0), (5, 5, 0), (6, 5, 0), (7, 5, 0), (8, 5, 0), (9, 5, 0), (10, 5, 0), (11, 5, 0), (12, 5, 0), (13, 5, 0), (14, 5, 0), (15, 5, 0), (16, 5, 0), (18, 5, 0), (4, 6, 0), (5, 6, 0), (6, 6, 0), (7, 6, 0), (8, 6, 0), (9, 6, 0), (10, 6, 0), (11, 6, 0), (12, 6, 0), (13, 6, 0), (14, 6, 0), (15, 6, 0), (16, 6, 0), (17, 6, 0), (18, 6, 0), (3, 7, 0), (4, 7, 0), (5, 7, 0), (6, 7, 0), (7, 7, 0), (8, 7, 0), (9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (15, 7, 0), (16, 7, 0), (17, 7, 0), (18, 7, 0), (19, 7, 0), (4, 8, 0), (5, 8, 0), (6, 8, 0), (7, 8, 0), (8, 8, 0), (9, 8, 0), (10, 8, 0), (11, 8, 0), (12, 8, 0), (13, 8, 0), (14, 8, 0), (15, 8, 0), (16, 8, 0), (17, 8, 0), (18, 8, 0), (3, 9, 0), (4, 9, 0), (5, 9, 0), (6, 9, 0), (7, 9, 0), (8, 9, 0), (9, 9, 0), (10, 9, 0), (11, 9, 0), (14, 9, 0), (15, 9, 0), (16, 9, 0), (17, 9, 0), (2, 10, 0), (3, 10, 0), (4, 10, 0), (5, 10, 0), (6, 10, 0), (7, 10, 0), (8, 10, 0), (11, 10, 0), (12, 10, 0), (15, 10, 0), (16, 10, 0), (17, 10, 0), (18, 10, 0), (4, 11, 0), (5, 11, 0), (6, 11, 0), (7, 11, 0), (16, 11, 0), (17, 11, 0), (18, 11, 0), (19, 11, 0), (3, 12, 0), (4, 12, 0), (5, 12, 0), (6, 12, 0), (7, 12, 0), (16, 12, 0), (17, 12, 0), (18, 12, 0), (3, 13, 0), (5, 13, 0), (6, 13, 0), (7, 13, 0), (16, 13, 0), (17, 13, 0), (18, 13, 0), (4, 14, 0), (5, 14, 0), (6, 14, 0), (7, 14, 0), (16, 14, 0), (17, 14, 0), (18, 14, 0), (19, 14, 0), (4, 15, 0), (6, 15, 0), (7, 15, 0), (16, 15, 0), (17, 15, 0), (6, 16, 0), (7, 16, 0), (16, 16, 0), (17, 16, 0), (5, 17, 0), (7, 17, 0), (16, 17, 0), (17, 17, 0), (7, 18, 0), (16, 18, 0), (18, 18, 0)],
        "Wild White Hair" => return vec![(10, 5, 12), (11, 5, 12), (12, 5, 12), (13, 5, 12), (15, 5, 12), (7, 6, 12), (9, 6, 12), (10, 6, 12), (11, 6, 12), (12, 6, 12), (13, 6, 12), (14, 6, 12), (7, 7, 12), (8, 7, 12), (9, 7, 12), (10, 7, 12), (11, 7, 12), (12, 7, 12), (13, 7, 12), (14, 7, 12), (15, 7, 12), (17, 7, 12), (5, 8, 12), (6, 8, 12), (7, 8, 12), (8, 8, 12), (9, 8, 12), (10, 8, 12), (11, 8, 12), (12, 8, 12), (13, 8, 12), (14, 8, 12), (15, 8, 12), (16, 8, 12), (18, 8, 12), (6, 9, 12), (7, 9, 12), (8, 9, 12), (9, 9, 12), (11, 9, 12), (12, 9, 12), (13, 9, 12), (14, 9, 12), (15, 9, 12), (16, 9, 12), (17, 9, 12), (18, 9, 12), (5, 10, 12), (6, 10, 12), (7, 10, 12), (8, 10, 12), (10, 10, 12), (12, 10, 12), (15, 10, 12), (16, 10, 12), (17, 10, 12), (4, 11, 12), (6, 11, 12), (7, 11, 12), (8, 11, 12), (12, 11, 12), (16, 11, 12), (17, 11, 12), (18, 11, 12), (5, 12, 12), (6, 12, 12), (7, 12, 12), (12, 12, 12), (17, 12, 12), (19, 12, 12), (4, 13, 12), (6, 13, 12), (13, 13, 12), (17, 13, 12), (18, 13, 12), (3, 14, 12), (4, 14, 12), (5, 14, 12), (6, 14, 12), (17, 14, 12), (18, 14, 12), (5, 15, 12), (6, 15, 12), (17, 15, 12), (18, 15, 12), (19, 15, 12), (5, 16, 12), (6, 16, 12), (17, 16, 12), (4, 17, 12), (18, 17, 12)],
        "Medical Mask" => return vec![(7, 13, 11), (8, 14, 11), (15, 14, 11), (9, 15, 11), (10, 15, 11), (11, 15, 11), (12, 15, 58), (13, 15, 11), (14, 15, 11), (9, 16, 11), (10, 16, 11), (11, 16, 11), (12, 16, 11), (13, 16, 11), (14, 16, 11), (9, 17, 58), (10, 17, 11), (11, 17, 11), (12, 17, 11), (13, 17, 11), (14, 17, 58), (8, 18, 11), (9, 18, 11), (10, 18, 11), (11, 18, 11), (12, 18, 11), (13, 18, 11), (14, 18, 11), (15, 18, 11), (9, 19, 11), (10, 19, 11), (11, 19, 11), (12, 19, 11), (13, 19, 11), (14, 19, 11), (10, 20, 11), (11, 20, 11), (12, 20, 11), (13, 20, 11)],
        "Black Lipstick" => return vec![(11, 18, 0), (12, 18, 0), (13, 18, 0)],
        "Hot Lipstick" => return vec![(11, 18, 116), (12, 18, 116), (13, 18, 116)],
        "Purple Lipstick" => return vec![(11, 18, 117), (12, 18, 117), (13, 18, 117)],
        "Cigarette" => return vec![(19, 10, 59), (19, 11, 59), (19, 12, 59), (19, 13, 59), (19, 14, 59), (19, 15, 59), (14, 17, 0), (15, 17, 0), (16, 17, 0), (17, 17, 0), (18, 17, 0), (19, 17, 0), (13, 18, 0), (14, 18, 64), (15, 18, 64), (16, 18, 64), (17, 18, 64), (18, 18, 64), (19, 18, 121), (20, 18, 0), (14, 19, 0), (15, 19, 0), (16, 19, 0), (17, 19, 0), (18, 19, 0), (19, 19, 0)],
        "Pipe" => return vec![(20, 11, 41), (19, 12, 41), (20, 12, 41), (21, 12, 41), (19, 13, 41), (20, 13, 41), (21, 13, 41), (20, 15, 41), (20, 17, 41), (14, 18, 0), (13, 19, 0), (14, 19, 37), (15, 19, 0), (18, 19, 0), (19, 19, 0), (20, 19, 0), (21, 19, 0), (22, 19, 0), (14, 20, 0), (15, 20, 37), (16, 20, 0), (18, 20, 0), (19, 20, 37), (20, 20, 37), (21, 20, 37), (22, 20, 0), (15, 21, 0), (16, 21, 37), (17, 21, 0), (18, 21, 0), (19, 21, 83), (20, 21, 37), (21, 21, 83), (22, 21, 0), (16, 22, 0), (17, 22, 37), (18, 22, 37), (19, 22, 37), (20, 22, 83), (21, 22, 0), (17, 23, 0), (18, 23, 0), (19, 23, 0), (20, 23, 0)],
        "Vape" => return vec![(14, 17, 0), (15, 17, 0), (16, 17, 0), (17, 17, 0), (18, 17, 0), (19, 17, 0), (20, 17, 0), (13, 18, 0), (14, 18, 65), (15, 18, 65), (16, 18, 65), (17, 18, 65), (18, 18, 65), (19, 18, 122), (20, 18, 0), (14, 19, 0), (15, 19, 0), (16, 19, 0), (17, 19, 0), (18, 19, 0), (19, 19, 0), (20, 19, 0)],
        "Choker" => return vec![(9, 20, 0), (9, 21, 0), (10, 21, 0), (10, 22, 0), (11, 22, 0)],
        "Gold Chain" => return vec![(9, 22, 84), (10, 22, 84), (11, 22, 84)],
        "Silver Chain" => return vec![(9, 22, 85), (10, 22, 85), (11, 22, 85)],
        "Clown Nose" => return vec![(12, 15, 73), (13, 15, 73), (12, 16, 73), (13, 16, 73)],
        "0" => return vec![(9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (8, 8, 0), (9, 8, 4), (10, 8, 4), (11, 8, 4), (12, 8, 4), (13, 8, 4), (14, 8, 4), (15, 8, 0), (7, 9, 0), (8, 9, 4), (9, 9, 12), (10, 9, 4), (11, 9, 4), (12, 9, 4), (13, 9, 4), (14, 9, 4), (15, 9, 4), (16, 9, 0), (7, 10, 0), (8, 10, 4), (9, 10, 4), (10, 10, 4), (11, 10, 4), (12, 10, 4), (13, 10, 4), (14, 10, 4), (15, 10, 4), (16, 10, 0), (7, 11, 0), (8, 11, 4), (9, 11, 4), (10, 11, 4), (11, 11, 4), (12, 11, 4), (13, 11, 4), (14, 11, 4), (15, 11, 4), (16, 11, 0), (6, 12, 0), (7, 12, 4), (8, 12, 4), (9, 12, 74), (10, 12, 74), (11, 12, 4), (12, 12, 4), (13, 12, 4), (14, 12, 74), (15, 12, 74), (16, 12, 0), (6, 13, 0), (7, 13, 4), (8, 13, 4), (9, 13, 0), (10, 13, 96), (11, 13, 4), (12, 13, 4), (13, 13, 4), (14, 13, 0), (15, 13, 96), (16, 13, 0), (6, 14, 0), (7, 14, 0), (8, 14, 4), (9, 14, 4), (10, 14, 4), (11, 14, 4), (12, 14, 4), (13, 14, 4), (14, 14, 4), (15, 14, 4), (16, 14, 0), (7, 15, 0), (8, 15, 4), (9, 15, 4), (10, 15, 4), (11, 15, 4), (12, 15, 4), (13, 15, 4), (14, 15, 4), (15, 15, 4), (16, 15, 0), (7, 16, 0), (8, 16, 4), (9, 16, 4), (10, 16, 4), (11, 16, 4), (12, 16, 0), (13, 16, 4), (14, 16, 4), (15, 16, 4), (16, 16, 0), (7, 17, 0), (8, 17, 4), (9, 17, 4), (10, 17, 4), (11, 17, 4), (12, 17, 4), (13, 17, 4), (14, 17, 4), (15, 17, 4), (16, 17, 0), (7, 18, 0), (8, 18, 4), (9, 18, 4), (10, 18, 4), (11, 18, 0), (12, 18, 0), (13, 18, 0), (14, 18, 4), (15, 18, 4), (16, 18, 0), (8, 19, 0), (9, 19, 4), (10, 19, 4), (11, 19, 4), (12, 19, 4), (13, 19, 4), (14, 19, 4), (15, 19, 0), (8, 20, 0), (9, 20, 4), (10, 20, 0), (11, 20, 4), (12, 20, 4), (13, 20, 4), (14, 20, 0), (8, 21, 0), (9, 21, 4), (10, 21, 4), (11, 21, 0), (12, 21, 0), (13, 21, 0), (8, 22, 0), (9, 22, 4), (10, 22, 4), (11, 22, 4), (12, 22, 0), (8, 23, 0), (9, 23, 4), (10, 23, 4), (11, 23, 4), (12, 23, 0)],
        "1" => return vec![(9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (8, 8, 0), (9, 8, 7), (10, 8, 7), (11, 8, 7), (12, 8, 7), (13, 8, 7), (14, 8, 7), (15, 8, 0), (7, 9, 0), (8, 9, 7), (9, 9, 81), (10, 9, 7), (11, 9, 7), (12, 9, 7), (13, 9, 7), (14, 9, 7), (15, 9, 7), (16, 9, 0), (7, 10, 0), (8, 10, 7), (9, 10, 7), (10, 10, 7), (11, 10, 7), (12, 10, 7), (13, 10, 7), (14, 10, 7), (15, 10, 7), (16, 10, 0), (7, 11, 0), (8, 11, 7), (9, 11, 7), (10, 11, 7), (11, 11, 7), (12, 11, 7), (13, 11, 7), (14, 11, 7), (15, 11, 7), (16, 11, 0), (6, 12, 0), (7, 12, 7), (8, 12, 7), (9, 12, 8), (10, 12, 8), (11, 12, 7), (12, 12, 7), (13, 12, 7), (14, 12, 8), (15, 12, 8), (16, 12, 0), (6, 13, 0), (7, 13, 7), (8, 13, 7), (9, 13, 0), (10, 13, 97), (11, 13, 7), (12, 13, 7), (13, 13, 7), (14, 13, 0), (15, 13, 97), (16, 13, 0), (6, 14, 0), (7, 14, 0), (8, 14, 7), (9, 14, 7), (10, 14, 7), (11, 14, 7), (12, 14, 7), (13, 14, 7), (14, 14, 7), (15, 14, 7), (16, 14, 0), (7, 15, 0), (8, 15, 7), (9, 15, 7), (10, 15, 7), (11, 15, 7), (12, 15, 7), (13, 15, 7), (14, 15, 7), (15, 15, 7), (16, 15, 0), (7, 16, 0), (8, 16, 7), (9, 16, 7), (10, 16, 7), (11, 16, 7), (12, 16, 0), (13, 16, 7), (14, 16, 7), (15, 16, 7), (16, 16, 0), (7, 17, 0), (8, 17, 7), (9, 17, 7), (10, 17, 7), (11, 17, 7), (12, 17, 7), (13, 17, 7), (14, 17, 7), (15, 17, 7), (16, 17, 0), (7, 18, 0), (8, 18, 7), (9, 18, 7), (10, 18, 7), (11, 18, 0), (12, 18, 0), (13, 18, 0), (14, 18, 7), (15, 18, 7), (16, 18, 0), (8, 19, 0), (9, 19, 7), (10, 19, 7), (11, 19, 7), (12, 19, 7), (13, 19, 7), (14, 19, 7), (15, 19, 0), (8, 20, 0), (9, 20, 7), (10, 20, 0), (11, 20, 7), (12, 20, 7), (13, 20, 7), (14, 20, 0), (8, 21, 0), (9, 21, 7), (10, 21, 7), (11, 21, 0), (12, 21, 0), (13, 21, 0), (8, 22, 0), (9, 22, 7), (10, 22, 7), (11, 22, 7), (12, 22, 0), (8, 23, 0), (9, 23, 7), (10, 23, 7), (11, 23, 7), (12, 23, 0)],
        "2" => return vec![(9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (8, 8, 0), (9, 8, 2), (10, 8, 2), (11, 8, 2), (12, 8, 2), (13, 8, 2), (14, 8, 2), (15, 8, 0), (7, 9, 0), (8, 9, 2), (9, 9, 136), (10, 9, 2), (11, 9, 2), (12, 9, 2), (13, 9, 2), (14, 9, 2), (15, 9, 2), (16, 9, 0), (7, 10, 0), (8, 10, 2), (9, 10, 2), (10, 10, 2), (11, 10, 2), (12, 10, 2), (13, 10, 2), (14, 10, 2), (15, 10, 2), (16, 10, 0), (7, 11, 0), (8, 11, 2), (9, 11, 2), (10, 11, 2), (11, 11, 2), (12, 11, 2), (13, 11, 2), (14, 11, 2), (15, 11, 2), (16, 11, 0), (6, 12, 0), (7, 12, 2), (8, 12, 2), (9, 12, 46), (10, 12, 46), (11, 12, 2), (12, 12, 2), (13, 12, 2), (14, 12, 46), (15, 12, 46), (16, 12, 0), (6, 13, 0), (7, 13, 2), (8, 13, 2), (9, 13, 0), (10, 13, 71), (11, 13, 2), (12, 13, 2), (13, 13, 2), (14, 13, 0), (15, 13, 71), (16, 13, 0), (6, 14, 0), (7, 14, 0), (8, 14, 2), (9, 14, 2), (10, 14, 2), (11, 14, 2), (12, 14, 2), (13, 14, 2), (14, 14, 2), (15, 14, 2), (16, 14, 0), (7, 15, 0), (8, 15, 2), (9, 15, 2), (10, 15, 2), (11, 15, 2), (12, 15, 2), (13, 15, 2), (14, 15, 2), (15, 15, 2), (16, 15, 0), (7, 16, 0), (8, 16, 2), (9, 16, 2), (10, 16, 2), (11, 16, 2), (12, 16, 0), (13, 16, 2), (14, 16, 2), (15, 16, 2), (16, 16, 0), (7, 17, 0), (8, 17, 2), (9, 17, 2), (10, 17, 2), (11, 17, 2), (12, 17, 2), (13, 17, 2), (14, 17, 2), (15, 17, 2), (16, 17, 0), (7, 18, 0), (8, 18, 2), (9, 18, 2), (10, 18, 2), (11, 18, 0), (12, 18, 0), (13, 18, 0), (14, 18, 2), (15, 18, 2), (16, 18, 0), (8, 19, 0), (9, 19, 2), (10, 19, 2), (11, 19, 2), (12, 19, 2), (13, 19, 2), (14, 19, 2), (15, 19, 0), (8, 20, 0), (9, 20, 2), (10, 20, 0), (11, 20, 2), (12, 20, 2), (13, 20, 2), (14, 20, 0), (8, 21, 0), (9, 21, 2), (10, 21, 2), (11, 21, 0), (12, 21, 0), (13, 21, 0), (8, 22, 0), (9, 22, 2), (10, 22, 2), (11, 22, 2), (12, 22, 0), (8, 23, 0), (9, 23, 2), (10, 23, 2), (11, 23, 2), (12, 23, 0)],
        "3" => return vec![(9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (8, 8, 0), (9, 8, 3), (10, 8, 3), (11, 8, 3), (12, 8, 3), (13, 8, 3), (14, 8, 3), (15, 8, 0), (7, 9, 0), (8, 9, 3), (9, 9, 115), (10, 9, 3), (11, 9, 3), (12, 9, 3), (13, 9, 3), (14, 9, 3), (15, 9, 3), (16, 9, 0), (7, 10, 0), (8, 10, 3), (9, 10, 3), (10, 10, 3), (11, 10, 3), (12, 10, 3), (13, 10, 3), (14, 10, 3), (15, 10, 3), (16, 10, 0), (7, 11, 0), (8, 11, 3), (9, 11, 3), (10, 11, 3), (11, 11, 3), (12, 11, 3), (13, 11, 3), (14, 11, 3), (15, 11, 3), (16, 11, 0), (6, 12, 0), (7, 12, 3), (8, 12, 3), (9, 12, 111), (10, 12, 111), (11, 12, 3), (12, 12, 3), (13, 12, 3), (14, 12, 111), (15, 12, 111), (16, 12, 0), (6, 13, 0), (7, 13, 3), (8, 13, 3), (9, 13, 0), (10, 13, 129), (11, 13, 3), (12, 13, 3), (13, 13, 3), (14, 13, 0), (15, 13, 129), (16, 13, 0), (6, 14, 0), (7, 14, 0), (8, 14, 3), (9, 14, 3), (10, 14, 3), (11, 14, 3), (12, 14, 3), (13, 14, 3), (14, 14, 3), (15, 14, 3), (16, 14, 0), (7, 15, 0), (8, 15, 3), (9, 15, 3), (10, 15, 3), (11, 15, 3), (12, 15, 3), (13, 15, 3), (14, 15, 3), (15, 15, 3), (16, 15, 0), (7, 16, 0), (8, 16, 3), (9, 16, 3), (10, 16, 3), (11, 16, 3), (12, 16, 0), (13, 16, 3), (14, 16, 3), (15, 16, 3), (16, 16, 0), (7, 17, 0), (8, 17, 3), (9, 17, 3), (10, 17, 3), (11, 17, 3), (12, 17, 3), (13, 17, 3), (14, 17, 3), (15, 17, 3), (16, 17, 0), (7, 18, 0), (8, 18, 3), (9, 18, 3), (10, 18, 3), (11, 18, 0), (12, 18, 0), (13, 18, 0), (14, 18, 3), (15, 18, 3), (16, 18, 0), (8, 19, 0), (9, 19, 3), (10, 19, 3), (11, 19, 3), (12, 19, 3), (13, 19, 3), (14, 19, 3), (15, 19, 0), (8, 20, 0), (9, 20, 3), (10, 20, 0), (11, 20, 3), (12, 20, 3), (13, 20, 3), (14, 20, 0), (8, 21, 0), (9, 21, 3), (10, 21, 3), (11, 21, 0), (12, 21, 0), (13, 21, 0), (8, 22, 0), (9, 22, 3), (10, 22, 3), (11, 22, 3), (12, 22, 0), (8, 23, 0), (9, 23, 3), (10, 23, 3), (11, 23, 3), (12, 23, 0)],
        "Alien" => return vec![(9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (8, 8, 0), (9, 8, 5), (10, 8, 5), (11, 8, 5), (12, 8, 5), (13, 8, 5), (14, 8, 5), (15, 8, 0), (7, 9, 0), (8, 9, 5), (9, 9, 5), (10, 9, 101), (11, 9, 5), (12, 9, 5), (13, 9, 5), (14, 9, 5), (15, 9, 5), (16, 9, 0), (7, 10, 0), (8, 10, 5), (9, 10, 101), (10, 10, 5), (11, 10, 5), (12, 10, 5), (13, 10, 5), (14, 10, 5), (15, 10, 5), (16, 10, 0), (7, 11, 0), (8, 11, 5), (9, 11, 5), (10, 11, 5), (11, 11, 5), (12, 11, 5), (13, 11, 5), (14, 11, 5), (15, 11, 5), (16, 11, 0), (6, 12, 0), (7, 12, 66), (8, 12, 5), (9, 12, 102), (10, 12, 0), (11, 12, 5), (12, 12, 5), (13, 12, 5), (14, 12, 102), (15, 12, 0), (16, 12, 0), (6, 13, 0), (7, 13, 5), (8, 13, 5), (9, 13, 0), (10, 13, 66), (11, 13, 5), (12, 13, 5), (13, 13, 5), (14, 13, 0), (15, 13, 66), (16, 13, 0), (6, 14, 0), (7, 14, 0), (8, 14, 5), (9, 14, 5), (10, 14, 5), (11, 14, 5), (12, 14, 5), (13, 14, 5), (14, 14, 5), (15, 14, 5), (16, 14, 0), (7, 15, 0), (8, 15, 5), (9, 15, 5), (10, 15, 5), (11, 15, 5), (12, 15, 5), (13, 15, 5), (14, 15, 5), (15, 15, 5), (16, 15, 0), (7, 16, 0), (8, 16, 5), (9, 16, 5), (10, 16, 5), (11, 16, 5), (12, 16, 0), (13, 16, 5), (14, 16, 5), (15, 16, 5), (16, 16, 0), (7, 17, 0), (8, 17, 5), (9, 17, 5), (10, 17, 5), (11, 17, 5), (12, 17, 5), (13, 17, 5), (14, 17, 5), (15, 17, 5), (16, 17, 0), (7, 18, 0), (8, 18, 5), (9, 18, 5), (10, 18, 5), (11, 18, 0), (12, 18, 0), (13, 18, 0), (14, 18, 5), (15, 18, 5), (16, 18, 0), (8, 19, 0), (9, 19, 5), (10, 19, 5), (11, 19, 5), (12, 19, 5), (13, 19, 5), (14, 19, 5), (15, 19, 0), (8, 20, 0), (9, 20, 5), (10, 20, 0), (11, 20, 5), (12, 20, 5), (13, 20, 5), (14, 20, 0), (8, 21, 0), (9, 21, 5), (10, 21, 5), (11, 21, 0), (12, 21, 0), (13, 21, 0), (8, 22, 0), (9, 22, 5), (10, 22, 5), (11, 22, 5), (12, 22, 0), (8, 23, 0), (9, 23, 5), (10, 23, 5), (11, 23, 5), (12, 23, 0)],
        "Green Alien" => return vec![(9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (8, 8, 0), (9, 8, 138), (10, 8, 138), (11, 8, 138), (12, 8, 138), (13, 8, 138), (14, 8, 138), (15, 8, 0), (7, 9, 0), (8, 9, 138), (9, 9, 138), (10, 9, 141), (11, 9, 138), (12, 9, 138), (13, 9, 138), (14, 9, 138), (15, 9, 138), (16, 9, 0), (7, 10, 0), (8, 10, 138), (9, 10, 141), (10, 10, 138), (11, 10, 138), (12, 10, 138), (13, 10, 138), (14, 10, 138), (15, 10, 138), (16, 10, 0), (7, 11, 0), (8, 11, 138), (9, 11, 138), (10, 11, 138), (11, 11, 138), (12, 11, 138), (13, 11, 138), (14, 11, 138), (15, 11, 138), (16, 11, 0), (6, 12, 0), (7, 12, 140), (8, 12, 138), (9, 12, 139), (10, 12, 0), (11, 12, 138), (12, 12, 138), (13, 12, 138), (14, 12, 139), (15, 12, 0), (16, 12, 0), (6, 13, 0), (7, 13, 138), (8, 13, 138), (9, 13, 0), (10, 13, 140), (11, 13, 138), (12, 13, 138), (13, 13, 138), (14, 13, 0), (15, 13, 140), (16, 13, 0), (6, 14, 0), (7, 14, 0), (8, 14, 138), (9, 14, 138), (10, 14, 138), (11, 14, 138), (12, 14, 138), (13, 14, 138), (14, 14, 138), (15, 14, 138), (16, 14, 0), (7, 15, 0), (8, 15, 138), (9, 15, 138), (10, 15, 138), (11, 15, 138), (12, 15, 138), (13, 15, 138), (14, 15, 138), (15, 15, 138), (16, 15, 0), (7, 16, 0), (8, 16, 138), (9, 16, 138), (10, 16, 138), (11, 16, 138), (12, 16, 0), (13, 16, 138), (14, 16, 138), (15, 16, 138), (16, 16, 0), (7, 17, 0), (8, 17, 138), (9, 17, 138), (10, 17, 138), (11, 17, 138), (12, 17, 138), (13, 17, 138), (14, 17, 138), (15, 17, 138), (16, 17, 0), (7, 18, 0), (8, 18, 138), (9, 18, 138), (10, 18, 138), (11, 18, 0), (12, 18, 0), (13, 18, 0), (14, 18, 138), (15, 18, 138), (16, 18, 0), (8, 19, 0), (9, 19, 138), (10, 19, 138), (11, 19, 138), (12, 19, 138), (13, 19, 138), (14, 19, 138), (15, 19, 0), (8, 20, 0), (9, 20, 138), (10, 20, 0), (11, 20, 138), (12, 20, 138), (13, 20, 138), (14, 20, 0), (8, 21, 0), (9, 21, 138), (10, 21, 138), (11, 21, 0), (12, 21, 0), (13, 21, 0), (8, 22, 0), (9, 22, 138), (10, 22, 138), (11, 22, 138), (12, 22, 0), (8, 23, 0), (9, 23, 138), (10, 23, 138), (11, 23, 138), (12, 23, 0)],
        "Red 0 Alien" => return build_female_alien_asset(vec![179,180,181,178]),
        "Red 1 Alien" => return build_female_alien_asset(vec![182,183,184,178]),
        "Red 2 Alien" => return build_female_alien_asset(vec![185,186,187,178]),
        "Red 3 Alien" => return build_female_alien_asset(vec![188,189,190,178]),
        "Red 4 Alien" => return build_female_alien_asset(vec![191,192,193,178]),
        "Yellow Alien" => return build_female_alien_asset(vec![146,148,147,149]),
        "White Alien" => return build_female_alien_asset(vec![150,152,151,153]),
        "Black Alien" => return build_female_alien_asset(vec![154,156,155,157]),
        "Blue 0 Alien" => return build_female_alien_asset(vec![158,160,159,161]),
        "Blue 1 Alien" => return build_female_alien_asset(vec![162,164,163,165]),
        "Blue 2 Alien" => return build_female_alien_asset(vec![166,168,167,169]),
        "Blue 3 Alien" => return build_female_alien_asset(vec![170,172,171,173]),
        "Blue 4 Alien" => return build_female_alien_asset(vec![174,174,175,177]),
        "Ape" => return vec![(9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (8, 8, 0), (9, 8, 15), (10, 8, 15), (11, 8, 15), (12, 8, 15), (13, 8, 15), (14, 8, 15), (15, 8, 0), (7, 9, 0), (8, 9, 15), (9, 9, 15), (10, 9, 103), (11, 9, 15), (12, 9, 15), (13, 9, 15), (14, 9, 15), (15, 9, 15), (16, 9, 0), (7, 10, 0), (8, 10, 15), (9, 10, 103), (10, 10, 15), (11, 10, 15), (12, 10, 15), (13, 10, 15), (14, 10, 15), (15, 10, 15), (16, 10, 0), (7, 11, 0), (8, 11, 15), (9, 11, 15), (10, 11, 0), (11, 11, 0), (12, 11, 0), (13, 11, 0), (14, 11, 15), (15, 11, 15), (16, 11, 0), (6, 12, 0), (7, 12, 15), (8, 12, 13), (9, 12, 76), (10, 12, 76), (11, 12, 13), (12, 12, 13), (13, 12, 13), (14, 12, 76), (15, 12, 76), (16, 12, 0), (6, 13, 0), (7, 13, 15), (8, 13, 13), (9, 13, 0), (10, 13, 104), (11, 13, 13), (12, 13, 13), (13, 13, 13), (14, 13, 0), (15, 13, 104), (16, 13, 0), (6, 14, 0), (7, 14, 0), (8, 14, 13), (9, 14, 13), (10, 14, 13), (11, 14, 13), (12, 14, 13), (13, 14, 13), (14, 14, 13), (15, 14, 13), (16, 14, 0), (7, 15, 0), (8, 15, 15), (9, 15, 13), (10, 15, 13), (11, 15, 13), (12, 15, 13), (13, 15, 13), (14, 15, 13), (15, 15, 13), (16, 15, 0), (7, 16, 0), (8, 16, 15), (9, 16, 15), (10, 16, 13), (11, 16, 13), (12, 16, 0), (13, 16, 13), (14, 16, 13), (15, 16, 15), (16, 16, 0), (7, 17, 0), (8, 17, 15), (9, 17, 13), (10, 17, 13), (11, 17, 13), (12, 17, 13), (13, 17, 13), (14, 17, 13), (15, 17, 13), (16, 17, 0), (7, 18, 0), (8, 18, 15), (9, 18, 13), (10, 18, 13), (11, 18, 0), (12, 18, 0), (13, 18, 0), (14, 18, 13), (15, 18, 13), (16, 18, 0), (8, 19, 0), (9, 19, 15), (10, 19, 13), (11, 19, 13), (12, 19, 13), (13, 19, 13), (14, 19, 13), (15, 19, 0), (8, 20, 0), (9, 20, 15), (10, 20, 0), (11, 20, 13), (12, 20, 13), (13, 20, 13), (14, 20, 0), (8, 21, 0), (9, 21, 15), (10, 21, 15), (11, 21, 0), (12, 21, 0), (13, 21, 0), (8, 22, 0), (9, 22, 15), (10, 22, 15), (11, 22, 15), (12, 22, 0), (8, 23, 0), (9, 23, 15), (10, 23, 15), (11, 23, 15), (12, 23, 0)],
        "Zombie" => return vec![(9, 7, 0), (10, 7, 0), (11, 7, 0), (12, 7, 0), (13, 7, 0), (14, 7, 0), (8, 8, 0), (9, 8, 6), (10, 8, 6), (11, 8, 6), (12, 8, 6), (13, 8, 6), (14, 8, 6), (15, 8, 0), (7, 9, 0), (8, 9, 6), (9, 9, 6), (10, 9, 105), (11, 9, 6), (12, 9, 6), (13, 9, 6), (14, 9, 6), (15, 9, 6), (16, 9, 0), (7, 10, 0), (8, 10, 6), (9, 10, 105), (10, 10, 6), (11, 10, 6), (12, 10, 6), (13, 10, 6), (14, 10, 6), (15, 10, 6), (16, 10, 0), (7, 11, 0), (8, 11, 6), (9, 11, 6), (10, 11, 6), (11, 11, 6), (12, 11, 6), (13, 11, 6), (14, 11, 6), (15, 11, 6), (16, 11, 0), (6, 12, 0), (7, 12, 6), (8, 12, 6), (9, 12, 48), (10, 12, 48), (11, 12, 6), (12, 12, 6), (13, 12, 6), (14, 12, 48), (15, 12, 48), (16, 12, 0), (6, 13, 0), (7, 13, 6), (8, 13, 6), (9, 13, 106), (10, 13, 0), (11, 13, 6), (12, 13, 6), (13, 13, 6), (14, 13, 106), (15, 13, 0), (16, 13, 0), (6, 14, 0), (7, 14, 0), (8, 14, 6), (9, 14, 48), (10, 14, 6), (11, 14, 6), (12, 14, 6), (13, 14, 6), (14, 14, 48), (15, 14, 6), (16, 14, 0), (7, 15, 0), (8, 15, 6), (9, 15, 6), (10, 15, 6), (11, 15, 6), (12, 15, 6), (13, 15, 6), (14, 15, 6), (15, 15, 6), (16, 15, 0), (7, 16, 0), (8, 16, 6), (9, 16, 6), (10, 16, 6), (11, 16, 6), (12, 16, 0), (13, 16, 6), (14, 16, 6), (15, 16, 6), (16, 16, 0), (7, 17, 0), (8, 17, 6), (9, 17, 6), (10, 17, 6), (11, 17, 6), (12, 17, 6), (13, 17, 6), (14, 17, 6), (15, 17, 6), (16, 17, 0), (7, 18, 0), (8, 18, 6), (9, 18, 6), (10, 18, 6), (11, 18, 0), (12, 18, 0), (13, 18, 0), (14, 18, 6), (15, 18, 6), (16, 18, 0), (8, 19, 0), (9, 19, 6), (10, 19, 6), (11, 19, 48), (12, 19, 6), (13, 19, 6), (14, 19, 6), (15, 19, 0), (8, 20, 0), (9, 20, 6), (10, 20, 0), (11, 20, 6), (12, 20, 6), (13, 20, 6), (14, 20, 0), (8, 21, 0), (9, 21, 6), (10, 21, 6), (11, 21, 0), (12, 21, 0), (13, 21, 0), (8, 22, 0), (9, 22, 6), (10, 22, 6), (11, 22, 6), (12, 22, 0), (8, 23, 0), (9, 23, 6), (10, 23, 6), (11, 23, 6), (12, 23, 0)],
        _ => return vec![]
    }
}

fn generate_randomu8(
    address: &str,
    count: u64,
    index: u64,
) -> u8 {
    (hash((address.to_string() + &count.to_string() + &index.to_string()).as_bytes()).wrapping_rem(32)) as u8
}

pub fn generate_address(
    address: &str,
    count: u64,
) -> Bech32 {
    return Bech32 {
        hrp: String::from("terra1"),
        data: vec![
            generate_randomu8(address, count, 0u64),
            generate_randomu8(address, count, 1u64),
            generate_randomu8(address, count, 2u64),
            generate_randomu8(address, count, 3u64),
            generate_randomu8(address, count, 4u64),
            generate_randomu8(address, count, 5u64),
            generate_randomu8(address, count, 6u64),
            generate_randomu8(address, count, 7u64),
            generate_randomu8(address, count, 8u64),
            generate_randomu8(address, count, 9u64),
            generate_randomu8(address, count, 10u64),
            generate_randomu8(address, count, 11u64),
            generate_randomu8(address, count, 12u64),
            generate_randomu8(address, count, 13u64),
            generate_randomu8(address, count, 14u64),
            generate_randomu8(address, count, 15u64),
            generate_randomu8(address, count, 16u64),
            generate_randomu8(address, count, 17u64),
            generate_randomu8(address, count, 18u64),
            generate_randomu8(address, count, 19u64),
            generate_randomu8(address, count, 20u64),
            generate_randomu8(address, count, 21u64),
            generate_randomu8(address, count, 22u64),
            generate_randomu8(address, count, 23u64),
            generate_randomu8(address, count, 24u64),
            generate_randomu8(address, count, 25u64),
            generate_randomu8(address, count, 26u64),
            generate_randomu8(address, count, 27u64),
            generate_randomu8(address, count, 28u64),
            generate_randomu8(address, count, 29u64),
            generate_randomu8(address, count, 30u64),
            generate_randomu8(address, count, 31u64)]//String::from("qq4lzrgzj5kx048mv2zawvh2pn3eumnr").into_bytes()
    };
}

