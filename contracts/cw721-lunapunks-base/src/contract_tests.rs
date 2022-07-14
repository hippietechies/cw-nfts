#![cfg(test)]
use std::fs::File;
use std::marker::PhantomData;
use crate::contract::{LunaPunkExecuteMsg, LunaPunkQueryMsg, self};
use crate::execute::generate_address;
use crate::state::{Cw721ExtendedContract, Extension};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, CosmosMsg, DepsMut, Response, WasmMsg, StdResult, StdError};

use cw721::{
    ApprovalsResponse, ContractInfoResponse, Cw721Query, Cw721ReceiveMsg, Expiration,
    NftInfoResponse, OwnerOfResponse, OperatorsResponse, NumTokensResponse, TokensResponse, ApprovalResponse, Approval,
};
use cw721_base::{MintMsg, InstantiateMsg, ContractError};


use seahash::hash;


const MINTER: &str = "terra1vwyra0qafx8qf5x84530tef44z9wjvzytdgzxy";
const CREATOR: &str = "terra1dcegyrekltswvyy0xy69ydgxn9x8x32zdtapd8";
const USER1: &str = "terra1qzw84hfrha4hjr4q4xsntqduk8lkjmdz2r5deg";
const CONTRACT_NAME: &str = "Magic Power";
const SYMBOL: &str = "MGK";

fn setup_contract(deps: DepsMut<'_>) {
    // let contract = Cw721ExtendedContract::default();
    let msg = InstantiateMsg {
        name: CONTRACT_NAME.to_string(),
        symbol: SYMBOL.to_string(),
        minter: String::from(MINTER),
    };
    let info = mock_info(CREATOR, &[]);
    let res = contract::instantiate(deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
}

#[test]
fn proper_instantiation() {
    let mut deps = mock_dependencies();
    let contract = Cw721ExtendedContract::default();

    let msg = InstantiateMsg {
        name: CONTRACT_NAME.to_string(),
        symbol: SYMBOL.to_string(),
        minter: String::from(MINTER),
    };
    let info = mock_info(CREATOR, &[]);

    // we can just call .unwrap() to assert this was a success
    let res = contract
        .instantiate(deps.as_mut(), mock_env(), info, msg)
        .unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = contract.minter(deps.as_ref()).unwrap();
    assert_eq!(MINTER, res.minter);
    let info = contract.contract_info(deps.as_ref()).unwrap();
    assert_eq!(
        info,
        ContractInfoResponse {
            name: CONTRACT_NAME.to_string(),
            symbol: SYMBOL.to_string(),
        }
    );

    let count = contract.num_tokens(deps.as_ref()).unwrap();
    assert_eq!(0, count.count);

    // list the token_ids
    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
    assert_eq!(0, tokens.tokens.len());
}

#[test]
fn janantest() -> StdResult<()> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use bech32::Bech32;
    use std::convert::TryInto;
    use std::ops::Div;



    fn my_hash<T>(obj: T) -> String
    where
        T: Hash,
    {
        let mut hasher = DefaultHasher::new();
        obj.hash(&mut hasher);
        let retu: u64 = hasher.finish() / u32::MAX as u64;
        return retu.to_string();
    }
    println!("STARTing:");

    fn pp(address: String) {

        let mut add = address;
        add.push_str("janan");
        println!("hello: {}", my_hash::<String>(add));
    }

    pp("terra1vwyra0qafx8qf5x84530tef44z9wjvzytdgzxy".to_string());



    // let whitelist_password = Some("janan");434704956
    // let password = Some("4188125025");
    // let sender = "terra1dcegyrekltswvyy0xy69ydgxn9x8x32zdtapd8";

    // println!("hello: {}", whitelist_password.is_some());

    // if whitelist_password.is_some() {
    //     match password {
    //         Some(password) => {
    //             let mut add = sender.to_string().to_owned();
    //             add.push_str(&whitelist_password.unwrap_or_default());
    //             println!("hello: {}", password);
    //             println!("hello: {}", my_hash(&add));
    //             println!("hello: {}", password != my_hash(&add));
    //             if password == my_hash(&add) {
    //                 println!("hello: {}", password == my_hash(add));
    //             }
    //         }
    //         None => {},
    //     }
    // }

    Ok(())
}
    #[test]
fn janan3() -> StdResult<()> {
    use bech32::Bech32;
    // use rand::Rng; // 0.8.0

    // Generate random number in the range [0, 99]

    // for index in 0..100 {
    //     let test = Bech32 {
    //         hrp: String::from("terra1"),
    //         // data: vec![rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32)]
    //     };
    //     println!("{:?}", test.to_string());
    // }

        let test = Bech32 {
            hrp: String::from("terra"),
            data: vec![15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8, 15u8]
        };
        println!("{:?}", test.to_string());
    Ok(())
}
#[test]
fn janan4() -> StdResult<()> {
    use bech32::Bech32;
    // use rand::Rng; // 0.8.0

    // Generate random number in the range [0, 99]

    // for index in 0..100 {
    //     let test = Bech32 {
    //         hrp: String::from("terra1"),
    //         // data: vec![rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32), rand::thread_rng().gen_range(0..32)]
    //     };
    //     println!("{:?}", test.to_string());
    // }

    use std::cmp::max;
    let price = u128::from(
        max(
            5737909u64.saturating_sub(5537809u64),
            100000,
        ) / 100000
            * 1000000,
    );
    println!("{:?}", price);
    Ok(())
}

pub fn generate_random2(
    address: &str,
    count: u64,
    index: u64,
) -> u8 {
    (hash((address.to_string() + &count.to_string() + &index.to_string()).as_bytes()).wrapping_rem(2)) as u8
}

#[test]
fn randomflip() -> StdResult<()> {
    use std::fs::write;
    use std::io::Write;
    // use std::fs::write_fmt;
    let path = "results.txt";
    let mut output = File::create(path).ok().unwrap();
    let line = "hello";

    for count in 0u64..50000u64 {
        writeln!(output, "{}", generate_random2("terra1vwyra0qafx8qf5x84530tef44z9wjvzytdgzxy", 1u64, count).to_string());

        // println!("{:?}", generate_random2("terra1vwyra0qafx8qf5x84530tef44z9wjvzytdgzxy", 1u64, count));
    }


    Ok(())
}
#[test]
fn randomAdd() -> StdResult<()> {
    use bech32::Bech32;
    let add = "terra1vwyra0qafx8qf5x84530tef44z9wjvzytdgzxy".to_string() + &"2".to_string();
    let jan = (hash((add.to_string() + &"1".to_string()).as_bytes()) / u8::MAX as u64) as u8;
    let jan2 = (hash((add.to_string() + &"2".to_string()).as_bytes()) / u8::MAX as u64) as u8;
    let jan3 = (hash((add.to_string() + &"3".to_string()).as_bytes()) / u8::MAX as u64) as u8;
    let jan4 = (hash((add.to_string() + &"4".to_string()).as_bytes()) / u8::MAX as u64) as u8;
    let jan5 = (hash((add.to_string() + &"5".to_string()).as_bytes()) / u8::MAX as u64) as u8;
    let jan6 = (hash((add.to_string() + &"6".to_string()).as_bytes()) / u8::MAX as u64) as u8;

    let jan7 = generate_address(&add, 1u64);
    // println!("{:?}", jan);
    // println!("{:?}", jan2);
    // println!("{:?}", jan3);
    // println!("{:?}", jan4);
    // println!("{:?}", jan5);
    // println!("{:?}", jan6);
    println!("{:?}", jan7.data);
    println!("{:?}", jan7.to_string());

    let female_max = Bech32 {
        hrp: String::from("terra1"),
        data: vec![3u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8]//String::from("qq4lzrgzj5kx048mv2zawvh2pn3eumnr").into_bytes()
    };
    println!("{:?}", female_max.data);
    println!("{:?}", female_max.to_string());
    // 117
    // 193
    // 186
    // 229
    // 145
    // 31
    Ok(())
}


#[test]
fn janan2() -> StdResult<()> {
    use bech32::Bech32;

    let female_max = Bech32 {
        hrp: String::from("terra1"),
        data: vec![3u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8]//String::from("qq4lzrgzj5kx048mv2zawvh2pn3eumnr").into_bytes()
    };
    let male_max = Bech32 {
        hrp: String::from("terra1"),
        data: vec![2u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8]//String::from("qq4lzrgzj5kx048mv2zawvh2pn3eumnr").into_bytes()
    };
    let female_max_zom = Bech32 {
        hrp: String::from("terra1"),
        data: vec![3u8, 31u8, 31u8, 31u8, 31u8, 23u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8]//String::from("qq4lzrgzj5kx048mv2zawvh2pn3eumnr").into_bytes()
    };
    let male_max_zom = Bech32 {
        hrp: String::from("terra1"),
        data: vec![2u8, 31u8, 31u8, 31u8, 31u8, 23u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8]//String::from("qq4lzrgzj5kx048mv2zawvh2pn3eumnr").into_bytes()
    };
    let female_max_ape = Bech32 {
        hrp: String::from("terra1"),
        data: vec![3u8, 31u8, 31u8, 31u8, 31u8, 14u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8]//String::from("qq4lzrgzj5kx048mv2zawvh2pn3eumnr").into_bytes()
    };
    let male_max_ape = Bech32 {
        hrp: String::from("terra1"),
        data: vec![2u8, 31u8, 31u8, 31u8, 31u8, 14u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8]//String::from("qq4lzrgzj5kx048mv2zawvh2pn3eumnr").into_bytes()
    };

    // 31 earring any
    // 29-30 gold chain 768 - 1024
    // 19-21 pipe 640-896
    // 13-17 small shades 992- 1024
    // 8 cap forwardHead 0-63

    let wyner_male_max_ape = Bech32 {
        hrp: String::from("terra1"),
        data: vec![2u8, 30u8, 4u8, 15u8, 31u8, 14u8, 30u8, 29u8, 21u8, 20u8, 0u8, 0u8, 15u8, 1u8, 16u8, 24u8, 4u8, 16u8, 19u8, 9u8, 12u8, 11u8, 1u8, 2u8, 31u8, 23u8, 21u8, 8u8, 28u8, 11u8, 0u8, 0u8]//String::from("qq4lzrgzj5kx048mv2zawvh2pn3eumnr").into_bytes()
    };

    // hoodie
    // rosy cheeks
    // buck teeth
    let jan2_male_max = Bech32 {
        hrp: String::from("terra1"),
        data: vec![2u8, 4u8, 2u8, 0u8, 31u8, 31u8, 24u8, 22u8, 5u8, 4u8, 2u8, 0u8, 9u8, 9u8, 9u8, 6u8, 6u8, 6u8, 9u8, 4u8, 2u8, 0u8, 1u8, 3u8, 3u8, 7u8, 29u8, 15u8, 20u8, 6u8, 6u8, 6u8]//String::from("qq4lzrgzj5kx048mv2zawvh2pn3eumnr").into_bytes()
    };


    let jan1_max_zom = Bech32 {
        hrp: String::from("terra1"),
        data: vec![2u8, 31u8, 6u8, 9u8, 31u8, 23u8, 30u8, 30u8, 25u8, 25u8, 24u8, 6u8, 27u8, 30u8, 4u8, 6u8, 13u8, 21u8, 23u8, 13u8, 31u8, 18u8, 26u8, 24u8, 21u8, 11u8, 29u8, 27u8, 29u8, 2u8, 12u8, 5u8]//String::from("qq4lzrgzj5kx048mv2zawvh2pn3eumnr").into_bytes()
    };

    let jan3_max_ape = Bech32 {
        hrp: String::from("terra1"),
        data: vec![2u8, 30u8, 4u8, 15u8, 31u8, 14u8, 30u8, 30u8, 25u8, 25u8, 24u8, 31u8, 12u8, 21u8, 22u8, 25u8, 12u8, 19u8, 29u8, 28u8, 26u8, 31u8, 30u8, 15u8, 24u8, 3u8, 29u8, 7u8, 29u8, 25u8, 2u8, 1u8]//String::from("qq4lzrgzj5kx048mv2zawvh2pn3eumnr").into_bytes()
    };


    let jan4_max_zom = Bech32 {
        hrp: String::from("terra1"),
        data: vec![3u8, 30u8, 4u8, 15u8, 31u8, 23u8, 31u8, 31u8, 27u8, 23u8, 1u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 29u8, 17u8, 23u8, 15u8, 28u8, 19u8, 17u8, 13u8, 21u8, 1u8]//String::from("qq4lzrgzj5kx048mv2zawvh2pn3eumnr").into_bytes()
    };
    let jan5_max_zom = Bech32 {
        hrp: String::from("terra1"),
        data: vec![3u8, 31u8, 31u8, 31u8, 3u8, 23u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 31u8, 29u8, 1u8, 30u8, 26u8, 29u8, 2u8, 30u8, 0u8, 29u8, 7u8, 28u8, 18u8, 17u8, 27u8, 1u8, 2u8, 3u8, 4u8]//String::from("qq4lzrgzj5kx048mv2zawvh2pn3eumnr").into_bytes()
    };


    println!("{}", "Hello");
    println!("{:?}", female_max.to_string());
    println!("{:?}", male_max.to_string());
    println!("{:?}", female_max_zom.to_string());
    println!("{:?}", male_max_zom.to_string());
    println!("{:?}", female_max_ape.to_string());
    println!("{:?}", male_max_ape.to_string());
    println!("{:?}", wyner_male_max_ape.to_string());
    println!("{:?}", jan2_male_max.to_string());
    println!("{:?}", jan1_max_zom.to_string());
    println!("{:?}", jan3_max_ape.to_string());
    println!("{:?}", jan4_max_zom.to_string());
    println!("{:?}", jan5_max_zom.to_string());
    Ok(())

}

#[test]
fn demo() -> StdResult<()> {
    use cosmwasm_std::testing::{MockStorage};
    use cosmwasm_std::{Order};

    use serde::{Deserialize, Serialize};
    use cw_storage_plus::{Bound,Map, MultiIndex};

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Data {
        pub name: String,
        pub age: i32,
    }

    const PEOPLE: Map<&[u8], Data> = Map::new("people");
    const ALLOWANCE: Map<(&[u8], &[u8]), u64> = Map::new("allow");

    #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
    struct Bid {
        pub bidder: String,
        pub coins: Vec<u32>,
    }

    let mut store = MockStorage::new();

    // save and load on two keys
    let data = Data { name: "John".to_string(), age: 32 };
    PEOPLE.save(&mut store, b"john", &data)?;
    let data2 = Data { name: "Jim".to_string(), age: 44 };
    PEOPLE.save(&mut store, b"jim", &data2)?;

    // iterate over them all
    let all: StdResult<Vec<_>> = PEOPLE
        .range(&store, None, None, Order::Ascending)
        .collect();
    assert_eq!(
        all?,
        vec![(b"jim".to_vec(), data2), (b"john".to_vec(), data.clone())]
    );

    // or just show what is after jim
    let all: StdResult<Vec<_>> = PEOPLE
        .range(
            &store,
            Some(Bound::Exclusive((&b"jim".to_vec(), PhantomData))),
            None,
            Order::Ascending,
        )
        .collect();
    assert_eq!(all?, vec![(b"john".to_vec(), data)]);

    // save and load on three keys, one under different owner
    ALLOWANCE.save(&mut store, (b"owner", b"spender"), &1000)?;
    ALLOWANCE.save(&mut store, (b"owner", b"spender2"), &3000)?;
    ALLOWANCE.save(&mut store, (b"owner2", b"spender"), &5000)?;

    // get all under one key
    let all: StdResult<Vec<_>> = ALLOWANCE
        .prefix(b"owner")
        .range(&store, None, None, Order::Ascending)
        .collect();
    assert_eq!(
        all?,
        vec![(b"spender".to_vec(), 1000), (b"spender2".to_vec(), 3000)]
    );

    // Or ranges between two items (even reverse)
    let all: StdResult<Vec<_>> = ALLOWANCE
        .prefix(b"owner")
        .range(
            &store,
            Some(Bound::Exclusive((&b"spender1".to_vec(), PhantomData))),
            Some(Bound::Inclusive((&b"spender2".to_vec(), PhantomData))),
            Order::Descending,
        )
        .collect();
    assert_eq!(all?, vec![(b"spender2".to_vec(), 3000)]);

    Ok(())
}



#[test]
fn minting() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    let token_id = "1".to_string();
    let token_id2 = "2".to_string();
    let token_uri = "https://www.merriam-webster.com/dictionary/petrify".to_string();
    let mint_msg = LunaPunkExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id.clone(),
        owner: String::from("medusa"),
        token_uri: Some(token_uri.clone()),
        extension: None,
    });

    // random cannot mint
    let random = mock_info("random", &[]);
    let err = contract::execute(deps.as_mut(), mock_env(), random, mint_msg.clone().into()).unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // minter can mint
    let allowed = mock_info(MINTER, &[]);
    let _ = contract::execute(deps.as_mut(), mock_env(), allowed, mint_msg.into()).unwrap();

    // ensure num tokens increases
    let count: NumTokensResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), LunaPunkQueryMsg::NumTokens {  }).unwrap()).unwrap();
    assert_eq!(1, count.count);

    // unknown nft returns error
    let error = contract::query(deps.as_ref(), mock_env(), LunaPunkQueryMsg::NftInfo { token_id: "unknown".to_string()}).unwrap_err();
    assert_eq!(
        error,
        StdError::parse_err("unknown", "token_id should be digit string")
    );
    // this nft info is correct
    let nft_info_message = LunaPunkQueryMsg::NftInfo { token_id: token_id.clone()};
    let info: NftInfoResponse<Extension> = from_binary(&contract::query(deps.as_ref(), mock_env(), nft_info_message).unwrap()).unwrap();
    assert_eq!(
        info,
        NftInfoResponse::<Extension> {
            token_uri: Some(token_uri),
            extension: info.extension.clone(),
        }
    );

    // owner info is correct
    let owner_of_message = LunaPunkQueryMsg::OwnerOf { token_id: token_id.clone(), include_expired: Some(true) };
    let owner: OwnerOfResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), owner_of_message).unwrap()).unwrap();
    assert_eq!(
        owner,
        OwnerOfResponse {
            owner: String::from("medusa"),
            approvals: vec![],
        }
    );

    // should auto increment token_id mint
    let mint_msg2 = LunaPunkExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id.clone(),
        owner: String::from("hercules"),
        token_uri: None,
        extension: None,
    });

    let allowed = mock_info(MINTER, &[]);
    let _ = contract::execute(deps.as_mut(), mock_env(), allowed, mint_msg2.into()).unwrap();

    // list the token_ids
    let all_tokens_msg = LunaPunkQueryMsg::AllTokens { start_after: None, skip: None, limit: None };
    let tokens: TokensResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), all_tokens_msg).unwrap()).unwrap();
    assert_eq!(2, tokens.tokens.len());
    assert_eq!(vec![token_id, token_id2], tokens.tokens);
}

#[test]
fn transferring_nft() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    // Mint a token
    let token_id = "1".to_string();
    let token_uri = "https://www.merriam-webster.com/dictionary/melt".to_string();

    let mint_msg = LunaPunkExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id.clone(),
        owner: String::from("venus"),
        token_uri: Some(token_uri),
        extension: None,
    });

    let minter = mock_info(MINTER, &[]);
    contract::execute(deps.as_mut(), mock_env(), minter, mint_msg.into())
        .unwrap();

    // random cannot transfer
    let random = mock_info("random", &[]);
    let transfer_msg = LunaPunkExecuteMsg::TransferNft {
        recipient: String::from("random"),
        token_id: token_id.clone(),
    };

    let err = contract::execute(deps.as_mut(), mock_env(), random, transfer_msg.into())
        .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // owner can
    let random = mock_info("venus", &[]);
    let transfer_msg = LunaPunkExecuteMsg::TransferNft {
        recipient: String::from("random"),
        token_id: token_id.clone(),
    };

    let res = contract::execute(deps.as_mut(), mock_env(), random, transfer_msg.into())
        .unwrap();

    assert_eq!(
        res,
        Response::new()
            .add_attribute("action", "transfer_nft")
            .add_attribute("sender", "venus")
            .add_attribute("recipient", "random")
            .add_attribute("token_id", token_id)
    );
}

#[test]
fn sending_nft() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    // Mint a token
    let token_id = "1".to_string();
    let token_uri = "https://www.merriam-webster.com/dictionary/melt".to_string();

    let mint_msg = LunaPunkExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id.clone(),
        owner: String::from("venus"),
        token_uri: Some(token_uri),
        extension: None,
    });

    let minter = mock_info(MINTER, &[]);
    contract::execute(deps.as_mut(), mock_env(), minter, mint_msg.into())
        .unwrap();

    let msg = to_binary("You now have the melting power").unwrap();
    let target = String::from("another_contract");
    let send_msg = LunaPunkExecuteMsg::SendNft {
        contract: target.clone(),
        token_id: token_id.clone(),
        msg: msg.clone(),
    };

    let random = mock_info("random", &[]);
    let err = contract::execute(deps.as_mut(), mock_env(), random, send_msg.clone().into())
        .unwrap_err();
    assert_eq!(err, ContractError::Unauthorized {});

    // but owner can
    let random = mock_info("venus", &[]);
    let res = contract::execute(deps.as_mut(), mock_env(), random, send_msg.into())
        .unwrap();

    let payload = Cw721ReceiveMsg {
        sender: String::from("venus"),
        token_id: token_id.clone(),
        msg,
    };
    let expected = payload.into_cosmos_msg(target.clone()).unwrap();
    // ensure expected serializes as we think it should
    match &expected {
        CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, .. }) => {
            assert_eq!(contract_addr, &target)
        }
        m => panic!("Unexpected message type: {:?}", m),
    }
    // and make sure this is the request sent by the contract
    assert_eq!(
        res,
        Response::new()
            .add_message(expected)
            .add_attribute("action", "send_nft")
            .add_attribute("sender", "venus")
            .add_attribute("recipient", "another_contract")
            .add_attribute("token_id", token_id)
    );
}

#[test]
fn approving_revoking() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    // Mint a token
    let token_id = "1".to_string();
    let token_uri = "https://www.merriam-webster.com/dictionary/grow".to_string();
    println!("1");
    let mint_msg = LunaPunkExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id.clone(),
        owner: String::from(USER1),
        token_uri: Some(token_uri),
        extension: None,
    });

    let minter = mock_info(MINTER, &[]);
    println!("2a");
    contract::execute(deps.as_mut(), mock_env(), minter, mint_msg.into())
        .unwrap();
        println!("2b");

    // Give random transferring power
    let approve_msg = LunaPunkExecuteMsg::Approve {
        spender: String::from("random"),
        token_id: token_id.clone(),
        expires: None,
    };
    let owner = mock_info(USER1, &[]);
    let res = contract::execute(deps.as_mut(), mock_env(), owner, approve_msg.into())
        .unwrap();

    let approve_msg = LunaPunkQueryMsg::Approval {
        spender: String::from("random"),
        token_id: token_id.clone(),
        include_expired: Some(true),
    };
    // let res: ApprovalResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), approve_msg.into()).unwrap()).unwrap();
    assert_eq!(
        res,
        // ApprovalResponse { approval: todo!() }
        Response::new()
            .add_attribute("action", "approve")
            .add_attribute("sender", USER1)
            .add_attribute("spender", "random")
            .add_attribute("token_id", token_id.clone())
    );
    println!("3");

    // random can now transfer
    let random = mock_info("random", &[]);
    let transfer_msg = LunaPunkExecuteMsg::TransferNft {
        recipient: String::from("person"),
        token_id: token_id.clone(),
    };
    contract::execute(deps.as_mut(), mock_env(), random, transfer_msg.into())
        .unwrap();
        println!("4");

    // Approvals are removed / cleared
    let query_msg = LunaPunkQueryMsg::OwnerOf {
        token_id: token_id.clone(),
        include_expired: None,
    };
    let res: OwnerOfResponse = from_binary(
        &contract::query(deps.as_ref(), mock_env(), query_msg.clone().into())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        res,
        OwnerOfResponse {
            owner: String::from("person"),
            approvals: vec![],
        }
    );
    println!("5");

    // Approve, revoke, and check for empty, to test revoke
    let approve_msg = LunaPunkExecuteMsg::Approve {
        spender: String::from("random"),
        token_id: token_id.clone(),
        expires: None,
    };
    let owner = mock_info("person", &[]);
    contract::execute(deps.as_mut(), mock_env(), owner.clone(), approve_msg.into())
        .unwrap();

    let revoke_msg = LunaPunkExecuteMsg::Revoke {
        spender: String::from("random"),
        token_id,
    };
    contract::execute(deps.as_mut(), mock_env(), owner, revoke_msg.into())
        .unwrap();

    // Approvals are now removed / cleared
    let res: OwnerOfResponse = from_binary(
        &contract::query(deps.as_ref(), mock_env(), query_msg.into())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        res,
        OwnerOfResponse {
            owner: String::from("person"),
            approvals: vec![],
        }
    );
}

#[test]
fn approving_all_revoking_all1() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    // Mint a couple tokens (from the same owner)
    let token_id1 = "1".to_string();
    let token_uri1 = "https://www.merriam-webster.com/dictionary/grow1".to_string();

    let token_id2 = "2".to_string();
    let token_uri2 = "https://www.merriam-webster.com/dictionary/grow2".to_string();

    let mint_msg1 = LunaPunkExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id1.clone(),
        owner: String::from(USER1),
        token_uri: Some(token_uri1),
        extension: None,
    });

    let minter = mock_info(MINTER, &[]);
    contract::execute(deps.as_mut(), mock_env(), minter.clone(), mint_msg1.into())
        .unwrap();

    let mint_msg2 = LunaPunkExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id2.clone(),
        owner: String::from(USER1),
        token_uri: Some(token_uri2),
        extension: None,
    });

    contract::execute(deps.as_mut(), mock_env(), minter, mint_msg2.into())
        .unwrap();

    // paginate the token_ids

    let all_tokens_msg = LunaPunkQueryMsg::AllTokens { start_after: None, skip: None, limit: Some(1) };
    let tokens: TokensResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), all_tokens_msg).unwrap()).unwrap();

    assert_eq!(1, tokens.tokens.len());
    assert_eq!(vec![token_id1.clone()], tokens.tokens);

    let all_tokens_msg = LunaPunkQueryMsg::AllTokens { start_after: None, skip: None, limit: Some(3) };
    let tokens: TokensResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), all_tokens_msg).unwrap()).unwrap();

    assert_eq!(2, tokens.tokens.len());
    assert_eq!(vec![token_id1.clone(), token_id2.clone()], tokens.tokens);

    // demeter gives random full (operator) power over her tokens
    let approve_all_msg = LunaPunkExecuteMsg::ApproveAll {
        operator: String::from("random"),
        expires: None,
    };
    let owner = mock_info(USER1, &[]);
    let res = contract::execute(deps.as_mut(), mock_env(), owner, approve_all_msg.into())
        .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_attribute("action", "approve_all")
            .add_attribute("sender", USER1)
            .add_attribute("operator", "random")
    );

    // random can now transfer
    let random = mock_info("random", &[]);
    let transfer_msg = LunaPunkExecuteMsg::TransferNft {
        recipient: String::from("person"),
        token_id: token_id1,
    };
    contract::execute(deps.as_mut(), mock_env(), random.clone(), transfer_msg.into())
        .unwrap();

    // random can now send
    let inner_msg = WasmMsg::Execute {
        contract_addr: "another_contract".into(),
        msg: to_binary("You now also have the growing power").unwrap(),
        funds: vec![],
    };
    let msg: CosmosMsg = CosmosMsg::Wasm(inner_msg);

    let send_msg = LunaPunkExecuteMsg::SendNft {
        contract: String::from("another_contract"),
        token_id: token_id2.to_string(),
        msg: to_binary(&msg).unwrap(),
    };
    contract::execute(deps.as_mut(), mock_env(), random, send_msg.into())
        .unwrap();

    // Approve_all, revoke_all, and check for empty, to test revoke_all
    let approve_all_msg = LunaPunkExecuteMsg::ApproveAll {
        operator: String::from("operator"),
        expires: None,
    };
    // person is now the owner of the tokens
    let owner = mock_info("person", &[]);
    contract::execute(deps.as_mut(), mock_env(), owner.clone(), approve_all_msg.into())
        .unwrap();

    let operators_msg = LunaPunkQueryMsg::AllOperators { include_expired: Some(true), owner: owner.sender.to_string(), start_after: None, limit: None  };
    let res: OperatorsResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), operators_msg).unwrap()).unwrap();

    assert_eq!(
        res,
        OperatorsResponse { operators: vec![
            Approval {spender:String::from("operator"),expires:Expiration::Never{}}
        ] }
    );
    // second approval
    let buddy_expires = Expiration::AtHeight(1234567);
    let approve_all_msg = LunaPunkExecuteMsg::ApproveAll {
        operator: String::from("buddy"),
        expires: Some(buddy_expires),
    };
    let owner = mock_info("person", &[]);
    contract::execute(deps.as_mut(), mock_env(), owner.clone(), approve_all_msg.into())
        .unwrap();

    // and paginate queries
    let operators_msg = LunaPunkQueryMsg::AllOperators { include_expired: Some(true), owner: String::from("person"), start_after: None, limit: None  };
    let res: OperatorsResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), operators_msg).unwrap()).unwrap();

    assert_eq!(
        res,
        OperatorsResponse { operators: vec![
            Approval {spender:String::from("buddy"),expires:buddy_expires},
            Approval {spender:String::from("operator"),expires:Expiration::Never{}}
        ] }
    );

    let revoke_all_msg = LunaPunkExecuteMsg::RevokeAll {
        operator: String::from("operator"),
    };
    contract::execute(deps.as_mut(), mock_env(), owner, revoke_all_msg.into())
        .unwrap();

    // Approvals are removed / cleared without affecting others
    let operators_msg = LunaPunkQueryMsg::AllOperators { include_expired: Some(false), owner: String::from("person"), start_after: None, limit: None  };
    let res: OperatorsResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), operators_msg).unwrap()).unwrap();

    assert_eq!(
        res,
        OperatorsResponse { operators: vec![
            Approval {spender:String::from("buddy"),expires:buddy_expires},
        ] }
    );

    // ensure the filter works (nothing should be here
    let mut late_env = mock_env();
    late_env.block.height = 1234568; //expired
    let operators_msg = LunaPunkQueryMsg::AllOperators { include_expired: Some(false), owner: String::from("person"), start_after: None, limit: None  };
    let res: OperatorsResponse = from_binary(&contract::query(deps.as_ref(), late_env, operators_msg).unwrap()).unwrap();

    assert_eq!(
        0,
        res.operators.len()
    );
}

#[test]
fn query_tokens_by_owner() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());
    let minter = mock_info(MINTER, &[]);

    // Mint a couple tokens (from the same owner)
    let token_id1 = "1".to_string();
    let demeter = String::from("terra1qzw84hfrha4hjr4q4xsntqduk8lkjmdz2r5deg");
    let token_id2 = "2".to_string();
    let ceres = String::from("terra1qfy2nfr0zh70jyr3h4ns9rzqx4fl8rxpf09ytv");
    let token_id3 = "3".to_string();

    let mint_msg = LunaPunkExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id1.clone(),
        owner: demeter.clone(),
        token_uri: None,
        extension: None,
    });
    contract::execute(deps.as_mut(), mock_env(), minter.clone(), mint_msg.into())
        .unwrap();

    let mint_msg = LunaPunkExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id2.clone(),
        owner: ceres.clone(),
        token_uri: None,
        extension: None,
    });
    contract::execute(deps.as_mut(), mock_env(), minter.clone(), mint_msg.into())
        .unwrap();

    let mint_msg = LunaPunkExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id3.clone(),
        owner: demeter.clone(),
        token_uri: None,
        extension: None,
    });
    contract::execute(deps.as_mut(), mock_env(), minter, mint_msg.into())
        .unwrap();

    // get all tokens in order:
    let expected = vec![token_id1.clone(), token_id2.clone(), token_id3.clone()];

    let all_tokens_msg = LunaPunkQueryMsg::AllTokens { start_after: None, skip: None, limit: None };
    let tokens: TokensResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), all_tokens_msg).unwrap()).unwrap();

    assert_eq!(&expected, &tokens.tokens);
    // paginate

    let all_tokens_msg = LunaPunkQueryMsg::AllTokens { start_after: None, skip: None, limit: Some(2) };
    let tokens: TokensResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), all_tokens_msg).unwrap()).unwrap();

    assert_eq!(&expected[..2], &tokens.tokens[..]);

    let all_tokens_msg = LunaPunkQueryMsg::AllTokens { start_after: Some(expected[1].clone()), skip: None, limit: None };
    let tokens: TokensResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), all_tokens_msg).unwrap()).unwrap();

    assert_eq!(&expected[2..], &tokens.tokens[..]);

    // get by owner
    let by_ceres = vec![token_id2];
    let by_demeter = vec![token_id1, token_id3];

    // all tokens by owner
    let all_tokens_msg = LunaPunkQueryMsg::Tokens {  start_after: None, owner: demeter.clone(), skip: None, limit: None };
    let tokens: TokensResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), all_tokens_msg).unwrap()).unwrap();

    assert_eq!(&by_demeter, &tokens.tokens);

    let tokens_msg = LunaPunkQueryMsg::Tokens { start_after: None, owner: ceres, skip: None, limit: None };
    let tokens: TokensResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), tokens_msg).unwrap()).unwrap();

    assert_eq!(&by_ceres, &tokens.tokens);

    // paginate for demeter

    let tokens_msg = LunaPunkQueryMsg::Tokens { start_after: None, owner: demeter.clone(), skip: None, limit: Some(1) };
    let tokens: TokensResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), tokens_msg).unwrap()).unwrap();

    assert_eq!(&by_demeter[..1], &tokens.tokens[..]);

    let tokens_msg = LunaPunkQueryMsg::Tokens { start_after: Some(by_demeter[0].clone()), owner: demeter.clone(), skip: None, limit: Some(3) };
    let tokens: TokensResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), tokens_msg).unwrap()).unwrap();
    assert_eq!(&by_demeter[1..], &tokens.tokens[..]);

    let tokens_msg = LunaPunkQueryMsg::Tokens { start_after: None, owner: demeter, skip: Some(1), limit: Some(1) };
    let tokens: TokensResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), tokens_msg).unwrap()).unwrap();
    assert_eq!(&by_demeter[1..], &tokens.tokens[..]);
}

#[test]
fn approving_all_revoking_all2() {
    let mut deps = mock_dependencies();
    let contract = setup_contract(deps.as_mut());

    // Mint a couple tokens (from the same owner)
    let token_id1 = "1".to_string();
    let token_uri1 = "https://www.merriam-webster.com/dictionary/grow1".to_string();

    let token_id2 = "2".to_string();
    let token_uri2 = "https://www.merriam-webster.com/dictionary/grow2".to_string();

    let mint_msg1 = LunaPunkExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id1.clone(),
        owner: String::from(USER1),
        token_uri: Some(token_uri1),
        extension: None,
    });

    let minter = mock_info(MINTER, &[]);
    let temp = contract::execute(deps.as_mut(), mock_env(), minter.clone(), mint_msg1.into())
        .unwrap();

    println!("mint result 1: {:?}", temp);
    let mint_msg2 = LunaPunkExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id2.clone(),
        owner: String::from(USER1),
        token_uri: Some(token_uri2),
        extension: None,
    });

    let temp2 = contract::execute(deps.as_mut(), mock_env(), minter, mint_msg2.into())
        .unwrap();

    println!("mint result 2: {:?}", temp2);
    // paginate the token_ids

    let all_tokens_msg = LunaPunkQueryMsg::AllTokens { start_after: None, skip: None, limit: Some(1) };
    let tokens: TokensResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), all_tokens_msg).unwrap()).unwrap();

    assert_eq!(1, tokens.tokens.len());
    assert_eq!(vec![token_id1.clone()], tokens.tokens);

    let all_tokens_msg = LunaPunkQueryMsg::AllTokens { start_after: None, skip: None, limit: Some(3) };
    let tokens: TokensResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), all_tokens_msg).unwrap()).unwrap();

    assert_eq!(2, tokens.tokens.len());
    assert_eq!(vec![token_id1.clone(), token_id2.clone()], tokens.tokens);

    // demeter gives random full (operator) power over her tokens
    let approve_all_msg = LunaPunkExecuteMsg::ApproveAll {
        operator: String::from("random"),
        expires: None,
    };
    let owner = mock_info(USER1, &[]);
    let res = contract::execute(deps.as_mut(), mock_env(), owner, approve_all_msg.into())
        .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_attribute("action", "approve_all")
            .add_attribute("sender", USER1)
            .add_attribute("operator", "random")
    );

    // random can now transfer
    let random = mock_info("random", &[]);
    let transfer_msg = LunaPunkExecuteMsg::TransferNft {
        recipient: String::from("person"),
        token_id: token_id1,
    };
    contract::execute(deps.as_mut(), mock_env(), random.clone(), transfer_msg.into())
        .unwrap();

    // random can now send
    let inner_msg = WasmMsg::Execute {
        contract_addr: "another_contract".into(),
        msg: to_binary("You now also have the growing power").unwrap(),
        funds: vec![],
    };
    let msg: CosmosMsg = CosmosMsg::Wasm(inner_msg);

    let send_msg = LunaPunkExecuteMsg::SendNft {
        contract: String::from("another_contract"),
        token_id: token_id2,
        msg: to_binary(&msg).unwrap(),
    };
    contract::execute(deps.as_mut(), mock_env(), random, send_msg.into())
        .unwrap();

    // Approve_all, revoke_all, and check for empty, to test revoke_all
    let approve_all_msg = LunaPunkExecuteMsg::ApproveAll {
        operator: String::from("operator"),
        expires: None,
    };
    // person is now the owner of the tokens
    let owner = mock_info("person", &[]);
    contract::execute(deps.as_mut(), mock_env(), owner, approve_all_msg.into())
        .unwrap();

    let all_operator_msg = LunaPunkQueryMsg::AllOperators { start_after: None, limit: None, owner: String::from("person"), include_expired: Some(true) };
    let res: OperatorsResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), all_operator_msg).unwrap()).unwrap();

    assert_eq!(
        res,
        OperatorsResponse {
            operators: vec![cw721::Approval {
                spender: String::from("operator"),
                expires: Expiration::Never {}
            }]
        }
    );

    // second approval
    let buddy_expires = Expiration::AtHeight(1234567);
    let approve_all_msg = LunaPunkExecuteMsg::ApproveAll {
        operator: String::from("buddy"),
        expires: Some(buddy_expires),
    };
    let owner = mock_info("person", &[]);
    let test = contract::execute(deps.as_mut(), mock_env(), owner.clone(), approve_all_msg.into())
        .unwrap();
    println!("test: {:?}", test);
    // and paginate queries


    let all_operator_msg = LunaPunkQueryMsg::AllOperators { start_after: None, limit: Some(1), owner: String::from("person"), include_expired: Some(true) };
    let res: OperatorsResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), all_operator_msg).unwrap()).unwrap();

    assert_eq!(
        res,
        OperatorsResponse {
            operators: vec![cw721::Approval {
                spender: String::from("buddy"),
                expires: buddy_expires,
            }]
        }
    );

    let all_operator_msg = LunaPunkQueryMsg::AllOperators { start_after: Some(String::from("buddy")), limit: Some(2), owner: String::from("person"), include_expired: Some(true) };
    let res: OperatorsResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), all_operator_msg).unwrap()).unwrap();

    assert_eq!(
        res,
        OperatorsResponse {
            operators: vec![cw721::Approval {
                spender: String::from("operator"),
                expires: Expiration::Never {}
            }]
        }
    );

    let revoke_all_msg = LunaPunkExecuteMsg::RevokeAll {
        operator: String::from("operator"),
    };
    contract::execute(deps.as_mut(), mock_env(), owner, revoke_all_msg.into())
        .unwrap();

    // Approvals are removed / cleared without affecting others
    let all_operator_msg = LunaPunkQueryMsg::AllOperators { start_after: None, limit: None, owner: String::from("person"), include_expired: Some(false) };
    let res: OperatorsResponse = from_binary(&contract::query(deps.as_ref(), mock_env(), all_operator_msg).unwrap()).unwrap();

    assert_eq!(
        res,
        OperatorsResponse {
            operators: vec![cw721::Approval {
                spender: String::from("buddy"),
                expires: buddy_expires,
            }]
        }
    );

    // ensure the filter works (nothing should be here
    let mut late_env = mock_env();
    late_env.block.height = 1234568; //expired

    let all_operator_msg = LunaPunkQueryMsg::AllOperators { start_after: None, limit: None, owner: String::from("person"), include_expired: Some(false) };
    let res: OperatorsResponse = from_binary(&contract::query(deps.as_ref(), late_env.clone(), all_operator_msg).unwrap()).unwrap();
    assert_eq!(0, res.operators.len());

    let all_operator_msg = LunaPunkQueryMsg::AllOperators { start_after: None, limit: None, owner: String::from("person"), include_expired: Some(true) };
    let res: OperatorsResponse = from_binary(&contract::query(deps.as_ref(), late_env, all_operator_msg).unwrap()).unwrap();
    assert_eq!(1, res.operators.len());
}
