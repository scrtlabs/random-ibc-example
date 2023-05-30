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