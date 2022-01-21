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
import { LinearVesting, VestingSchedule } from "token-vesting-api/dist/models";

export async function createVestingTypeCommand(
  vestingSchedule: VestingSchedule,
): Promise<string> {
  let { tokenVesting, processTransaction } = await helpers.bootstrapTools();

  const transaction = await tokenVesting.createVestingType(
    new CreateVestingTypeInstruction(
      vestingSchedule.token_count!.toNumber(),
      vestingSchedule.vesting_count!,
      vestingSchedule.vestings!
    )
  );

  await processTransaction(transaction);
  return await getVestingTypeStatisticCommand();
}

export async function changeVestingTypeScheduleCommand(
  vestingSchedule: VestingSchedule,
): Promise<string> {
  let { tokenVesting, processTransaction } = await helpers.bootstrapTools();
  const transaction: web3.Transaction =
    await tokenVesting.changeVestingTypeSchedule(
      new ChangeVestingTypeScheduleInstruction(
      vestingSchedule.token_count!.toNumber(),
      vestingSchedule.vesting_count!,
      vestingSchedule.vestings!
      )
    );
  await processTransaction(transaction);
  return await getVestingTypeStatisticCommand();
}

export async function withdrawExcessiveFromPoolCommand(
  amountTokens: string
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
          new BigNumber(amountTokens)
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
  amountTokens: string
): Promise<string> {
  let { tokenVesting, processTransaction } = await helpers.bootstrapTools();
  const receiverPubkey = new web3.PublicKey(receiver);
  const transaction: web3.Transaction = await tokenVesting.createVestingAccount(
    receiverPubkey,
    new CreateVestingAccountInstruction(
      new BN(
        new BigNumber(amountTokens).multipliedBy(web3.LAMPORTS_PER_SOL).toString()
      )
    )
  );

  await processTransaction(transaction);

  return await getVestingStatisticCommand(receiver);
}

export async function withdrawFromVesting(
  amountTokens: string,
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
        new BigNumber(amountTokens).multipliedBy(web3.LAMPORTS_PER_SOL).toString()
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

export async function parseVestingSchedule(schedule: object): Promise<VestingSchedule | undefined> {
  // PnYnMnDTnHnMnS
  // NOTE: fractional values of `n` are not supported
  const IS08601_DURATION_REGEX = /P(?=\d+[YMWD])(\d+Y)?(\d+M)?(\d+W)?(\d+D)?(T(?=\d+[HMS])(\d+H)?(\d+M)?(\d+S)?)?/;

  let schema = yup.object().shape({
      name: yup.string().required(),
      amount: yup.number().required().positive().integer(),
      schedule: yup.array().min(1).of(
        yup.object({
          type: yup.string().oneOf(["offseted", "fixed", "onetime"]).required(),
          part: yup.number().nullable().defined().positive().min(1).integer(),
          count: yup.number().positive().integer().min(1).default(1),
          time: yup.date().optional(),
          offset: yup.string().matches(IS08601_DURATION_REGEX),
          period: yup.string().nullable().matches(IS08601_DURATION_REGEX),
        })).required(),
    });

  const validated_schedule = await schema.validate(schedule);

  let total_tokens = new BN(validated_schedule.amount);
  let builder = VestingSchedule.with_tokens(total_tokens);

  const parse_duration = (duration: string): BN => {
    let result = new BN(0);
    let matches = duration.match(IS08601_DURATION_REGEX); 
    if (matches !== null) {
      let hadTimeSpecifier = false;
      for (const match of matches) {
        if (match === undefined) continue;
        if (match.startsWith('P')) continue;
        if (match.startsWith('T')) {
          hadTimeSpecifier = true;
          continue;
        }

        let n = parseInt(match.substring(0, match.length - 1));
        switch (match[match.length - 1]) {
          case 'Y':
            result = result.addn(n*60*60*24*30*365);
            break;
          case 'M':
            if (hadTimeSpecifier) {
              // minutes
              result = result.addn(n*60);
            } else {
              // months
              result = result.addn(n*60*60*24*30);
            }
            break;
          case 'W':
            result = result.addn(n*60*60*24*7);
            break;
          case 'D':
            result = result.addn(n*60*60*24);
            break;
          case 'H':
            result = result.addn(n*60*60);
            break;
          case 'S':
            result = result.addn(n);
            break;
        }
      }
    }
    return result;
  };

  // Schedule type semantics:
  // - fixed:
  //   used for defining general linear vestings
  //   uses absolute time
  //   required fields: type, part, time, count, period
  // - onetime:
  //   used for defining cliffs and other 1-time unlocks
  //   uses absolute time
  //   required fields: type, part, time
  // - offseted:
  //   used for defining unlocks which will be offseted relatively to the previous one
  //   uses relative time
  //   required fields: type, part, offset, count, period
  for (const schedule_item of validated_schedule.schedule) {
    let tokens: BN | undefined = schedule_item.part === null ? undefined : new BN(schedule_item.part!);

    switch (schedule_item.type) {
      case 'fixed': {
        let {time, count, period} = schedule_item;
        builder.add(
          new LinearVesting(
            new BN(Math.floor(time!.getTime()/1000)),
            parse_duration(period!),
            count),
          tokens)
      }
      break;
      case 'onetime': {
        let {time} = schedule_item;
        builder.cliff(new BN(Math.floor(time!.getTime()/1000)), tokens);
      }
      break;
      case 'offseted': {
        let {offset, count, period} = schedule_item;
        period = period ?? "P"; // empty duration
        builder.offseted_by(
          parse_duration(offset!),
          LinearVesting.without_start(parse_duration(period), count),
          tokens);
      }
      break;
    }
  }

  return builder.build();
}

export async function fillVestingTypesFromJson(fileName: PathLike) {
  // TODO: change this shit
  const content = readFileSync(fileName, "utf-8");
  const types = JSON.parse(content);

  let schema = yup.array().of(
    yup.object().required()
    .shape({
      name: yup.string().required(),
      amount: yup.number().required().positive().integer(),
      schedule: yup.array().min(1)
    })
  );

  const validated_types = await schema.validate(types);

  let vestingTypes: Array<[string, VestingSchedule]> = new Array();

  for (const vesting_type of validated_types!) {
    let schedule = await parseVestingSchedule(vesting_type)
    if (schedule === undefined) 
      throw Error('Failed to build schedule');


    vestingTypes.push([vesting_type.name, schedule]);
  }

  // throw Error("42");

  for (const [vestingName, vestingType] of vestingTypes) {
    let { tokenVesting, processTransaction } = await helpers.bootstrapTools(
      vestingName
    );
    const transaction = await tokenVesting.createVestingType(
      new CreateVestingTypeInstruction(
        vestingType.token_count!.toNumber(),
        vestingType.vesting_count!,
        vestingType.vestings!,
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
    tokens: yup.number().integer().positive().required(),
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
