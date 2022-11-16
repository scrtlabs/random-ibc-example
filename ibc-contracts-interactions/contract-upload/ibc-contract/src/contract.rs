use cosmwasm_std::{
    Binary, Deps, DepsMut, entry_point, Env, from_binary, Ibc3ChannelOpenResponse,
    IbcBasicResponse, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg,
    IbcChannelOpenResponse, IbcMsg, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg,
    IbcReceiveResponse, IbcTimeout, MessageInfo, Response, StdResult, to_binary,
};

use serde::{Deserialize, Serialize};

use crate::msg::{ExecuteMsg, InstantiateMsg, PacketMsg, QueryMsg};
use crate::state::{Channel, Operation, StoredRandomness};

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

    Ok(
        Response::default().add_attribute(
            "init",
            to_binary(&Operation::get_last(deps.storage)?)?.to_string(),
        )
    )
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    Operation::save_last(deps.storage, Operation {
        name: "execute".to_string(),
        parameters: vec![]
    })?;

    match msg {
        ExecuteMsg::SendIbcPacket { message } => {
            let channel_id = Channel::get_last_opened(deps.storage)?;
            let packet = PacketMsg::Message {
                value: channel_id.clone() + &message,
            };

            return Ok(Response::new().add_message(IbcMsg::SendPacket {
                channel_id,
                data: to_binary(&packet)?,
                timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(PACKET_LIFETIME)),
            }));
        }

        ExecuteMsg::RequestRandomnessFromOtherChain {} => {
            let channel_id = Channel::get_last_opened(deps.storage)?;
            let packet = PacketMsg::RequestRandomness { job_id: "request-randomness-id".to_string() };

            Operation::save_last(deps.storage, Operation {
                name: "SendIbcPacket".to_string(),
                parameters: vec![
                    format!("channel_id: {}", channel_id),
                    format!("content: {}", "job_id: request-randomness-id"),
                ]
            })?;

            return Ok(Response::new().add_message(IbcMsg::SendPacket {
                channel_id,
                data: to_binary(&packet)?,
                timeout: IbcTimeout::with_timestamp(env.block.time.plus_seconds(PACKET_LIFETIME)),
            }));
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::LastIbcOperation {} => Ok(
            to_binary(
                &Operation::get_last(deps.storage)?
            )?
        ),

        QueryMsg::ViewReceivedRandomness {} => Ok(
            to_binary(
                &StoredRandomness::get(deps.storage)?
            )?
        )
    }
}

#[entry_point]
pub fn ibc_channel_open(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> StdResult<IbcChannelOpenResponse> {
    match msg {
        IbcChannelOpenMsg::OpenInit { channel } => Operation::save_last(deps.storage, Operation {
            name: "ChannelOpen/Init".to_string(),
            parameters: vec![
                format!("connection_id: {}", channel.connection_id),
                format!("channel_id: {}", channel.endpoint.channel_id),
                format!("port_id: {}", channel.endpoint.port_id),
            ],
        })?,

        IbcChannelOpenMsg::OpenTry {
            channel,
            counterparty_version,
        } => Operation::save_last(deps.storage, Operation {
            name: "ChannelOpen/Try".to_string(),
            parameters: vec![
                format!("counterparty_version: {}", counterparty_version),
                format!("connection_id: {}", channel.connection_id),
                format!("channel_id: {}", channel.endpoint.channel_id),
                format!("port_id: {}", channel.endpoint.port_id),
            ],
        })?,

        _ => Operation::save_last(deps.storage, Operation {
            name: format!("unknown channel open message: {}", to_binary(&msg)?),
            parameters: vec![],
        })?
    };

    Ok(Some(Ibc3ChannelOpenResponse {
        version: IBC_APP_VERSION.to_string(),
    }))
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ContractInfo {
    pub address: String,
    pub hash: String,
}

pub fn is_reply_on(num: u64) -> bool {
    match num {
        2 | 6 => true,
        _ => false,
    }
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
            Operation::save_last(deps.storage, Operation {
                name: "ChannelConnect/Ack".to_string(),
                parameters: vec![
                    format!("counterparty_version: {}", counterparty_version),
                    format!("connection_id: {}", channel.connection_id),
                    format!("channel_id: {}", channel.endpoint.channel_id),
                    format!("port_id: {}", channel.endpoint.port_id),
                ],
            })?;

            // save channel to state
            let channel_id = channel.endpoint.channel_id;
            Channel::save_last_opened(deps.storage, channel_id)?;
        },

        IbcChannelConnectMsg::OpenConfirm { channel } => {
            Operation::save_last(
                deps.storage, Operation {
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
            Operation::save_last(deps.storage, Operation {
                name: format!("unknown channel connect message: {}", to_binary(&msg)?),
                parameters: vec![],
            })?;
        }
    };

    Ok(IbcBasicResponse::default())
}

#[entry_point]
pub fn ibc_channel_close(
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    match msg {
        IbcChannelCloseMsg::CloseInit { channel } => Operation::save_last(deps.storage, Operation {
            name: "ChannelClose/Init".to_string(),
            parameters: vec![
                format!("connection_id: {}", channel.connection_id),
                format!("channel_id: {}", channel.endpoint.channel_id),
                format!("port_id: {}", channel.endpoint.port_id),
            ],
        })?,

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

        _ => Operation::save_last(deps.storage, Operation {
            name: format!("unknown channel close message: {}", to_binary(&msg)?),
            parameters: vec![],
        })?
    };

    Ok(IbcBasicResponse::default())
}

fn ack_success() -> Binary {
    let res = PacketMsg::ReceiveRandomness { random_value: 52 };
    to_binary(&res).unwrap()
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
        PacketMsg::Test { .. } => {},
        PacketMsg::Message { .. } => {},

        PacketMsg::RequestRandomness { .. } => {
            let res = PacketMsg::ReceiveRandomness {
                random_value: 62, // I swear I chose this number at random!
            };
            let acknowledgement_data = to_binary(&res).unwrap();

            response = response.set_ack(acknowledgement_data);
        },

        _ => {}
    }

    // Ok(
    //     response.set_ack(ack_success())
    // )

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
                format!("original_packet_src_port_id: {}", msg.original_packet.src.port_id),
                format!("original_packet_src_channel_id: {}", msg.original_packet.src.channel_id),
                format!("relayer: {}", msg.relayer),
            ],
        },
    )?;

    let ack_data = from_binary(&msg.acknowledgement.data)?;
    match ack_data {
        PacketMsg::ReceiveRandomness { random_value } => {
            StoredRandomness::save(
                deps.storage,
                random_value,
            )?;
        },

        _ => {}
    }

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
                format!("original_packet_src_channel_id: {}", msg.packet.src.channel_id),
                format!("relayer: {}", msg.relayer),
            ],
        },
    )?;

    Ok(IbcBasicResponse::default())
}