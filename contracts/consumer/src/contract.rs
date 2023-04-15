use cosmwasm_std::{
    entry_point, to_binary, Binary, ContractInfo, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult,
};
use schemars::JsonSchema;
use secret_toolkit::storage::Item;

use crate::random::get_random_msg;
use serde::{Deserialize, Serialize};

pub const KEY_STORED_RANDOM_CONTRACT: &[u8] = b"rand";
pub const KEY_STORED_RANDOM_RESULT: &[u8] = b"rand_result";

pub static STORED_RANDOM: Item<ContractInfo> = Item::new(KEY_STORED_RANDOM_CONTRACT);
pub static STORED_RANDOM_RESULT: Item<(u64, String)> = Item::new(KEY_STORED_RANDOM_CONTRACT);

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::LastRandom {} => {
            let rand_value = get_rand_result(deps.storage)?;

            Ok(to_binary(&LastRandomResponse {
                height: rand_value.0,
                random: rand_value.1,
            })?)
        }
    }
}

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
        ExecuteMsg::RandomResponse { random, job_id, .. } => {
            store_rand_result(deps.storage, env.block.height, random.clone())?;

            Ok(Response::new()
                .add_attribute_plaintext("random", random)
                .add_attribute("job_id", job_id))
        }
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    LastRandom {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LastRandomResponse {
    pub height: u64,
    pub random: String,
}

pub fn get_contract(store: &dyn cosmwasm_std::Storage) -> StdResult<ContractInfo> {
    STORED_RANDOM
        .load(store)
        .map_err(|_err| StdError::generic_err("No stored random contract here"))
}

pub fn save_contract(store: &mut dyn cosmwasm_std::Storage, random: ContractInfo) -> StdResult<()> {
    STORED_RANDOM.save(store, &random)
}

pub fn get_rand_result(store: &dyn cosmwasm_std::Storage) -> StdResult<(u64, String)> {
    STORED_RANDOM_RESULT
        .load(store)
        .map_err(|_err| StdError::generic_err("No stored random contract here"))
}

pub fn store_rand_result(
    store: &mut dyn cosmwasm_std::Storage,
    height: u64,
    random: String,
) -> StdResult<()> {
    STORED_RANDOM_RESULT.save(store, &(height, random))
}
