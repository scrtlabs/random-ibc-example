use cosmwasm_std::{Binary, ContractInfo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InstantiateMsg {
    Init {},
}

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
// #[serde(rename_all = "snake_case")]
// pub enum ExecuteMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    LastIbcOperation {},
    ViewReceivedLifeAnswer {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PacketMsg {
    Message { value: String },
    RequestRandom { job_id: String, length: Option<u32> },
    RandomResponse { job_id: String, random: String },
}

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub enum RandomCallback {
//     RandomResponse {
//         random: String,
//         job_id: String,
//         msg: Option<Binary>,
//     },
// }
//
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// pub struct CallbackInfo {
//     pub msg: Option<Binary>,
//     pub contract: ContractInfo,
// }