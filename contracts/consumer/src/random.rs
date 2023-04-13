use crate::random::ExecuteMsg::RequestRandom;
use cosmwasm_std::WasmMsg::Execute;
use cosmwasm_std::{
    from_binary, to_binary, Binary, ContractInfo, CosmosMsg, Env, StdResult, WasmMsg,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
enum ExecuteMsg {
    RequestRandom { job_id: String, callback: WasmMsg },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RandomCallback {
    RandomResponse {
        random: String,
        job_id: String,
        msg: Option<Binary>,
    },
}

pub fn get_random_msg(
    env: Env,
    provider: ContractInfo,
    job_id: String,
    msg: Option<Binary>,
) -> StdResult<CosmosMsg> {
    Ok(CosmosMsg::Wasm(Execute {
        contract_addr: provider.address.to_string(),
        code_hash: provider.code_hash,
        msg: to_binary(&RequestRandom {
            job_id,
            callback: WasmMsg::Execute {
                contract_addr: env.contract.address.to_string(),
                code_hash: env.contract.code_hash,
                msg: Binary::default(),
                funds: vec![],
            },
        })?,
        funds: vec![],
    }))
}

pub fn parse_random_response(msg: Binary) -> StdResult<(String, String, Option<Binary>)> {
    let parsed_msg: RandomCallback = from_binary(&msg)?;

    match parsed_msg {
        RandomCallback::RandomResponse {
            random,
            job_id,
            msg,
        } => Ok((random, job_id, msg)),
    }
}
