use crate::msg::CallbackInfo;
use cosmwasm_std::{StdError, StdResult, Storage};
use secret_toolkit::storage::Item;

pub const KEY_LAST_OPENED_CHANNEL: &[u8] = b"opened_channel";
pub const KEY_STORED_RANDOM: &[u8] = b"rand";
pub const KEY_CALLBACK: &[u8] = b"cb";

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
    STORED_CALLBACK
        .load(store)
        .map_err(|_err| StdError::generic_err("no life answer was received on this contract yet"))
}

pub fn save_callback(store: &mut dyn Storage, msg: CallbackInfo) -> StdResult<()> {
    STORED_CALLBACK.save(store, &msg)
}
