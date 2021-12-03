use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw20::Expiration;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub native_denom: String,
    pub token_denom: String,
    pub token_address: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddLiquidity {
        min_liquidity: Uint128,
        max_token: Uint128,
        expiration: Option<Expiration>,
    },
    RemoveLiquidity {
        amount: Uint128,
        min_native: Uint128,
        min_token: Uint128,
        expiration: Option<Expiration>,
    },
    SwapToken1ForToken2 {
        min_token: Uint128,
        expiration: Option<Expiration>,
    },
    SwapToken2ForToken1 {
        token_amount: Uint128,
        min_native: Uint128,
        expiration: Option<Expiration>,
    },
    SwapTokenForToken {
        output_arcswap_address: Addr,
        input_token_amount: Uint128,
        output_min_token: Uint128,
        expiration: Option<Expiration>,
    },
    SwapNativeForTokenTo {
        recipient: Addr,
        min_token: Uint128,
        expiration: Option<Expiration>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Implements CW20. Returns the current balance of the given address, 0 if unset.
    Balance {
        address: String,
    },
    Info {},
    NativeForTokenPrice {
        native_amount: Uint128,
    },
    TokenForNativePrice {
        token_amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {
    pub native_reserve: Uint128,
    pub native_denom: String,
    pub token_reserve: Uint128,
    pub token_denom: String,
    pub token_address: String,
    pub lp_token_supply: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NativeForTokenPriceResponse {
    pub token_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenForNativePriceResponse {
    pub native_amount: Uint128,
}
