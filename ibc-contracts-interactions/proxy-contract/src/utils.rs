use cosmwasm_std::WasmMsg;

pub fn verify_callback(callback: &WasmMsg) -> bool {
    match callback {
        WasmMsg::Execute { funds, .. } | WasmMsg::Instantiate { funds, .. } => {
            funds.is_empty()
        }
        _ => { true }
    }
}