use cosmwasm_std::{StdError, StdResult, Storage, WasmMsg};
use schemars::JsonSchema;
use secret_toolkit::storage::Item;
use serde::{Deserialize, Serialize};
use crate::msg::CallbackInfo;

pub const KEY_LAST_IBC_OPERATION: &[u8] = b"last_op";
pub const KEY_LAST_OPENED_CHANNEL: &[u8] = b"opened_channel";
pub const KEY_STORED_RANDOM: &[u8] = b"rand";
pub const KEY_CALLBACK: &[u8] = b"cb";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Operation {
    pub name: String,
    pub parameters: Vec<String>,
}

pub static LAST_IBC_OPERATION: Item<Operation> = Item::new(KEY_LAST_IBC_OPERATION);

impl Operation {
    pub fn get_last(store: &dyn Storage) -> StdResult<Operation> {
        LAST_IBC_OPERATION
            .load(store)
            .map_err(|_err| StdError::generic_err("no ibc operation stored yet"))
    }

    pub fn save_last(store: &mut dyn Storage, operation: Operation) -> StdResult<()> {
        LAST_IBC_OPERATION.save(store, &operation)
    }
}

pub static LAST_OPENED_CHANNEL: Item<String> = Item::new(KEY_LAST_OPENED_CHANNEL);

pub struct Channel {}
impl Channel {
    pub fn get_last_opened(store: &dyn Storage) -> StdResult<String> {
        LAST_OPENED_CHANNEL
            .load(store)
            .map_err(|_err| StdError::generic_err("no channel was opened on this contract yet"))
    }

    pub fn save_last_opened(store: &mut dyn Storage, channel_id: String) -> StdResult<()> {
        LAST_OPENED_CHANNEL.save(store, &channel_id)
    }
}

pub static STORED_RANDOM: Item<String> = Item::new(KEY_STORED_RANDOM);
pub static STORED_CALLBACK: Item<CallbackInfo> = Item::new(KEY_CALLBACK);

pub struct StoredRandomAnswer {}
impl StoredRandomAnswer {
    pub fn get(store: &dyn Storage) -> StdResult<String> {
        STORED_RANDOM.load(store).map_err(|_err| {
            StdError::generic_err("no life answer was received on this contract yet")
        })
    }

    pub fn save(store: &mut dyn Storage, random: String) -> StdResult<()> {
        STORED_RANDOM.save(store, &random)
    }
}

pub fn load_callback(store: &dyn Storage) -> StdResult<CallbackInfo> {
    STORED_CALLBACK.load(store).map_err(|_err| {
        StdError::generic_err("no life answer was received on this contract yet")
    })
}

pub fn save_callback(store: &mut dyn Storage, msg: CallbackInfo) -> StdResult<()> {
    STORED_CALLBACK.save(store, &msg)
}