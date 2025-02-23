use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use arcswap::msg::{
    ExecuteMsg, InfoResponse, InstantiateMsg, NativeForTokenPriceResponse, QueryMsg,
    TokenForNativePriceResponse,
};
use arcswap::state::Token;
use cw20::BalanceResponse;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Token), &out_dir);
    export_schema(&schema_for!(BalanceResponse), &out_dir);
    export_schema(&schema_for!(InfoResponse), &out_dir);
    export_schema(&schema_for!(NativeForTokenPriceResponse), &out_dir);
    export_schema(&schema_for!(TokenForNativePriceResponse), &out_dir);
}
