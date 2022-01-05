import { LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import BN from "bn.js";
import { VestingSchedule } from "./models";
import BigNumber from "bignumber.js";

export class VestingStatistic {
  constructor(
    public allTokens: BN,
    public unlockedTokens: BN,
    public availableToWithdrawTokens: BN,
    public withdrawn_tokens: BN,
    public token_account: PublicKey,
    public vesting_type_account: PublicKey
  ) {}

  toString(): string {
    const allTokens = new BigNumber(this.allTokens.toString()).div(
      LAMPORTS_PER_SOL
    );
    const unlockedTokens = new BigNumber(this.unlockedTokens.toString()).div(
      LAMPORTS_PER_SOL
    );
    const availableToWithdrawTokens = new BigNumber(
      this.availableToWithdrawTokens.toString()
    ).div(LAMPORTS_PER_SOL);
    const withdrawn_tokens = new BigNumber(
      this.withdrawn_tokens.toString()
    ).div(LAMPORTS_PER_SOL);

    return (
      `All tokens: ${allTokens}\n` +
      `Unlocked tokens: ${unlockedTokens}\n` +
      `Available to withdraw: ${availableToWithdrawTokens}\n` +
      `Withdrawn tokens: ${withdrawn_tokens}\n` +
      `Receiver associated account: ${this.token_account.toString()}\n` +
      `Vesting type account: ${this.vesting_type_account.toString()}`
    );
  }
}

export class VestingTypeStatistic {
  constructor(
    public vesting_schedule: VestingSchedule,
    public locked_tokens_amount: BN,
    public administrator: PublicKey,
    public token_pool: PublicKey,
    public tokensInTokenPool: BN
  ) {}

  toString(): string {
    const locked_tokens_amount = new BigNumber(
      this.locked_tokens_amount.toString()
    ).div(LAMPORTS_PER_SOL);
    const tokensInTokenPool = new BigNumber(
      this.tokensInTokenPool.toString()
    ).div(LAMPORTS_PER_SOL);

    const unlockPeriod = this.vesting_schedule.unlock_period;
    const unlockPeriodTotalMinutes = unlockPeriod.divn(60);
    const unlockPeriodTotalHours = unlockPeriodTotalMinutes.divn(60);
    return (
      `Locked tokens: ${locked_tokens_amount}\n` +
      `Administrator: ${this.administrator.toString()}\n` +
      `Token pool: ${this.token_pool.toString()}\n` +
      `Tokens in token pool: ${tokensInTokenPool}\n` +
      `Start time: ${new Date(
        this.vesting_schedule.start_time.muln(1000).toNumber()
      )}\n` +
      `End time: ${new Date(
        this.vesting_schedule.end_time.muln(1000).toNumber()
      )}\n` +
      `Cliff: ${new Date(
        this.vesting_schedule.cliff.muln(1000).toNumber()
      )}\n` +
      `Unlock period: ${unlockPeriodTotalHours.divn(24)} days, ${unlockPeriodTotalHours.modn(24)} hours, ` +
      `${unlockPeriodTotalMinutes.modn(60)} minutes and ${unlockPeriod.modn(60)} seconds\n` +
      `Initial unlock: ${this.vesting_schedule.initial_unlock.toString()}`
    );
  }
}
