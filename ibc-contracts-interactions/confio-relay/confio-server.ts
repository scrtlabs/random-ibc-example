import { sha256 } from "@noble/hashes/sha256";
import * as fs from "fs";
import {
  SecretNetworkClient,
  toHex,
  Tx,
  Wallet,
} from "secretjs";
import { AminoWallet } from "secretjs/dist/wallet_amino";
import {
  storeContracts,
  waitForBlocks,
  Contract,
  instantiateContracts,
  createIbcConnection,
  createIbcChannel,
  loopRelayer
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

  contract.codeId = Number(tx.arrayLog.find((x) => x.key === "code_id").value);

  console.log("Instantiating contracts on " + chainName + "...");
  tx = await instantiateContracts(client, [contract], { init: {} });

  contract.address = tx.arrayLog.find((x) => x.key === "contract_address").value;
  contract.ibcPortId = "wasm." + contract.address;
  console.log("contract on " + chainName + " got address:", contract.address);

  return contract
}
let channelId1 = "";
let channelId2 = "";
const runRelayer = async () => {
  const linkPromise = createIbcConnection();

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

  wasmCode = fs.readFileSync(`${__dirname}/../ibc-contract/ibc.wasm`) as Uint8Array;
  contracts[chainNames[0]].codeHash = toHex(sha256(wasmCode));
  contracts[chainNames[1]].codeHash = toHex(sha256(wasmCode));

  contracts[chainNames[0]] = await uploadAndInstantiateContract(chainNames[0], accounts[0].secretjs);
  contracts[chainNames[1]] = await uploadAndInstantiateContract(chainNames[1], accounts2[0].secretjs);

  fs.writeFileSync("../contract-addresses.log", contracts[chainNames[0]].address + "\n");
  fs.appendFileSync("../contract-addresses.log", contracts[chainNames[1]].address);

  console.log("Waiting for IBC connection...");
  const link = await linkPromise;

  console.log("Creating IBC channel...");
  const channels = await createIbcChannel(link, contracts[chainNames[0]].ibcPortId, contracts[chainNames[1]].ibcPortId);

  channelId1 = channels.src.channelId;
  channelId2 = channels.dest.channelId;
  console.log("channelId1:...", channelId1);
  console.log("channelId2:...", channelId2);
  console.log("Looping relayer...");
  loopRelayer(link);
}

runRelayer();
