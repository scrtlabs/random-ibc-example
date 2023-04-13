import { IbcClient, Link } from "@confio/relayer";
import { ChannelPair } from "@confio/relayer/build/lib/link";
import { GasPrice } from "@cosmjs/stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { stringToPath } from "@cosmjs/crypto";

import {
  SecretNetworkClient,
  MsgStoreCode,
  MsgInstantiateContract,
  TxResponse,
  TxResultCode,
  Wallet,
} from "secretjs";
// import { State as ConnectionState } from "secretjs/src/grpc_gateway/ibc/core/connection/v1/connection.pb";
// import { State as ChannelState } from "secretjs/src/grpc_gateway/ibc/core/channel/v1/channel.pb";
import { Order } from "secretjs/dist/protobuf/ibc/core/channel/v1/channel";

export const chain1RPC = "http://localsecret-1:26657";
export const chain2RPC = "http://localsecret-2:26657";

export class Contract {
  address: string = "";
  codeId: number = 0;
  ibcPortId: string = "";
  codeHash: string = "";
}

export async function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function waitForBlocks(chainId: string, url: string) {
  const secretjs = new SecretNetworkClient({
    url,
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

// }

export async function storeContracts(
  account: SecretNetworkClient,
  wasms: Uint8Array[]
) {
  const tx: TxResponse = await account.tx.broadcast(
    wasms.map(wasm => new MsgStoreCode(
      {
        sender: account.address,
        wasm_byte_code: wasm,
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
  const tx: TxResponse = await account.tx.broadcast(
    contracts.map(contract => new MsgInstantiateContract(
      {
        sender: account.address,
        code_id: contract.codeId,
        code_hash: contract.codeHash,
        init_msg: initMsg,
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
  const signerA = await DirectSecp256k1HdWallet.fromMnemonic(
      "word twist toast cloth movie predict advance crumble escape whale sail such angry muffin balcony keen move employ cook valve hurt glimpse breeze brick", // account d
      { hdPaths: [stringToPath("m/44'/529'/0'/0/0")], prefix: "secret" },
  );
  const [account] = await signerA.getAccounts();

  const signerB = signerA;

  // Create IBC Client for chain A
  const clientA = await IbcClient.connectWithSigner(
      chain1RPC,
      signerA,
      account.address,
      {
        prefix: "secret",
        gasPrice: GasPrice.fromString("0.25uscrt"),
        estimatedBlockTime: 750,
        estimatedIndexerTime: 500,
      },
  );

  // Create IBC Client for chain A
  const clientB = await IbcClient.connectWithSigner(
      chain2RPC,
      signerB,
      account.address,
      {
        prefix: "secret",
        gasPrice: GasPrice.fromString("0.25uscrt"),
        estimatedBlockTime: 750,
        estimatedIndexerTime: 500,
      },
  );

  // Create new connection for the 2 clients
  return await Link.createWithNewConnections(clientA, clientB);
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
