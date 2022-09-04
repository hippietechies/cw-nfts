use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};

use cw721::{
    AllNftInfoResponse, ContractInfoResponse, NftInfoResponse,
    NumTokensResponse, OwnerOfResponse, TokensResponse,
};
use cw721_base::{InstantiateMsg, MinterResponse, Extension};
use cw721_lunapunks::contract::{LunaPunkExecuteMsg, LunaPunkQueryMsg};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema_with_title(&schema_for!(LunaPunkExecuteMsg<Extension>), &out_dir, "LunaPunkExecuteMsg");
    export_schema(&schema_for!(LunaPunkQueryMsg), &out_dir);
    export_schema_with_title(
        &schema_for!(AllNftInfoResponse<Extension>),
        &out_dir,
        "AllNftInfoResponse",
    );
    export_schema(&schema_for!(ContractInfoResponse), &out_dir);
    export_schema(&schema_for!(MinterResponse), &out_dir);
    export_schema_with_title(
        &schema_for!(NftInfoResponse<Extension>),
        &out_dir,
        "NftInfoResponse",
    );
    export_schema(&schema_for!(NumTokensResponse), &out_dir);
    export_schema(&schema_for!(OwnerOfResponse), &out_dir);
    export_schema(&schema_for!(TokensResponse), &out_dir);
}