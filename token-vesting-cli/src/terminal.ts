#! /usr/bin/env node
import {} from "dotenv";
import {
  changeVestingTypeScheduleCommand,
  withdrawExcessiveFromPoolCommand,
  createVestingTypeCommand,
  test,
  createVestingAccountCommand,
  withdrawFromVesting,
  getVestingStatisticCommand,
  getVestingTypeStatisticCommand,
  fillVestingTypesFromCsv,
  fillVestingsFromCsv,
} from "./commands";
console.log("heyhey")

function main(args: string[]) {
  console.log("heyhey")
  const config = require("dotenv").config();
  const program = require("commander");

  program
    .command("create")
    .alias("c")
    .description("Create vesting type")
    .requiredOption(
      "-a, --amountSol <number>",
      "Amount of tokens to pass to a pool"
    )
    .requiredOption(
      "-i, --initialUnlock <number>",
      "Part of tokens to unlock on start date"
    )
    .requiredOption("-s, --startTime <number>", "Timestamp of vesting start")
    .requiredOption("-e, --endTime <number>", "Timestamp of vesting end")
    .requiredOption(
      "-u, --unlockPeriod <number>",
      "Amount of seconds between unlocks"
    )
    .requiredOption("-c, --cliff <number>", "Timestamp of cliff end")
    .action(async (options: any) => {
      const {
        amountSol,
        initialUnlock,
        startTime,
        endTime,
        unlockPeriod,
        cliff,
      } = options;
      console.log(
        await createVestingTypeCommand(
          amountSol,
          initialUnlock,
          startTime,
          endTime,
          unlockPeriod,
          cliff
        )
      );
    });

  program
    .command("change-schedule")
    .alias("cs")
    .description("Change vesting type schedule")
    .requiredOption(
      "-i, --initialUnlock <number>",
      "Part of tokens to unlock on start date"
    )
    .requiredOption("-s, --startTime <number>", "Timestamp of vesting start")
    .requiredOption("-e, --endTime <number>", "Timestamp of vesting end")
    .requiredOption(
      "-u, --unlockPeriod <number>",
      "Amount of seconds between unlocks"
    )
    .requiredOption("-c, --cliff <number>", "Timestamp of cliff end")
    .action(async (options: any) => {
      const { initialUnlock, startTime, endTime, unlockPeriod, cliff } =
        options;
      console.log(
        await changeVestingTypeScheduleCommand(
          initialUnlock,
          startTime,
          endTime,
          unlockPeriod,
          cliff
        )
      );
    });

  program
    .command("withdraw-excessive")
    .alias("we")
    .description("Withdraw excessive from pool")
    .requiredOption(
      "-a, --amountSol <number>",
      "Amount of tokens to withdraw from pool"
    )
    .action(async (options: any) => {
      const amountSol = options.amountSol;
      console.log(await withdrawExcessiveFromPoolCommand(amountSol));
    });

  program
    .command("create-vesting")
    .alias("cv")
    .description("Create vesting")
    .requiredOption(
      "-r, --receiver <wallet>",
      "Wallet address for vesting receiver"
    )
    .requiredOption("-a, --amountSol <number>", "Amount of tokens for vesting")
    .action(async (options: any) => {
      const receiver = options.receiver;
      const amountSol = options.amountSol;
      console.log(await createVestingAccountCommand(receiver, amountSol));
    });

  program
    .command("get-vesting-statistic")
    .alias("gvs")
    .description("Get vesting statistic for receiver")
    .requiredOption(
      "-r, --receiver <wallet>",
      "Wallet address for vesting receiver"
    )
    .action(async (options: any) => {
      console.log(await getVestingStatisticCommand(options.receiver));
    });

  program
    .command("get-vesting-type-statistic")
    .alias("gvts")
    .description("Get vesting type statistic")
    .action(async () => {
      console.log(await getVestingTypeStatisticCommand());
    });

  program
    .command("withdraw-vesting")
    .alias("wv")
    .description("Withdraw from vesting")
    .requiredOption(
      "-a, --amountSol <number>",
      "Amount of tokens to withdraw from pool"
    )
    .requiredOption(
      "-r, --receiver <wallet>",
      "Wallet address for vesting receiver"
    )
    .action(async (options: any) => {
      console.log(
        await withdrawFromVesting(options.amountSol, options.receiver)
      );
    });

  program
    .command("test")
    .alias("t")
    .description("Create test")
    .action(() => {
      test();
    });

  program
    .command("fill-vesting-types")
    .alias("fvt")
    .description("Fill vesting types from csv file")
    .requiredOption(
      "-f, --filePath <path>",
      "Path to a csv file with vesting types data"
    )
    .action(async (options: any) => {
      await fillVestingTypesFromCsv(options.filePath);
    });


  program
    .command("fill-vestings")
    .alias("fv")
    .description("Fill vestings from csv file")
    .requiredOption(
      "-f, --filePath <path>",
      "Path to a csv file with vestings data"
    )
    .action(async (options: any) => {
      await fillVestingsFromCsv(options.filePath);
    });

  program.parse(process.argv);
}

main(process.argv);
