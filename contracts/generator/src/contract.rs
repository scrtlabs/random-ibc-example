use cosmwasm_std::{
    // import necessary types and traits from cosmwasm_std
    entry_point,
    from_binary,
    to_binary,
    Binary,
    Deps,
    DepsMut,
    Env,
    Ibc3ChannelOpenResponse,
    IbcBasicResponse,
    IbcChannelCloseMsg,
    IbcChannelConnectMsg,
    IbcChannelOpenMsg,
    IbcChannelOpenResponse,
    IbcMsg,
    IbcPacketAckMsg,
    IbcPacketReceiveMsg,
    IbcPacketTimeoutMsg,
    IbcReceiveResponse,
    IbcTimeout,
    MessageInfo,
    Response,
    StdResult,
    WasmMsg,
};

use crate::msg::{InstantiateMsg, PacketMsg, QueryMsg};
use crate::state::Channel;
// use crate::utils::verify_callback;
use secret_toolkit_crypto::Prng;

// Define a constant for the IBC app version
pub const IBC_APP_VERSION: &str = "ibc-v1";
// Define a constant for the packet lifetime in seconds
const PACKET_LIFETIME: u64 = 60 * 60;
const BECH32_LEN: usize = 32;

// Instantiate entry point
#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    // Return a response with an attribute "init" containing the serialized last operation
    Ok(Response::default()
        .add_attribute("init", to_binary(&"Initialized".to_string())?.to_string()))
}

// #[entry_point]
// pub fn execute(
//     deps: DepsMut,
//     env: Env,
//     _info: MessageInfo,
//     msg: ExecuteMsg,
// ) -> StdResult<Response> {
//     Ok(Response::default())
// }

// #[entry_point]
// pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
//     match msg {
//         QueryMsg::LastIbcOperation {} => Ok(to_binary(&"No operations".to_string())?),
//
//         QueryMsg::ViewReceivedLifeAnswer {} => {
//             // todo the StoredRandomAnswer is never saved to
//             Ok(to_binary(&StoredRandomAnswer::get(deps.storage)?)?)
//         }
//     }
// }

#[entry_point]
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcChannelOpenMsg,
) -> StdResult<IbcChannelOpenResponse> {
    Ok(Some(Ibc3ChannelOpenResponse {
        version: IBC_APP_VERSION.to_string(),
    }))
}

#[entry_point]
pub fn ibc_channel_connect(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {
    match msg {
        IbcChannelConnectMsg::OpenAck { channel, .. } => {
            // save channel to state
            let channel_id = channel.endpoint.channel_id;
            Channel::save_last_opened(deps.storage, channel_id)?;
        }

        IbcChannelConnectMsg::OpenConfirm { channel } => {
            // save channel to state
            let channel_id = channel.endpoint.channel_id;
            Channel::save_last_opened(deps.storage, channel_id)?;
        }

        _ => {}
    };

    Ok(IbcBasicResponse::default())
}

#[entry_point]
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketReceiveMsg,
) -> StdResult<IbcReceiveResponse> {
    let mut response = IbcReceiveResponse::new();

    let packet: PacketMsg = from_binary(&msg.packet.data)?;
    match packet {
        PacketMsg::Message { value } => {
            let res = PacketMsg::Message {
                value: format!("got your message: {}", value),
            };
            response = response.set_ack(to_binary(&res).unwrap());
        }

        // todo: return random with different lengths
        PacketMsg::RequestRandom { job_id, .. } => {
            deps.api.debug(&format!("{:?}", env));

            // todo: handle random not in block for some reason?
            let random = env.block.random.unwrap();

            let mut rng = Prng::new(random.as_slice(), job_id.as_bytes());
            let rand_for_job = hex::encode(rng.rand_bytes());

            let res = PacketMsg::RandomResponse {
                random: rand_for_job,
                job_id,
            };
            response = response.set_ack(to_binary(&res).unwrap());
        }

        _ => {}
    }

    Ok(response)
}

// #[entry_point]
// pub fn ibc_packet_ack(
//     deps: DepsMut,
//     _env: Env,
//     msg: IbcPacketAckMsg,
// ) -> StdResult<IbcBasicResponse> {
//     let ack_data = from_binary(&msg.acknowledgement.data)?;
//     match ack_data {
//         PacketMsg::Message { .. } => Ok(IbcBasicResponse::default()),
//
//         PacketMsg::RandomResponse { job_id, random } => {
//             let callback = pop_callback(deps.storage, &job_id)?;
//
//             let original_job_id = job_id[BECH32_LEN..].to_string();
//             let msg = create_random_response_callback(callback, original_job_id, random)?;
//
//             Ok(IbcBasicResponse::default().add_message(msg))
//         }
//
//         _ => Ok(IbcBasicResponse::default()),
//     }
// }

#[entry_point]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::default())
}

#[entry_point]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::default())
}
