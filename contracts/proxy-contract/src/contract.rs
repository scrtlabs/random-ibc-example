use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary, Deps, DepsMut, Env, Ibc3ChannelOpenResponse,
    IbcBasicResponse, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg,
    IbcChannelOpenResponse, IbcMsg, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg,
    IbcReceiveResponse, IbcTimeout, MessageInfo, Response, StdError, StdResult, WasmMsg,
};
use schemars::schema::SingleOrVec::Vec;

use crate::msg::PacketMsg::RandomResponse;
use crate::msg::{ExecuteMsg, InstantiateMsg, PacketMsg, QueryMsg, RandomCallback};
use crate::state::{load_callback, save_callback, Channel, Operation, StoredRandomAnswer};
use crate::utils::verify_callback;
use secret_toolkit_crypto::{sha_256, Prng};

pub const IBC_APP_VERSION: &str = "ibc-v1";
const PACKET_LIFETIME: u64 = 60 * 60;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Operation::save_last(
        deps.storage,
        Operation {
            name: "Just initialized".to_string(),
            parameters: vec![],
        },
    )?;

    Ok(Response::default().add_attribute(
        "init",
        to_binary(&Operation::get_last(deps.storage)?)?.to_string(),
    ))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::SendIbcPacket { message } => {
            let channel_id = Channel::get_last_opened(deps.storage)?;
            let packet = PacketMsg::Message {
                value: channel_id.clone() + &message,
            };

            Operation::save_last(
                deps.storage,
                Operation {
                    name: "execute".to_string(),
                    parameters: vec![format!("channel_id: {}", channel_id)],
                },
            )?;

            Ok(Response::new().add_message(IbcMsg::SendPacket {
                channel_id,
                data: to_binary(&packet)?,
                timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(PACKET_LIFETIME)),
            }))
        }

        // RequestRandom comes from a contract on the consuming side
        ExecuteMsg::RequestRandom { job_id, callback } => {
            if !verify_callback(&callback) {
                deps.api
                    .debug("Failed to verify callback! (trying to use sent_funds?)");
                return Err(StdError::generic_err("Failed to verify callback"));
            }

            let channel_id = Channel::get_last_opened(deps.storage)?;
            let packet = PacketMsg::RequestRandom {
                job_id: job_id.clone(),
                length: None,
            };

            Operation::save_last(
                deps.storage,
                Operation {
                    name: "SendIbcPacket".to_string(),
                    parameters: vec![
                        format!("channel_id: {}", channel_id),
                        format!("content: {}", job_id),
                    ],
                },
            )?;

            save_callback(deps.storage, callback)?;

            Ok(Response::new().add_message(IbcMsg::SendPacket {
                channel_id,
                data: to_binary(&packet)?,
                timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(PACKET_LIFETIME)),
            }))
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::LastIbcOperation {} => Ok(to_binary(&Operation::get_last(deps.storage)?)?),

        QueryMsg::ViewReceivedLifeAnswer {} => {
            Ok(to_binary(&StoredRandomAnswer::get(deps.storage)?)?)
        }
    }
}

#[entry_point]
pub fn ibc_channel_open(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> StdResult<IbcChannelOpenResponse> {
    match msg {
        IbcChannelOpenMsg::OpenInit { channel } => Operation::save_last(
            deps.storage,
            Operation {
                name: "ChannelOpen/Init".to_string(),
                parameters: vec![
                    format!("connection_id: {}", channel.connection_id),
                    format!("channel_id: {}", channel.endpoint.channel_id),
                    format!("port_id: {}", channel.endpoint.port_id),
                ],
            },
        )?,

        IbcChannelOpenMsg::OpenTry {
            channel,
            counterparty_version,
        } => Operation::save_last(
            deps.storage,
            Operation {
                name: "ChannelOpen/Try".to_string(),
                parameters: vec![
                    format!("counterparty_version: {}", counterparty_version),
                    format!("connection_id: {}", channel.connection_id),
                    format!("channel_id: {}", channel.endpoint.channel_id),
                    format!("port_id: {}", channel.endpoint.port_id),
                ],
            },
        )?,

        _ => Operation::save_last(
            deps.storage,
            Operation {
                name: format!("unknown channel open message: {}", to_binary(&msg)?),
                parameters: vec![],
            },
        )?,
    };

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
        IbcChannelConnectMsg::OpenAck {
            channel,
            counterparty_version,
        } => {
            Operation::save_last(
                deps.storage,
                Operation {
                    name: "ChannelConnect/Ack".to_string(),
                    parameters: vec![
                        format!("counterparty_version: {}", counterparty_version),
                        format!("connection_id: {}", channel.connection_id),
                        format!("channel_id: {}", channel.endpoint.channel_id),
                        format!("port_id: {}", channel.endpoint.port_id),
                    ],
                },
            )?;

            // save channel to state
            let channel_id = channel.endpoint.channel_id;
            Channel::save_last_opened(deps.storage, channel_id)?;
        }

        IbcChannelConnectMsg::OpenConfirm { channel } => {
            Operation::save_last(
                deps.storage,
                Operation {
                    name: "ChannelOpen/Confirm".to_string(),
                    parameters: vec![
                        format!("connection_id: {}", channel.connection_id),
                        format!("channel_id: {}", channel.endpoint.channel_id),
                        format!("port_id: {}", channel.endpoint.port_id),
                    ],
                },
            )?;

            // save channel to state
            let channel_id = channel.endpoint.channel_id;
            Channel::save_last_opened(deps.storage, channel_id)?;
        }

        _ => {
            Operation::save_last(
                deps.storage,
                Operation {
                    name: format!("unknown channel connect message: {}", to_binary(&msg)?),
                    parameters: vec![],
                },
            )?;
        }
    };

    Ok(IbcBasicResponse::default())
}

#[entry_point]
pub fn ibc_packet_receive(
    deps: DepsMut,
    env: Env,
    msg: IbcPacketReceiveMsg,
) -> StdResult<IbcReceiveResponse> {
    Operation::save_last(
        deps.storage,
        Operation {
            name: "PacketReceive".to_string(),
            parameters: vec![
                format!("packet_sequence: {}", msg.packet.sequence),
                format!("packet_data: {}", msg.packet.data),
                format!("packet_src_port_id: {}", msg.packet.src.port_id),
                format!("packet_src_channel_id: {}", msg.packet.src.channel_id),
                format!("relayer: {}", msg.relayer),
            ],
        },
    )?;

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

#[entry_point]
pub fn ibc_packet_ack(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    Operation::save_last(
        deps.storage,
        Operation {
            name: "PacketAck".to_string(),
            parameters: vec![
                format!("acknowledgement_data: {}", msg.acknowledgement.data),
                format!("original_packet_sequence: {}", msg.original_packet.sequence),
                format!("original_packet_data: {}", msg.original_packet.data),
                format!(
                    "original_packet_src_port_id: {}",
                    msg.original_packet.src.port_id
                ),
                format!(
                    "original_packet_src_channel_id: {}",
                    msg.original_packet.src.channel_id
                ),
                format!("relayer: {}", msg.relayer),
            ],
        },
    )?;

    let ack_data = from_binary(&msg.acknowledgement.data)?;
    match ack_data {
        PacketMsg::Message { .. } => Ok(IbcBasicResponse::default()),

        PacketMsg::RandomResponse { job_id, random } => {
            // todo: support more than 1 concurrent job
            let callback = load_callback(deps.storage)?;

            let msg = create_random_response_callback(callback, job_id, random)?;

            Ok(IbcBasicResponse::default().add_message(msg))
        }

        _ => Ok(IbcBasicResponse::default()),
    }
}

fn create_random_response_callback(
    callback: WasmMsg,
    job_id: String,
    random: String,
) -> StdResult<WasmMsg> {
    match callback {
        WasmMsg::Execute {
            contract_addr,
            code_hash,
            ..
        } => Ok(WasmMsg::Execute {
            contract_addr,
            code_hash,
            msg: to_binary(&RandomCallback::RandomResponse {
                job_id,
                random,
                msg: None,
            })?,
            funds: vec![],
        }),
        WasmMsg::Instantiate {
            code_id,
            code_hash,
            label,
            ..
        } => Ok(WasmMsg::Instantiate {
            code_id,
            code_hash,
            funds: vec![],
            label,
            msg: to_binary(&RandomCallback::RandomResponse {
                job_id,
                random,
                msg: None,
            })?,
        }),
        _ => Err(StdError::generic_err("Invalid callback type")),
    }
}

#[entry_point]
pub fn ibc_channel_close(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    match msg {
        IbcChannelCloseMsg::CloseInit { channel } => Operation::save_last(
            deps.storage,
            Operation {
                name: "ChannelClose/Init".to_string(),
                parameters: vec![
                    format!("connection_id: {}", channel.connection_id),
                    format!("channel_id: {}", channel.endpoint.channel_id),
                    format!("port_id: {}", channel.endpoint.port_id),
                ],
            },
        )?,

        IbcChannelCloseMsg::CloseConfirm { channel } => Operation::save_last(
            deps.storage,
            Operation {
                name: "ChannelClose/Confirm".to_string(),
                parameters: vec![
                    format!("connection_id: {}", channel.connection_id),
                    format!("channel_id: {}", channel.endpoint.channel_id),
                    format!("port_id: {}", channel.endpoint.port_id),
                ],
            },
        )?,

        _ => Operation::save_last(
            deps.storage,
            Operation {
                name: format!("unknown channel close message: {}", to_binary(&msg)?),
                parameters: vec![],
            },
        )?,
    };

    Ok(IbcBasicResponse::default())
}

#[entry_point]
pub fn ibc_packet_timeout(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    Operation::save_last(
        deps.storage,
        Operation {
            name: "PacketTimeout".to_string(),
            parameters: vec![
                format!("original_packet_sequence: {}", msg.packet.sequence),
                format!("original_packet_data: {}", msg.packet.data),
                format!("original_packet_src_port_id: {}", msg.packet.src.port_id),
                format!(
                    "original_packet_src_channel_id: {}",
                    msg.packet.src.channel_id
                ),
                format!("relayer: {}", msg.relayer),
            ],
        },
    )?;

    Ok(IbcBasicResponse::default())
}
