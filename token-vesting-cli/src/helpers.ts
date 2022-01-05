import * as web3 from "@solana/web3.js";
import {
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import * as fs from "mz/fs";
import { TokenVesting } from "token-vesting-api/dist/token-vesing";

export function getEnvString(name: string): string {
  const value = process.env[name];
  if (value === undefined) {
    throw new Error(`Missing ${name} environment variable`);
  }

  return value;
}

export async function bootstrapTools(vestingName?: string | undefined) {
  let connection = new Connection(
    getEnvString("CONNECTION_ENDPOINT"),
    "confirmed"
  );
  const secretKey = Uint8Array.from(
    JSON.parse(getEnvString("SENDER_SECRET_KEY"))
  );
  let payer = Keypair.fromSecretKey(secretKey);
  let programId = new PublicKey(getEnvString("VESTING_PROGRAM_ID"));
  const mint = new PublicKey(getEnvString("TOKEN_MINT"));
  const safeVestingName = vestingName ?? getEnvString("VESTING_NAME");

  if (getEnvString("SENDER_SECRET_KEY") == "true") {
    await requestAirdrop(connection, payer);
  }

  const tokenVesting = new TokenVesting(
    connection,
    programId,
    mint,
    payer.publicKey,
    safeVestingName
  );
  const processTransaction = async (transaction: web3.Transaction) => {
    console.log("Sending transaction...");
    await sendAndConfirmTransaction(connection, transaction, [payer]);
    console.log("Transaction confirmed!");
  };
  return { tokenVesting, processTransaction, payer };
}

async function requestAirdrop(connection: Connection, payer: Keypair) {
  const currentBalance = await connection.getBalance(payer.publicKey);
  if (currentBalance / web3.LAMPORTS_PER_SOL > 10) {
    return;
  }

  let airdropSignature = await connection.requestAirdrop(
    payer.publicKey,
    5 * web3.LAMPORTS_PER_SOL
  );
  await connection.confirmTransaction(airdropSignature);
}
