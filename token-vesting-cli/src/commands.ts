import * as web3 from "@solana/web3.js";
import * as helpers from "./helpers";
import {
  ChangeVestingTypeScheduleInstruction,
  CreateVestingAccountInstruction,
  CreateVestingTypeInstruction,
  WithdrawExcessiveFromPoolInstruction,
  WithdrawFromVestingInstruction,
} from "token-vesting-api/dist/schema";
import BN from "bn.js";
import BigNumber from "bignumber.js";
import { PathLike, readFileSync } from "fs";
import { parse } from "csv-parse";
import * as yup from "yup";

export async function createVestingTypeCommand(
  amountSol: number,
  initialUnlock: number,
  startTime: number,
  endTime: number,
  unlockPeriod: number,
  cliff: number
): Promise<string> {
  let { tokenVesting, processTransaction } = await helpers.bootstrapTools();

  const transaction = await tokenVesting.createVestingType(
    new BN(
      new BigNumber(amountSol).multipliedBy(web3.LAMPORTS_PER_SOL).toString()
    ),
    new CreateVestingTypeInstruction(
      initialUnlock,
      startTime,
      endTime,
      unlockPeriod,
      cliff
    )
  );

  await processTransaction(transaction);
  return await getVestingTypeStatisticCommand();
}

export async function changeVestingTypeScheduleCommand(
  initialUnlock: number,
  startTime: number,
  endTime: number,
  unlockPeriod: number,
  cliff: number
): Promise<string> {
  let { tokenVesting, processTransaction } = await helpers.bootstrapTools();
  const transaction: web3.Transaction =
    await tokenVesting.changeVestingTypeSchedule(
      new ChangeVestingTypeScheduleInstruction(
        initialUnlock,
        startTime,
        endTime,
        unlockPeriod,
        cliff
      )
    );
  await processTransaction(transaction);
  return await getVestingTypeStatisticCommand();
}

export async function withdrawExcessiveFromPoolCommand(
  amountSol: string
): Promise<string> {
  let { tokenVesting, processTransaction, payer } =
    await helpers.bootstrapTools();
  const tokensBefore = (
    await tokenVesting.getAssociatedTokenAccount(payer.publicKey)
  ).amount;
  const transaction: web3.Transaction =
    await tokenVesting.withdrawExcessiveFromPool(
      new WithdrawExcessiveFromPoolInstruction(
        new BN(
          new BigNumber(amountSol)
            .multipliedBy(web3.LAMPORTS_PER_SOL)
            .toString()
        )
      )
    );
  await processTransaction(transaction);

  const tokensAfter = (
    await tokenVesting.getAssociatedTokenAccount(payer.publicKey)
  ).amount;
  return `Lamports owned before withdraw: ${tokensBefore}, after: ${tokensAfter}\n${await getVestingTypeStatisticCommand()}`;
}

export async function createVestingAccountCommand(
  receiver: string,
  amountSol: string
): Promise<string> {
  let { tokenVesting, processTransaction } = await helpers.bootstrapTools();
  const receiverPubkey = new web3.PublicKey(receiver);
  const transaction: web3.Transaction = await tokenVesting.createVestingAccount(
    receiverPubkey,
    new CreateVestingAccountInstruction(
      new BN(
        new BigNumber(amountSol).multipliedBy(web3.LAMPORTS_PER_SOL).toString()
      )
    )
  );

  await processTransaction(transaction);

  return await getVestingStatisticCommand(receiver);
}

export async function withdrawFromVesting(
  amountSol: string,
  receiver: string
): Promise<string> {
  let { tokenVesting, processTransaction } = await helpers.bootstrapTools();
  const receiverPubkey = new web3.PublicKey(receiver);
  const tokensBefore = (
    await tokenVesting.getAssociatedTokenAccount(receiverPubkey)
  ).amount;
  const transaction: web3.Transaction = await tokenVesting.withdrawFromVesting(
    receiverPubkey,
    new WithdrawFromVestingInstruction(
      new BN(
        new BigNumber(amountSol).multipliedBy(web3.LAMPORTS_PER_SOL).toString()
      )
    )
  );
  await processTransaction(transaction);

  const tokensAfter = (
    await tokenVesting.getAssociatedTokenAccount(receiverPubkey)
  ).amount;
  return `Lamports owned before withdraw: ${tokensBefore}, after: ${tokensAfter}\n${await getVestingStatisticCommand(
    receiver
  )}`;
}

export async function getVestingStatisticCommand(
  receiver: string
): Promise<string> {
  let { tokenVesting } = await helpers.bootstrapTools();
  const receiverPubkey = new web3.PublicKey(receiver);
  return (await tokenVesting.getVestingStatistic(receiverPubkey)).toString();
}

export async function getVestingTypeStatisticCommand(): Promise<string> {
  let { tokenVesting } = await helpers.bootstrapTools();
  return (await tokenVesting.getVestingTypeStatistic()).toString();
}

export function test() {
  console.log("test");
}

export async function fillVestingTypesFromCsv(fileName: PathLike) {
  const content = readFileSync(fileName, "utf-8");
  const parser = parse(content, { columns: true });

  let schema = yup.object().shape({
    vestingName: yup.string().required(),
    initialUnlock: yup.string().required(),
    startTime: yup.string().required(),
    endTime: yup.string().required(),
    unlockPeriod: yup.string().required(),
    amountSol: yup.string().required(),
  });

  for await (const record of parser) {
    try {
      await schema.validate(record);
    } catch (e) {
      const validationMessage = (e as Error).message;
      throw Error(
        `Error while validating csv input: ${validationMessage}, for data: ${JSON.stringify(
          record
        )}`
      );
    }
    const {
      vestingName,
      initialUnlock,
      startTime,
      endTime,
      unlockPeriod,
      cliff,
      amountSol,
    } = record;
    let { tokenVesting, processTransaction } = await helpers.bootstrapTools(
      vestingName
    );
    const toTimestamp = (date: string) => new Date(date).getTime() / 1000;
    const transaction = await tokenVesting.createVestingType(
      new BN(
        new BigNumber(amountSol).multipliedBy(web3.LAMPORTS_PER_SOL).toString()
      ),
      new CreateVestingTypeInstruction(
        initialUnlock,
        toTimestamp(startTime),
        toTimestamp(endTime),
        unlockPeriod,
        toTimestamp(cliff)
      )
    );

    await processTransaction(transaction);

    console.log("");
    console.log(`Created vesting type named '${vestingName}':`);
    console.log((await tokenVesting.getVestingTypeStatistic()).toString());
    console.log("");
  }
}

export async function fillVestingsFromCsv(fileName: PathLike) {
  const content = readFileSync(fileName, "utf-8");
  const parser = parse(content, { columns: true });

  let schema = yup.object().shape({
    vestingName: yup.string().required(),
    receiver: yup.string().required(),
    amountSol: yup.string().required(),
  });

  for await (const record of parser) {
    try {
      await schema.validate(record);
    } catch (e) {
      const validationMessage = (e as Error).message;
      throw Error(
        `Error while validating csv input: ${validationMessage}, for data: ${JSON.stringify(
          record
        )}`
      );
    }
    const { vestingName, receiver, amountSol } = record;
    const receiverPubkey = new web3.PublicKey(receiver);
    let { tokenVesting, processTransaction } = await helpers.bootstrapTools(
      vestingName
    );
    const transaction: web3.Transaction = await tokenVesting.createVestingAccount(
      receiverPubkey,
      new CreateVestingAccountInstruction(
        new BN(
          new BigNumber(amountSol).multipliedBy(web3.LAMPORTS_PER_SOL).toString()
        )
      )
    );

    await processTransaction(transaction);

    console.log("");
    console.log(`Created vesting of type '${vestingName}' for receiver ${receiverPubkey.toBase58()}`);
    console.log((await tokenVesting.getVestingStatistic(receiverPubkey)).toString());
    console.log("");
  }
}
