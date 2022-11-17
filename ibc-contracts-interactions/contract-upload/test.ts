import { sha256 } from "@noble/hashes/sha256";
import { execSync } from "child_process";
import * as fs from "fs";
import {
  fromBase64,
  fromUtf8,
  SecretNetworkClient,
  toHex,
  Tx,
  TxResultCode,
  Wallet,
} from "secretjs";
import { AminoWallet } from "secretjs/dist/wallet_amino";
import {
  ibcDenom,
  sleep,
  storeContracts,
  waitForBlocks,
  waitForIBCChannel,
  Contract,
  instantiateContracts,
  cleanBytes,
} from "./utils";

type Account = {
  address: string;
  mnemonic: string;
  walletAmino: AminoWallet;
  walletProto: Wallet;
  secretjs: SecretNetworkClient;
};

// @ts-ignore
// accounts on secretdev-1
const accounts: Account[] = new Array(2);
const accounts2: Account[] = new Array(2);
let readonly: SecretNetworkClient;
let readonly2: SecretNetworkClient;

let wasmCode: Uint8Array;
const chainNames = ["secretdev-1", "secretdev-2"]
const contracts = {
  [chainNames[0]]: new Contract,
  [chainNames[1]]: new Contract,
};

const populateAccounts = async (accountList, mnemonics, chainId, endpoint) => {
  for (let i = 0; i < mnemonics.length; i++) {
    const mnemonic = mnemonics[i];
    const walletAmino = new AminoWallet(mnemonic);
    accountList[i] = {
      address: walletAmino.address,
      mnemonic: mnemonic,
      walletAmino,
      walletProto: new Wallet(mnemonic),
      secretjs: await SecretNetworkClient.create({
        grpcWebUrl: endpoint,
        wallet: walletAmino,
        walletAddress: walletAmino.address,
        chainId,
      }),
    };
  }
}

async function uploadAndInstantiateContract(chainName: string, client: SecretNetworkClient) {
  let contract = new Contract
  console.log("Storing contracts on " + chainName + "...");

  let tx: Tx = await storeContracts(client, [wasmCode]);
  expect(tx.code).toBe(TxResultCode.Success);

  contract.codeId = Number(tx.arrayLog.find((x) => x.key === "code_id").value);

  console.log("Instantiating contracts on " + chainName + "...");
  tx = await instantiateContracts(client, [contract], { init: {} });
  expect(tx.code).toBe(TxResultCode.Success);

  contract.address = tx.arrayLog.find((x) => x.key === "contract_address").value;
  contract.ibcPortId = "wasm." + contract.address;
  console.log("contract on " + chainName + " got address:", contract.address);

  return contract
}

beforeAll(async () => {
  const mnemonics = [
    "chair love bleak wonder skirt permit say assist aunt credit roast size obtain minute throw sand usual age smart exact enough room shadow charge",
    "word twist toast cloth movie predict advance crumble escape whale sail such angry muffin balcony keen move employ cook valve hurt glimpse breeze brick",
  ];

  await Promise.all([
    // Create clients for existing wallets in the chains
    populateAccounts(accounts, mnemonics, chainNames[0], "http://localhost:9091"),
    populateAccounts(accounts2, mnemonics, chainNames[1], "http://localhost:9391"),

    // Create readonly clients
    SecretNetworkClient.create({ chainId: chainNames[0], grpcWebUrl: "http://localhost:9091" }).then(result => readonly = result),
    SecretNetworkClient.create({ chainId: chainNames[1], grpcWebUrl: "http://localhost:9391" }).then(result => readonly2 = result),

    // Wait for the chains to be running
    waitForBlocks(chainNames[0]),
    waitForBlocks(chainNames[1]),
  ]);

  wasmCode = fs.readFileSync(`${__dirname}/ibc-contract/ibc.wasm`) as Uint8Array;
  contracts[chainNames[0]].codeHash = toHex(sha256(wasmCode));
  contracts[chainNames[1]].codeHash = toHex(sha256(wasmCode));

  contracts[chainNames[0]] = await uploadAndInstantiateContract(chainNames[0], accounts[0].secretjs);
  contracts[chainNames[1]] = await uploadAndInstantiateContract(chainNames[1], accounts2[0].secretjs);

  fs.writeFileSync("./contract-addresses.log", contracts[chainNames[0]].address + "\n");
  fs.appendFileSync("./contract-addresses.log", contracts[chainNames[1]].address);
});

describe("IBC", () => {
  test.only("just setup contracts", async () => {
    console.log("empty test to run the beforeAll");
  });

  test("transfer sanity", async () => {
    console.log("starting transfer sanity");
    const denom = ibcDenom(
      [{
        portId: "transfer",
        channelId: "channel-0",
      },],
      "uscrt"
    );
    const { balance: balanceBefore } = await readonly2.query.bank.balance({
      address: accounts2[0].address,
      denom,
    });
    const amountBefore = Number(balanceBefore?.amount ?? "0");

    console.log("starting transfer");
    const result = await accounts[0].secretjs.tx.ibc.transfer({
      receiver: accounts2[0].address,
      sender: accounts[0].address,
      sourceChannel: "channel-0",
      sourcePort: "transfer",
      token: {
        denom: "uscrt",
        amount: "1",
      },
      timeoutTimestampSec: String(Math.floor(Date.now() / 1000 + 30)),
    });

    if (result.code !== 0) {
      console.error(result.rawLog);
    }

    expect(result.code).toBe(TxResultCode.Success);

    // checking ack/timeout on secretdev-1 might be cleaner

    console.log("starting query");
    while (true) {
      try {
        const { balance: balanceAfter } = await readonly2.query.bank.balance({
          address: accounts2[0].address,
          denom,
        });
        const amountAfter = Number(balanceAfter?.amount ?? "0");

        if (amountAfter === amountBefore + 1) {
          break;
        }
      } catch (e) {
        console.error("ibc denom balance error:", e);
      }
      await sleep(200);
    }
    expect(true).toBe(true);
  }, 30_000 /* 30 seconds */);

  test.skip("contracts sanity", async () => {
    const command =
      "docker exec hermes-relayer hermes " +
      "--config /home/hermes-user/.hermes/alternative-config.toml " +
      "create channel " +
      "--a-chain secretdev-1 " +
      `--a-port ${contracts[chainNames[0]].ibcPortId} ` +
      `--b-port ${contracts[chainNames[1]].ibcPortId} ` +
      "--a-connection connection-0";

    console.log("calling relayer with command:", command);
    const result = execSync(command);

    const trimmedResult = result.toString().replace(/\s/g, "");

    const myRegexp = /ChannelId\("(channel-\d+)"/g;
    const channelId = myRegexp.exec(trimmedResult)[1];

    await waitForIBCChannel(chainNames[0], "http://localhost:9091", channelId);

    await waitForIBCChannel(chainNames[1], "http://localhost:9391", channelId);

    const tx = await accounts[0].secretjs.tx.compute.executeContract(
      {
        sender: accounts[0].address,
        contractAddress: contracts[chainNames[0]].address,
        codeHash: contracts[chainNames[0]].codeHash,
        msg: {
          send_ibc_packet: {
            message: "hello from test",
          },
        },
      },
      { gasLimit: 250_000 }
    );
    console.log("tx", tx);
    if (tx.code !== TxResultCode.Success) {
      console.error(tx.rawLog);
    }
    expect(tx.code).toBe(TxResultCode.Success);
    console.log(
      "tx after triggering ibc send endpoint",
      JSON.stringify(cleanBytes(tx), null, 2)
    );

    expect(tx.arrayLog.find((x) => x.key === "packet_data").value).toBe(
      `{"message":{"value":"${channelId}hello from test"}}`
    );

    const packetSendCommand =
      "docker exec hermes-relayer hermes " +
      "--config /home/hermes-user/.hermes/alternative-config.toml " +
      "tx packet-recv --dst-chain secretdev-2 --src-chain secretdev-1 " +
      `--src-port ${contracts[chainNames[0]].ibcPortId} ` +
      `--src-channel ${channelId}`;

    console.log(
      "calling docker exec on relayer with command",
      packetSendCommand
    );
    let packetSendResult = execSync(packetSendCommand);
    console.log(
      "finished executing command, result:",
      packetSendResult.toString()
    );

    const packetAckCommand =
      "docker exec hermes-relayer hermes " +
      "--config /home/hermes-user/.hermes/alternative-config.toml " +
      "tx packet-ack --dst-chain secretdev-1 --src-chain secretdev-2 " +
      `--src-port ${contracts[chainNames[0]].ibcPortId} ` +
      `--src-channel ${channelId}`;

    console.log(
      "calling docker exec on relayer with command",
      packetAckCommand
    );
    const packetAckResult = execSync(packetAckCommand);
    console.log(
      "finished executing command, result:",
      packetAckResult.toString()
    );

    let queryResult: any =
      await accounts[0].secretjs.query.compute.queryContract({
        contractAddress: contracts[chainNames[0]].address,
        codeHash: contracts[chainNames[0]].codeHash,
        query: {
          last_ibc_operation: {},
        },
      });

    const ack = fromUtf8(fromBase64(queryResult));

    expect(ack).toBe(`recv${channelId}hello from test`);

    queryResult = await accounts2[0].secretjs.query.compute.queryContract({
      contractAddress: contracts[chainNames[1]].address,
      codeHash: contracts[chainNames[1]].codeHash,
      query: {
        last_ibc_operation: {},
      },
    });

    expect(queryResult).toBe(`no ack yet`);

    queryResult = await accounts[0].secretjs.query.compute.queryContract({
      contractAddress: contracts[chainNames[0]].address,
      codeHash: contracts[chainNames[0]].codeHash,
      query: {
        last_ibc_operation: {},
      },
    });

    expect(queryResult).toBe(`no receive yet`);

    queryResult = await accounts2[0].secretjs.query.compute.queryContract({
      contractAddress: contracts[chainNames[1]].address,
      codeHash: contracts[chainNames[1]].codeHash,
      query: {
        last_ibc_operation: {},
      },
    });

    expect(queryResult).toBe(`${channelId}hello from test`);
  }, 80_000 /* 80 seconds */);
});
