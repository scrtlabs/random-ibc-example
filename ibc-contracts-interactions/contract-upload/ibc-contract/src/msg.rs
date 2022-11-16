use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::Operation;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InstantiateMsg {
    Init {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SendIbcPacket {
        message: String,
    },
    RequestRandomnessFromOtherChain {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    LastIbcOperation {},
    ViewReceivedRandomness {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    LastIBCOperation(Operation),
    ViewReceivedRandomness(u32),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PacketMsg {
    Test {},
    Message { value: String },
    RequestRandomness { job_id: String },
    ReceiveRandomness { random_value: u32 },
}