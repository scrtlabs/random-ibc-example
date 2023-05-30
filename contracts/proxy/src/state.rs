use crate::msg::CallbackInfo;
use cosmwasm_std::{StdError, StdResult, Storage};
use secret_toolkit::storage::{Item, Keymap};

pub const KEY_LAST_OPENED_CHANNEL: &[u8] = b"opened_channel";
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

pub static STORED_CALLBACK: Keymap<String, CallbackInfo> = Keymap::new(KEY_CALLBACK);

pub fn pop_callback(store: &mut dyn Storage, job_id: &String) -> StdResult<CallbackInfo> {
    let callback = STORED_CALLBACK
        .get(store, job_id)
        .ok_or(StdError::generic_err("no active job with that id was found"))?;

    STORED_CALLBACK
        .remove(store, job_id)
        .map_err(|_err| StdError::generic_err("unable to remove job"))?;

    Ok(callback)
}

pub fn save_callback(store: &mut dyn Storage, job_id: &String, msg: CallbackInfo) -> StdResult<()> {
    STORED_CALLBACK.insert(store, job_id, &msg)
}
