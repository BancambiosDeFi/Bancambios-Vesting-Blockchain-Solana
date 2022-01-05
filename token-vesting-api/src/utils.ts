import { AccountInfo, PublicKey, LAMPORTS_PER_SOL } from "@solana/web3.js";
import crypto from "crypto";

export function getTokenPoolSeed(
  programId: PublicKey,
  vestingName: string
): string {
  return getSeed(programId.toBase58(), "TokenPool", vestingName);
}

export function getVestingTypeSeed(
  programId: PublicKey,
  vestingName: string
): string {
  return getSeed(programId.toBase58(), "VestingType", vestingName);
}

export function getVestingSeed(
  programId: PublicKey,
  tokenPool: PublicKey,
  receiver: PublicKey
): string {
  return getSeed(
    programId.toBase58(),
    "VestingAccount",
    tokenPool.toBase58(),
    receiver.toBase58()
  );
}

function getSeed(...params: string[]): string {
  const concatenated = params.reduce((sum, current) => sum + current, "");
  const digest = crypto
    .createHash("sha256")
    .update(concatenated)
    .digest("base64");
  return digest.slice(-32);
}

export type AccountValidation = (
  account: AccountInfo<Buffer> | null,
  name: string
) => void;

export function validateAccountExist(
  account: AccountInfo<Buffer> | null,
  name: string
) {
  if (account === null) {
    throw Error(`${name} does not exist`);
  }
}

export function validateAccountDoesNotExist(
  account: AccountInfo<Buffer> | null,
  name: string
) {
  if (account !== null) {
    throw Error(`${name} already exist`);
  }
}

export const FloatAsU64Creator = {
  serialize: (value: number, writer: any) => {
    writer.writeU64(value * LAMPORTS_PER_SOL);
  },
  deserialize: (reader: any): number => {
    let value = reader.readU64();
    return value / LAMPORTS_PER_SOL;
  },
};
