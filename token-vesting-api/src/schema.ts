import { field, generateSchemas, variant } from "@solvei/borsh/schema";
import BN from "bn.js";
import { FloatAsU64Creator } from "./utils";

class Super {}

@variant(0)
export class CreateVestingTypeInstruction extends Super {
  @field(FloatAsU64Creator)
  public initial_unlock: number;

  @field({ type: "u64" })
  public start_time: number;

  @field({ type: "u64" })
  public end_time: number;

  @field({ type: "u64" })
  public unlock_period: number;

  @field({ type: "u64" })
  public cliff: number;

  constructor(
    initial_unlock: number,
    start_time: number,
    end_time: number,
    unlock_period: number,
    cliff: number
  ) {
    super();
    this.initial_unlock = initial_unlock;
    this.start_time = start_time;
    this.end_time = end_time;
    this.unlock_period = unlock_period;
    this.cliff = cliff;
  }
}

@variant(1)
export class CreateVestingAccountInstruction extends Super {
  @field({ type: "u64" })
  public amount: BN;

  constructor(amount: BN) {
    super();
    this.amount = amount;
  }
}
@variant(2)
export class WithdrawFromVestingInstruction extends Super {
  @field({ type: "u64" })
  public amount: BN;

  constructor(amount: BN) {
    super();
    this.amount = amount;
  }
}

@variant(3)
export class WithdrawExcessiveFromPoolInstruction extends Super {
  @field({ type: "u64" })
  public amount: BN;

  constructor(amount: BN) {
    super();
    this.amount = amount;
  }
}

@variant(4)
export class ChangeVestingTypeScheduleInstruction extends Super {
  @field(FloatAsU64Creator)
  public initial_unlock: number;

  @field({ type: "u64" })
  public start_time: number;

  @field({ type: "u64" })
  public end_time: number;

  @field({ type: "u64" })
  public unlock_period: number;

  @field({ type: "u64" })
  public cliff: number;

  constructor(
    initial_unlock: number,
    start_time: number,
    end_time: number,
    unlock_period: number,
    cliff: number
  ) {
    super();
    this.initial_unlock = initial_unlock;
    this.start_time = start_time;
    this.end_time = end_time;
    this.unlock_period = unlock_period;
    this.cliff = cliff;
  }
}

export class Instruction {
  @field({ type: Super })
  public enum: Super;

  constructor(value: Super) {
    this.enum = value;
  }
}

export const schemas = generateSchemas([
  CreateVestingTypeInstruction,
  CreateVestingAccountInstruction,
  WithdrawFromVestingInstruction,
  WithdrawExcessiveFromPoolInstruction,
  ChangeVestingTypeScheduleInstruction,
  Instruction,
]);
