#!/usr/bin/env node
import {} from "dotenv";
import {readFileSync} from "fs";

import {
  changeVestingTypeScheduleCommand,
  withdrawExcessiveFromPoolCommand,
  createVestingTypeCommand,
  test,
  createVestingAccountCommand,
  withdrawFromVesting,
  getVestingStatisticCommand,
  getVestingTypeStatisticCommand,
  fillVestingTypesFromJson,
  fillVestingsFromCsv,
  parseVestingSchedule,
} from "./commands";

function main(args: string[]) {
  const config = require("dotenv").config();
  const program = require("commander");

  program
    .command("create")
    .alias("c")
    .description("Create vesting type")
    .requiredOption(
      "-f, --filePath <path>",
      "File with the description of vesting type to be created"
    )
    .action(async (options: any) => {
      const {
        filePath,
      } = options;
      const content = readFileSync(filePath, "utf-8");
      const scheduleObject = JSON.parse(content);
      const vestingSchedule = await parseVestingSchedule(scheduleObject);
      if (vestingSchedule === undefined) {
        console.log("Failed to parse vesting schedule");
        return;
      }

      console.log(
        await createVestingTypeCommand(
          vestingSchedule!
        )
      );
    });

  program
    .command("change-schedule")
    .alias("cs")
    .description("Change vesting type schedule")
    .requiredOption(
      "-f, --filePath <path>",
      "Part of tokens to unlock on start date"
    )
    .action(async (options: any) => {
      const { filePath } = options;

      const content = readFileSync(filePath, "utf-8");
      const scheduleObject = JSON.parse(content);
      const vestingSchedule = await parseVestingSchedule(scheduleObject);
      if (vestingSchedule === undefined) {
        console.log("Failed to parse vesting schedule");
        return;
      }

      console.log(
        await changeVestingTypeScheduleCommand(vestingSchedule)
      );
    });

  program
    .command("withdraw-excessive")
    .alias("we")
    .description("Withdraw excessive from pool")
    .requiredOption(
      "-a, --amountTokens <number>",
      "Amount of tokens to withdraw from pool"
    )
    .action(async (options: any) => {
      const amountTokens = options.amountTokens;
      console.log(await withdrawExcessiveFromPoolCommand(amountTokens));
    });

  program
    .command("create-vesting")
    .alias("cv")
    .description("Create vesting")
    .requiredOption(
      "-r, --receiver <wallet>",
      "Wallet address for vesting receiver"
    )
    .requiredOption("-a, --amountTokens <number>", "Amount of tokens for vesting")
    .action(async (options: any) => {
      const receiver = options.receiver;
      const amountTokens = options.amountTokens;
      console.log(await createVestingAccountCommand(receiver, amountTokens));
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
      "-a, --amountTokens <number>",
      "Amount of tokens to withdraw from pool"
    )
    .requiredOption(
      "-r, --receiver <wallet>",
      "Wallet address for vesting receiver"
    )
    .action(async (options: any) => {
      console.log(
        await withdrawFromVesting(options.amountTokens, options.receiver)
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
      await fillVestingTypesFromJson(options.filePath);
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

  program.parse(args);
}

main(process.argv);
