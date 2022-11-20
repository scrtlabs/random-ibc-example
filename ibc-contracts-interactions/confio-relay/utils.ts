import { IbcClient, Link } from "@confio/relayer";
import { ChannelPair } from "@confio/relayer/build/lib/link";
import { GasPrice } from "@cosmjs/stargate";
import {
  SecretNetworkClient,
  MsgStoreCode,
  MsgInstantiateContract,
  Tx,
  TxResultCode,
  Wallet,
} from "secretjs";
import { State as ConnectionState } from "secretjs/dist/protobuf_stuff/ibc/core/connection/v1/connection";
import { Order, State as ChannelState } from "secretjs/dist/protobuf_stuff/ibc/core/channel/v1/channel";


export class Contract {
  address: string;
  codeId: number;
  ibcPortId: string;
  codeHash: string;
}

export async function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function waitForBlocks(chainId: string) {
  const secretjs = await SecretNetworkClient.create({
    grpcWebUrl: "http://localhost:9091",
    chainId,
  });

  console.log(`Waiting for blocks on ${chainId}...`);
  while (true) {
    try {
      const { block } = await secretjs.query.tendermint.getLatestBlock({});

      if (Number(block?.header?.height) >= 1) {
        console.log(`Current block on ${chainId}: ${block!.header!.height}`);
        break;
      }
    } catch (e) {
      console.error("block error:", e);
    }
    await sleep(100);
  }
}

export async function waitForIBCConnection(
  chainId: string,
  grpcWebUrl: string
) {
  const secretjs = await SecretNetworkClient.create({
    grpcWebUrl,
    chainId,
  });

  console.log("Waiting for open connections on", chainId + "...");
  while (true) {
    try {
      const { connections } = await secretjs.query.ibc_connection.connections(
        {}
      );

      if (
        connections.length >= 1 &&
        connections[0].state === ConnectionState.STATE_OPEN
      ) {
        console.log("Found an open connection on", chainId);
        break;
      }
    } catch (e) {
      console.error("IBC error:", e, "on chain", chainId);
    }
    await sleep(100);
  }
}

export async function waitForIBCChannel(
  chainId: string,
  grpcWebUrl: string,
  channelId: string
) {
  const secretjs = await SecretNetworkClient.create({
    grpcWebUrl,
    chainId,
  });

  console.log(`Waiting for ${channelId} on ${chainId}...`);
  outter: while (true) {
    try {
      const { channels } = await secretjs.query.ibc_channel.channels({});

      for (const c of channels) {
        if (c.channelId === channelId && c.state == ChannelState.STATE_OPEN) {
          console.log(`${channelId} is open on ${chainId}`);
          break outter;
        }
      }
    } catch (e) {
      console.error("IBC error:", e, "on chain", chainId);
    }
    await sleep(100);
  }
}

export async function storeContracts(
  account: SecretNetworkClient,
  wasms: Uint8Array[]
) {
  const tx: Tx = await account.tx.broadcast(
    wasms.map(wasm => new MsgStoreCode(
      {
        sender: account.address,
        wasmByteCode: wasm,
        source: "",
        builder: "",
      }
    )),
    { gasLimit: 5_000_000 }
  );

  if (tx.code !== TxResultCode.Success) {
    console.error(tx.rawLog);
  }

  return tx;
}

export async function instantiateContracts(
  account: SecretNetworkClient,
  contracts: Contract[],
  initMsg: {},
) {
  const tx: Tx = await account.tx.broadcast(
    contracts.map(contract => new MsgInstantiateContract(
      {
        sender: account.address,
        codeId: contract.codeId,
        codeHash: contract.codeHash,
        initMsg: initMsg,
        label: `v1-${Date.now()}`,
      }
    )),
    { gasLimit: 300_000 }
  );
  if (tx.code !== TxResultCode.Success) {
    console.error(tx.rawLog);
  }

  return tx;
}

export async function createIbcConnection(): Promise<Link> {
  // Create signers as LocalSecret account d
  // (Both sides are localsecret so same account can be used on both sides)
  const signerA = new Wallet(
    "word twist toast cloth movie predict advance crumble escape whale sail such angry muffin balcony keen move employ cook valve hurt glimpse breeze brick"
  );
  const signerB = signerA;

  // Create IBC Client for chain A
  const clientA = await IbcClient.connectWithSigner("http://localhost:26657", signerA, signerA.address, {
    prefix: "secret",
    gasPrice: GasPrice.fromString("0.25uscrt"),
    estimatedBlockTime: 5750,
    estimatedIndexerTime: 500,
  });
  console.group("IBC client for chain A");
  console.log(JSON.stringify(clientA));
  console.groupEnd();

  // Create IBC Client for chain A
  const clientB = await IbcClient.connectWithSigner("http://localhost:36657", signerB, signerB.address, {
    prefix: "secret",
    gasPrice: GasPrice.fromString("0.25uscrt"),
    estimatedBlockTime: 5750,
    estimatedIndexerTime: 500,
  });
  console.group("IBC client for chain B");
  console.log(JSON.stringify(clientB));
  console.groupEnd();

  // Create new connectiosn for the 2 clients
  const link = await Link.createWithNewConnections(clientA, clientB);
  return link;
}
export async function createIbcChannel(link: Link, srcPort: string, destPort: string): Promise<ChannelPair> {
  await Promise.all([link.updateClient("A"), link.updateClient("B")]);

  // Create a channel for the connections
  const channels = await link.createChannel("A", srcPort, destPort, Order.ORDER_UNORDERED, "ibc-v1");
  return channels;
}

export async function loopRelayer(link: Link) {
  let nextRelay = {};
  while (true) {
    try {
      nextRelay = await link.relayAll();
      await Promise.all([link.updateClient("A"), link.updateClient("B")]);
    } catch (e) {
      console.error(`Caught error: `, e);
    }
    await sleep(5000);
  }
}
