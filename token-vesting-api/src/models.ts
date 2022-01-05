import { LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import { field } from "@solvei/borsh/schema";
import BigNumber from "bignumber.js";
import BN from "bn.js";
import { FloatAsU64Creator } from "./utils";

const PublicKeyCreator = {
  serialize: (value: PublicKey, writer: any) => {
    writer.writeU256(new BN(value.encode(), 16, "be"));
  },
  deserialize: (reader: any): PublicKey => {
    let value = reader.readU256();
    let buffer = value.toArrayLike(Buffer, "be");
    return PublicKey.decode(buffer);
  },
};

export class VestingSchedule {
  @field(FloatAsU64Creator)
  public initial_unlock: number; //8
  @field({ type: "u64" })
  public start_time: BN; //8
  @field({ type: "u64" })
  public end_time: BN; //8
  @field({ type: "u64" })
  public unlock_period: BN; //8
  @field({ type: "u64" })
  public cliff: BN; //8

  static readonly space: number = 40;

  constructor(
    initial_unlock: number,
    start_time: BN,
    end_time: BN,
    unlock_period: BN,
    cliff: BN
  ) {
    this.initial_unlock = initial_unlock;
    this.start_time = start_time;
    this.end_time = end_time;
    this.unlock_period = unlock_period;
    this.cliff = cliff;
  }

  calculateUnlockedPart(now: BN): BigNumber {
    if (now.lt(this.start_time)) return new BigNumber(0.0);
    if (now.lt(this.cliff)) return new BigNumber(this.initial_unlock);
    if (now.gt(this.end_time)) return new BigNumber(1.0);

    let total_unlocks_count = new BigNumber(
      this.end_time.sub(this.cliff).div(this.unlock_period).toString()
    );
    total_unlocks_count = total_unlocks_count.plus(1); // for unlock immideately at the end of a cliff

    if (this.end_time.sub(this.cliff).mod(this.unlock_period).gtn(0)) {
      total_unlocks_count = total_unlocks_count.plus(1); // for a last non-full period
    }

    let part_per_unlock = new BigNumber(1.0 - this.initial_unlock).div(
      total_unlocks_count
    );

    let elapsed_unlocks = new BigNumber(
      now.sub(this.cliff).div(this.unlock_period).toString()
    );

    elapsed_unlocks = elapsed_unlocks.plus(1); // unlock immideately at the end of a cliff

    return elapsed_unlocks
      .multipliedBy(part_per_unlock)
      .plus(this.initial_unlock);
  }
} //40 bytes

export class VestingTypeAccount {
  @field({ type: "u8" })
  public is_initialized: boolean | undefined; //1
  @field({ type: VestingSchedule })
  public vesting_schedule: VestingSchedule | undefined; //40
  @field({ type: "u64" })
  public locked_tokens_amount: BN | undefined; //8
  @field(PublicKeyCreator)
  public administrator: PublicKey | undefined; //32
  @field(PublicKeyCreator)
  public token_pool: PublicKey | undefined; //32

  static readonly space: number = 113;

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
  @field({ type: "u64" })
  public total_tokens: BN | undefined; //8
  @field({ type: "u64" })
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
    if (!this.total_tokens || !this.withdrawn_tokens)
      throw Error("Deserialization error");

    let unlocked_part = schedule.calculateUnlockedPart(new BN(now));
    const totalTokens = new BigNumber(this.total_tokens.toString());
    let unlocked_amount = new BN(
      totalTokens.multipliedBy(unlocked_part).dp(0, BigNumber.ROUND_FLOOR).toString()
    );
    unlocked_amount = BN.min(unlocked_amount, this.total_tokens); // safeguard check
    return BN.max(unlocked_amount.sub(this.withdrawn_tokens), new BN(0));
  }
} //81 bytes
