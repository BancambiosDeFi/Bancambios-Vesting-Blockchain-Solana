import { PublicKey } from "@solana/web3.js";
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
    const allTokens = new BigNumber(this.allTokens.toString());
    const unlockedTokens = new BigNumber(this.unlockedTokens.toString());
    const availableToWithdrawTokens = new BigNumber(
      this.availableToWithdrawTokens.toString()
    );
    const withdrawn_tokens = new BigNumber(
      this.withdrawn_tokens.toString()
    );

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
    let result = '';
    result += `Locked tokens: ${this.locked_tokens_amount}\n`;
    result += `Administrator: ${this.administrator.toString()}\n`;
    result += `Token pool: ${this.token_pool.toString()}\n`;
    result += `Tokens in pool: ${this.tokensInTokenPool.toString()}\n`;
    result += 'Vesting schedule:\n';
    result += `    Total tokens: ${this.vesting_schedule.token_count?.toString()}\n`;
    result += `    Total vestings: ${this.vesting_schedule.vesting_count}\n`;
    for(let i = 0; i < this.vesting_schedule.vesting_count!; i+=1) {
      result += `        Vesting ${i}:\n`
      result += `            Tokens: ${this.vesting_schedule.vestings![i][0]}\n`;
      result += `            Start time: ${this.vesting_schedule.vestings![i][1].start_time.toString()}\n`;
      result += `            Unlock period: ${this.vesting_schedule.vestings![i][1].unlock_period.toString()}\n`;
      result += `            Unlock count: ${this.vesting_schedule.vestings![i][1].unlock_count}\n`;
    }
    return result;
  }
}
