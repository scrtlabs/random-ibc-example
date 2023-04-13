use cosmwasm_std::CosmosMsg::Wasm;
use cosmwasm_std::{
    entry_point, Binary, ContractInfo, CosmosMsg, DepsMut, Env, Event, MessageInfo, Response,
    StdError, StdResult, WasmMsg,
};
use schemars::JsonSchema;
use secret_toolkit::storage::Item;
use secret_toolkit::utils::types::Contract;

use crate::random::get_random_msg;
use serde::{Deserialize, Serialize};

pub const KEY_STORED_RANDOM: &[u8] = b"rand";
pub static STORED_RANDOM: Item<ContractInfo> = Item::new(KEY_STORED_RANDOM);

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    save_contract(deps.storage, msg.get_contract())?;

    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    let rand_provider = get_contract(deps.storage)?;

    match msg {
        ExecuteMsg::DoSomething { .. } => {
            let msg = get_random_msg(
                env.clone(),
                rand_provider,
                env.block.height.to_string(),
                None,
            )?;
            Ok(Response::new().add_message(msg))
        }
        ExecuteMsg::RandomResponse { random, job_id, .. } => Ok(Response::new()
            .add_attribute_plaintext("random", random)
            .add_attribute("job_id", job_id)),
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InstantiateMsg {
    Init { rand_provider: ContractInfo },
}

impl InstantiateMsg {
    fn get_contract(self) -> ContractInfo {
        match self {
            InstantiateMsg::Init { rand_provider } => rand_provider,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    DoSomething {},
    RandomResponse {
        random: String,
        job_id: String,
        msg: Option<Binary>,
    },
}

pub fn get_contract(store: &dyn cosmwasm_std::Storage) -> StdResult<ContractInfo> {
    STORED_RANDOM
        .load(store)
        .map_err(|_err| StdError::generic_err("No stored random contract here"))
}

pub fn save_contract(store: &mut dyn cosmwasm_std::Storage, random: ContractInfo) -> StdResult<()> {
    STORED_RANDOM.save(store, &random)
}
