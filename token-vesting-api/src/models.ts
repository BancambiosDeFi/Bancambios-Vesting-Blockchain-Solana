import {  PublicKey } from "@solana/web3.js";
import { field } from "@solvei/borsh/schema";
import { BinaryReader, BinaryWriter } from "@solvei/borsh/binary";
import BigNumber from "bignumber.js";
import BN from "bn.js";

const PublicKeyCreator = {
  serialize: (value: PublicKey, writer: BinaryWriter) => {
    writer.writeU256(new BN(value.encode(), 32, "be"));
  },
  deserialize: (reader: BinaryReader): PublicKey => {
    let value = reader.readU256();
    let buffer = value.toArrayLike(Buffer, "be", 32);
    return PublicKey.decode(buffer);
  },
};

function numberToBytes(number: number): Uint8Array {
  if (!Number.isInteger(number)) throw "Non integers are not supported";
  const byteArray = new Uint8Array(8);

  for (let index = 0; index < byteArray.length; index++) {
    const byte = number & 0xff;
    byteArray[index] = byte;
    number = (number - byte) / 256;
  }

  return byteArray;
}

export const TokenCountCreator = {
  serialize: (value: BN, writer: BinaryWriter) => {
    if (value.toString() != parseInt(value.toString()).toString())
      throw "Could not serialize BN";
    writer.writeFixedArray(numberToBytes(parseInt(value.toString())));
  },
  deserialize: (reader: BinaryReader): BN => {
    let n = 0;
    let mutltiplier = 1;
    for (let i = 0; i < 8; i+=1) {
      n += reader.readU8() * mutltiplier;
      mutltiplier *= 256;
    }
    return new BN(n.toString());
  }
}

export const VestingsCreator = {
  serialize: (value: Array<[BN, LinearVesting]>, writer: any) => {
    if (value.length > MAX_VESTINGS)
        throw new Error("Too many vestings in schedule");
    for (let i = 0; i < value.length; i+=1) {
        if (value[i][0].toString() != parseInt(value[i][0].toString()).toString())
          throw "Could not serialize BN";
        writer.writeBuffer(numberToBytes(parseInt(value[i][0].toString())));

        writer.writeU64(value[i][1].start_time);
        writer.writeU64(value[i][1].unlock_period);
        writer.writeU8(value[i][1].unlock_count);
    }
    for (let i = 0; i < MAX_VESTINGS - value.length; i+=1) {
        writer.writeU64(0);
        writer.writeU64(0);
        writer.writeU64(0);
        writer.writeU8(0);
    }
  },
  deserialize: (reader: any): Array<[BN, LinearVesting]> => {
    let result = Array<[BN, LinearVesting]>(MAX_VESTINGS);
    for (let i = 0; i < MAX_VESTINGS; i+=1) {
      let tokens = reader.readU64();
      let start_time = reader.readU64();
      let unlock_period = reader.readU64();
      let unlock_count = reader.readU8();
      result[i] = [tokens, new LinearVesting(start_time, unlock_period, unlock_count)];
    }
    return result;
  }
}

export class LinearVesting {
  @field({ type: "u64" })
  public start_time: BN
  @field({ type: "u64" })
  public unlock_period: BN
  @field({ type: "u8" })
  public unlock_count: number
  static space: number = 17;

  constructor(start_time: BN, unlock_period: BN, unlock_count: number) {
      this.start_time = start_time;
      this.unlock_period = unlock_period;
      this.unlock_count = unlock_count;
  }

  public static without_start(unlock_period: BN, unlock_count: number): LinearVesting {
      return new LinearVesting(new BN(0), unlock_period, unlock_count);
  }

  public static cliff(start_time: BN): LinearVesting {
      return new LinearVesting(start_time, new BN(0), 1);
  }

  public last(): BN {
      return this.start_time.add(this.unlock_period.mul(new BN(this.unlock_count-1)));
  }

  public part(): number {
      return 1 / this.unlock_count;
  }

  public available(time: BN): number {
    if (time.lt(this.start_time)) {
        return 0.0;
    }
    if (time.gte(this.last())) {
        return 1.0;
    }
    time = time.sub(this.start_time);
    let unlocks = time.divRound(this.unlock_period).toNumber() + 1;
    return Math.min(1.0, this.part() * unlocks);
  }
} // 17

export const MAX_VESTINGS = 16;

export class VestingSchedule {
  @field(TokenCountCreator)
  public token_count: BN | undefined; // 8
  @field({ type: "u8" })
  public vesting_count: number | undefined; // 1
  @field(VestingsCreator)
  public vestings: Array<[BN, LinearVesting]> | undefined; // 25 * 16 = 400

  static readonly space: number = 409;

  constructor(token_count: BN, vestings: Array<[BN, LinearVesting]>) {
    this.token_count = token_count;
    this.vesting_count = vestings === undefined ? undefined : vestings.length;
    this.vestings = vestings;
  }

  public calculateUnlockedPart(now: BN): BigNumber {
    let unlocked = new BN(0);
    for (let i = 0; i < this.vesting_count!; i+=1) {
      if (this.vestings![i][1].start_time.gt(now)) {
        break;
      }
      let unlocked_part = this.vestings![i][1].available(now);
      unlocked = unlocked.add(this.vestings![i][0].muln(unlocked_part));
    }
    return new BigNumber(unlocked.toString());
  }

  public start_time(): BN {
    return this.vestings![0][1].start_time;
  }

  public last(): BN {
    return this.vestings![this.vesting_count!-1][1].last();
  }
} // 407 bytes

export class ScheduleBuilder {
    token_count: BN;
    used_tokens: BN;
    vestings: Array<[BN, LinearVesting]>;

    constructor(token_count: BN) {
      this.token_count = token_count;
      this.used_tokens = new BN(0);
      this.vestings = new Array<[BN, LinearVesting]>();
    }

    use_tokens(tokens: BN) {
      this.used_tokens = this.used_tokens.add(tokens);
    }

    available_tokens(): BN {
      if (this.used_tokens.gte(this.token_count)) {
          return new BN(0);
      } else {
          return this.token_count.sub(this.used_tokens);
      }
    }

    remove_last(): [BN, LinearVesting] | undefined {
        let last = this.vestings.pop();
        if (last !== undefined) 
          this.used_tokens = this.used_tokens.sub(last![0]);
        return last;
    }
    public add(vesting: LinearVesting, tokens?: BN): ScheduleBuilder {
      let tokens_: BN = tokens === undefined ? this.available_tokens() : tokens!;
      this.use_tokens(tokens_);

      this.vestings.push([tokens_, vesting]);
      return this;
    }

    public cliff(time: BN, tokens?: BN): ScheduleBuilder {
      return this.add(LinearVesting.cliff(time), tokens);
    }

    public offseted_by(
        offset: BN,
        vesting: LinearVesting,
        tokens?: BN
    ): ScheduleBuilder | undefined {
        if (this.vestings.length === 0) {
          return undefined;
        }
        let last_vesting = this.vestings[this.vestings.length-1][1];
        return this.add(
          new LinearVesting(
            last_vesting.last().add(offset),
            vesting.unlock_period,
            vesting.unlock_count),
          tokens);
    }

    public offseted(
        vesting: LinearVesting,
        tokens?: BN,
    ): ScheduleBuilder | undefined {
        return this.offseted_by(vesting.unlock_period, vesting, tokens)
    }

    public ending_at(end_time: BN): ScheduleBuilder | undefined {
        if (this.vestings.length == 0) {
          return undefined;
            // return Err(ScheduleBuilderError::EmptyBuilder);
        }
        let last_vesting = this.vestings[this.vestings.length - 1];
        if (end_time >= last_vesting[1].last()) {
          return this;
        } else {
            let last_vesting = this.remove_last()!;
            let new_unlock_count = (end_time.sub(last_vesting[1].start_time))
                                   .divRound(last_vesting[1].unlock_period)
                                   .addn(1);

            let linear_tokens = last_vesting[0].mul(new_unlock_count)
                                               .divRound(new BN(last_vesting[1].unlock_count));
            let cliff_tokens = last_vesting[0].sub(linear_tokens);

            this.add(
              new LinearVesting(
                last_vesting[1].start_time,
                last_vesting[1].unlock_period,
                new_unlock_count.toNumber()),
              linear_tokens)
            .cliff(end_time, cliff_tokens)
        }
    }

    public legacy(
        start_time: BN,
        end_time: BN,
        unlock_period: BN,
        cliff: BN,
        initial_unlock_tokens: BN,
        tokens?: BN,
    ): ScheduleBuilder | undefined {
        if (start_time >= end_time) {
          return undefined;
            // return Err(ScheduleBuilderError::InvalidTimeInterval);
        }
        let tokens_ = tokens === undefined ? this.available_tokens() : tokens!;
        if (initial_unlock_tokens.gte( tokens_)) {
          return undefined;
            // return Err(ScheduleBuilderError::InitialUnlockTooBig);
        }

        let builder: ScheduleBuilder = this;
        if (initial_unlock_tokens.gtn(0)) {
          builder = builder.cliff(start_time, initial_unlock_tokens);
        }


        let remaining_tokens = tokens_.sub(initial_unlock_tokens);

        let total_linear_unlocks = end_time.sub(start_time)
                                           .divRound(unlock_period)
                                           .addn(1);
        if (!end_time.sub(start_time).mod(unlock_period).eqn(0) ){
            total_linear_unlocks = total_linear_unlocks.addn(1);
        }

        let unlocks_before_cliff = cliff.sub(start_time)
                                        .divRound(unlock_period)
                                        .addn(1);
        if (unlocks_before_cliff.gtn(0)) {
            let tokens_at_cliff = remaining_tokens.mul(unlocks_before_cliff)
                                                  .divRound(total_linear_unlocks);
            remaining_tokens = remaining_tokens.sub(tokens_at_cliff);
            builder = builder.cliff(cliff, tokens_at_cliff)
        }

        let first_linear_unlock = cliff.add(cliff.mod(unlock_period));
        builder
            .add(
                new LinearVesting(
                    first_linear_unlock,
                    unlock_period,
                    total_linear_unlocks.sub(unlocks_before_cliff).toNumber(),
                ),
                remaining_tokens,
            )
            .ending_at(end_time)
    }

    public build(): VestingSchedule | undefined {
        if (!this.token_count.eq(this.used_tokens)) {
          console.log(`Warning: unused tokens. Total: ${this.token_count} Used: ${this.used_tokens}`)
            // return Err(ScheduleBuilderError::InvalidTokenAmountUsed((
            //     this.token_count,
            //     this.used_tokens,
            // )));
        }

        if (this.vestings.length > MAX_VESTINGS) {
          console.log("Error: too many vestings")
          return undefined;
            // return Err(ScheduleBuilderError::TooManyVestings);
        }

        for(let i = 1; i < this.vestings.length; i+=1) {
            if (this.vestings[i - 1][1].last() > this.vestings[i][1].start_time) {
                console.log("Error: vestings are not sorted")
                return undefined
            }
        }

        for(let i = 0; i < this.vestings.length; i+=1) {
            if (this.vestings[i][0].eqn(0)) {
              console.log("Error: vesting with 0 tokens")
              return undefined;
                // return Err(ScheduleBuilderError::ZeroTokens);
            }
        }

        return new VestingSchedule(this.token_count, this.vestings);
    }
}

export class VestingTypeAccount {
  @field({ type: "u8" })
  public is_initialized: boolean | undefined; //1
  @field({ type: VestingSchedule })
  public vesting_schedule: VestingSchedule | undefined; //409
  @field(TokenCountCreator)
  public locked_tokens_amount: BN | undefined; //8
  @field(PublicKeyCreator)
  public administrator: PublicKey | undefined; //32
  @field(PublicKeyCreator)
  public token_pool: PublicKey | undefined; //32

  static readonly space: number = 482;

  constructor(properties?: {
    is_initialized: boolean;
    vesting_schedule: VestingSchedule;
    locked_tokens_amount: BN;
    administrator: PublicKey;
    token_pool: PublicKey;
  }) {
    if (properties) {
      this.is_initialized = properties.is_initialized;
      this.vesting_schedule = properties.vesting_schedule;
      this.locked_tokens_amount = properties.locked_tokens_amount;
      this.administrator = properties.administrator;
      this.token_pool = properties.token_pool;
    }
  }
} //113 bytes

export class VestingAccount {
  @field({ type: "u8" })
  public is_initialized: boolean | undefined; //1
  @field(TokenCountCreator)
  public total_tokens: BN | undefined; //8
  @field(TokenCountCreator)
  public withdrawn_tokens: BN | undefined; //8
  @field(PublicKeyCreator)
  public token_account: PublicKey | undefined; //32
  @field(PublicKeyCreator)
  public vesting_type_account: PublicKey | undefined; //32

  static readonly space: number = 81;

  constructor(properties?: {
    is_initialized: boolean;
    total_tokens: BN;
    withdrawn_tokens: BN;
    token_account: PublicKey;
    vesting_type_account: PublicKey;
  }) {
    if (properties) {
      this.is_initialized = properties.is_initialized;
      this.total_tokens = properties.total_tokens;
      this.withdrawn_tokens = properties.withdrawn_tokens;
      this.token_account = properties.token_account;
      this.vesting_type_account = properties.vesting_type_account;
    }
  }

  calculate_available_to_withdraw_amount(
    schedule: VestingSchedule,
    now: number
  ): BN {
    if (this.total_tokens === undefined || this.withdrawn_tokens === undefined)
      throw Error("Deserialization error: VestingAccount.calculate_available_to_withdraw_amount");

    let unlocked_part = schedule.calculateUnlockedPart(new BN(now));
    const totalTokens = new BigNumber(this.total_tokens.toString());

    let unlocked_amount = new BN(
        unlocked_part
          .multipliedBy(totalTokens)
          .dividedBy(new BigNumber(schedule.token_count!.toString()))
          .dp(0, BigNumber.ROUND_FLOOR)
          .toString());
    unlocked_amount = BN.min(unlocked_amount, this.total_tokens); // safeguard check
    return BN.max(unlocked_amount.sub(this.withdrawn_tokens), new BN(0));
  }
} //81 bytes
