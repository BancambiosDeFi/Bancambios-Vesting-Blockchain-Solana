import {
  AccountLayout,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  Token,
  TOKEN_PROGRAM_ID,
  AccountInfo as TokenAccountInfo,
  u64,
} from "@solana/spl-token";
import {
  AccountInfo,
  Connection,
  PublicKey,
  Signer,
  SystemProgram,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
import { deserialize, serialize } from "@solvei/borsh";
import { generateSchemas } from "@solvei/borsh/schema";
import BN from "bn.js";
import { VestingAccount, VestingTypeAccount } from "./models";
import { VestingStatistic, VestingTypeStatistic } from "./query";
import {
  ChangeVestingTypeScheduleInstruction,
  CreateVestingAccountInstruction,
  CreateVestingTypeInstruction,
  Instruction,
  schemas,
  WithdrawExcessiveFromPoolInstruction,
  WithdrawFromVestingInstruction,
} from "./schema";
import {
  getTokenPoolSeed,
  getVestingTypeSeed,
  getVestingSeed,
  AccountValidation,
  validateAccountExist,
  validateAccountDoesNotExist,
} from "./utils";

export interface ITokenVesting {
  createVestingType(
    amountLamports: BN,
    createVestingTypeInstruction: CreateVestingTypeInstruction
  ): Promise<Transaction>;

  withdrawExcessiveFromPool(
    withdrawExcessiveFromPoolInstruction: WithdrawExcessiveFromPoolInstruction
  ): Promise<Transaction>;

  changeVestingTypeSchedule(
    changeVestingTypeScheduleInstruction: ChangeVestingTypeScheduleInstruction
  ): Promise<Transaction>;

  createVestingAccount(
    receiver: PublicKey,
    createVestingInstruction: CreateVestingAccountInstruction
  ): Promise<Transaction>;

  getVestingType(vestingName: string): Promise<VestingTypeAccount>;
  getVesting(
    tokenPool: PublicKey,
    receiver: PublicKey
  ): Promise<VestingAccount>;
}
export class TokenVesting implements ITokenVesting {
  constructor(
    private connection: Connection,
    private programId: PublicKey,
    private mint: PublicKey,
    private creator: PublicKey,
    private vestingName: string
  ) {}

  async createVestingType(
    amountLamorts: BN,
    createVestingTypeInstruction: CreateVestingTypeInstruction
  ): Promise<Transaction> {
    const { pubkey: tokenAccountPubkey } =
      await this.getAssociatedTokenAccountContext(
        this.creator,
        validateAccountExist
      );

    const { seed: tokenPoolSeed, pubkey: tokenPoolPubkey } =
      await this.getTokenPoolAccountContext(validateAccountDoesNotExist);

    const { seed: vestingTypeSeed, pubkey: vestingTypePubkey } =
      await this.getVestingTypeAccountContext(validateAccountDoesNotExist);

    const lamportsForTokenPool =
      await this.connection.getMinimumBalanceForRentExemption(
        AccountLayout.span
      );

    const createTokenPoolAccount = SystemProgram.createAccountWithSeed({
      fromPubkey: this.creator,
      basePubkey: this.creator,
      newAccountPubkey: tokenPoolPubkey,
      lamports: lamportsForTokenPool,
      space: AccountLayout.span,
      programId: TOKEN_PROGRAM_ID,
      seed: tokenPoolSeed,
    });

    const initTokenPool = Token.createInitAccountInstruction(
      TOKEN_PROGRAM_ID,
      this.mint,
      tokenPoolPubkey,
      this.creator
    );
    const transferToTokenPool = Token.createTransferInstruction(
      TOKEN_PROGRAM_ID,
      tokenAccountPubkey,
      tokenPoolPubkey,
      this.creator,
      [],
      new u64(amountLamorts.toString())
    );

    const lamportsForVestingType =
      await this.connection.getMinimumBalanceForRentExemption(
        VestingTypeAccount.space
      );

    const createVestingTypeAccount = SystemProgram.createAccountWithSeed({
      fromPubkey: this.creator,
      basePubkey: this.creator,
      newAccountPubkey: vestingTypePubkey,
      lamports: lamportsForVestingType,
      space: VestingTypeAccount.space,
      programId: this.programId,
      seed: vestingTypeSeed,
    });

    const instance = new Instruction(createVestingTypeInstruction);
    const instructionData = serialize(schemas, instance);

    const createVestingType = new TransactionInstruction({
      keys: [
        { pubkey: this.creator, isSigner: true, isWritable: true },
        { pubkey: vestingTypePubkey, isSigner: false, isWritable: true },
        {
          pubkey: tokenPoolPubkey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: TOKEN_PROGRAM_ID,
          isSigner: false,
          isWritable: false,
        },
      ],
      programId: this.programId,
      data: Buffer.from(instructionData),
    });

    const transaction = new Transaction();
    transaction.add(createTokenPoolAccount);
    transaction.add(initTokenPool);
    transaction.add(transferToTokenPool);
    transaction.add(createVestingTypeAccount);
    transaction.add(createVestingType);
    return transaction;
  }

  async withdrawExcessiveFromPool(
    withdrawExcessiveFromPoolInstruction: WithdrawExcessiveFromPoolInstruction
  ): Promise<Transaction> {
    const { pubkey: tokenAccountPubkey } =
      await this.getAssociatedTokenAccountContext(
        this.creator,
        validateAccountExist
      );

    const { pubkey: vestingTypePubkey } =
      await this.getVestingTypeAccountContext(validateAccountExist);
    const vestingType = await this.getVestingType();

    const [pda, bumpSeed] = await PublicKey.findProgramAddress(
      [vestingTypePubkey.toBuffer()],
      this.programId
    );

    const tokenPoolPubkey = vestingType.token_pool;
    if (tokenPoolPubkey === undefined)
      throw Error("Vesting type does not have token pool");

    const instance = new Instruction(withdrawExcessiveFromPoolInstruction);
    const instructionData = serialize(schemas, instance);

    const withdrawInstruction = new TransactionInstruction({
      keys: [
        { pubkey: this.creator, isSigner: true, isWritable: true },
        { pubkey: tokenAccountPubkey, isSigner: false, isWritable: true },
        { pubkey: pda, isSigner: false, isWritable: false },
        { pubkey: tokenPoolPubkey, isSigner: false, isWritable: true },
        { pubkey: vestingTypePubkey, isSigner: false, isWritable: false },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      ],
      programId: this.programId,
      data: Buffer.from(instructionData),
    });

    const transaction = new Transaction();
    transaction.add(withdrawInstruction);
    return transaction;
  }

  async changeVestingTypeSchedule(
    changeVestingTypeScheduleInstruction: ChangeVestingTypeScheduleInstruction
  ): Promise<Transaction> {
    const { pubkey: vestingTypePubkey } =
      await this.getVestingTypeAccountContext(validateAccountExist);

    const instance = new Instruction(changeVestingTypeScheduleInstruction);
    const instructionData = serialize(schemas, instance);

    const changeVestingTypeSchedule = new TransactionInstruction({
      keys: [
        { pubkey: this.creator, isSigner: true, isWritable: true },
        { pubkey: vestingTypePubkey, isSigner: false, isWritable: true },
      ],
      programId: this.programId,
      data: Buffer.from(instructionData),
    });

    const transaction = new Transaction();
    transaction.add(changeVestingTypeSchedule);
    return transaction;
  }

  async createVestingAccount(
    receiver: PublicKey,
    createVestingInstruction: CreateVestingAccountInstruction
  ): Promise<Transaction> {
    const { pubkey: tokenPoolPubkey } = await this.getTokenPoolAccountContext(
      validateAccountExist
    );

    const { pubkey: vestingTypePubkey } =
      await this.getVestingTypeAccountContext(validateAccountExist);

    const { seed: vestingSeed, pubkey: vestingPubkey } =
      await this.getVestingAccountContext(
        tokenPoolPubkey,
        receiver,
        validateAccountDoesNotExist
      );

    const { pubkey: tokenAccountPubkey, account: tokenAccount } =
      await this.getAssociatedTokenAccountContext(receiver, null);

    const createAssociatedTokenAccount =
      Token.createAssociatedTokenAccountInstruction(
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        this.mint,
        tokenAccountPubkey,
        receiver,
        this.creator
      );

    const lamportsForVesting =
      await this.connection.getMinimumBalanceForRentExemption(
        VestingAccount.space
      );
    const createVestingAccount = SystemProgram.createAccountWithSeed({
      fromPubkey: this.creator,
      basePubkey: this.creator,
      newAccountPubkey: vestingPubkey,
      lamports: lamportsForVesting,
      space: VestingAccount.space,
      programId: this.programId,
      seed: vestingSeed,
    });

    const instance = new Instruction(createVestingInstruction);
    const instructionData = serialize(schemas, instance);
    const createVesting = new TransactionInstruction({
      keys: [
        { pubkey: this.creator, isSigner: true, isWritable: true },
        { pubkey: vestingTypePubkey, isSigner: false, isWritable: true },
        { pubkey: vestingPubkey, isSigner: false, isWritable: true },
        { pubkey: tokenAccountPubkey, isSigner: false, isWritable: true },
        { pubkey: tokenPoolPubkey, isSigner: false, isWritable: true },
      ],
      programId: this.programId,
      data: Buffer.from(instructionData),
    });

    const transaction = new Transaction();
    if (tokenAccount === null) {
      transaction.add(createAssociatedTokenAccount);
    }
    transaction.add(createVestingAccount);
    transaction.add(createVesting);
    return transaction;
  }

  async withdrawFromVesting(
    receiverPubkey: PublicKey,
    withdrawFromVestingInstruction: WithdrawFromVestingInstruction
  ): Promise<Transaction> {
    const { pubkey: tokenAccountPubkey } =
      await this.getAssociatedTokenAccountContext(
        receiverPubkey,
        validateAccountExist
      );

    const { pubkey: vestingTypePubkey } =
      await this.getVestingTypeAccountContext(validateAccountExist);

    const vestingType = await this.getVestingType();
    if (!vestingType.token_pool) throw "";
    const { pubkey: vestingPubkey } = await this.getVestingAccountContext(
      vestingType.token_pool,
      receiverPubkey,
      validateAccountExist
    );

    const [pda, bumpSeed] = await PublicKey.findProgramAddress(
      [vestingTypePubkey.toBuffer()],
      this.programId
    );

    const tokenPoolPubkey = vestingType.token_pool;
    if (tokenPoolPubkey === undefined)
      throw Error("Vesting type does not have token pool");

    const instance = new Instruction(withdrawFromVestingInstruction);
    const instructionData = serialize(schemas, instance);

    const withdrawInstruction = new TransactionInstruction({
      keys: [
        { pubkey: vestingTypePubkey, isSigner: false, isWritable: true },
        { pubkey: vestingPubkey, isSigner: false, isWritable: true },
        { pubkey: tokenAccountPubkey, isSigner: false, isWritable: true },
        { pubkey: tokenPoolPubkey, isSigner: false, isWritable: true },
        { pubkey: pda, isSigner: false, isWritable: false },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
      ],
      programId: this.programId,
      data: Buffer.from(instructionData),
    });

    const transaction = new Transaction();
    transaction.add(withdrawInstruction);
    return transaction;
  }

  async getVestingType(): Promise<VestingTypeAccount> {
    const { account } = await this.getVestingTypeAccountContext(
      validateAccountExist
    );
    return deserialize(
      generateSchemas([VestingTypeAccount]),
      VestingTypeAccount,
      (account as AccountInfo<Buffer>).data
    );
  }

  async getVesting(
    tokenPool: PublicKey,
    receiver: PublicKey
  ): Promise<VestingAccount> {
    const { account } = await this.getVestingAccountContext(
      tokenPool,
      receiver,
      validateAccountExist
    );
    return deserialize(
      generateSchemas([VestingAccount]),
      VestingAccount,
      (account as AccountInfo<Buffer>).data
    );
  }

  async getAssociatedTokenAccount(
    receiver: PublicKey
  ): Promise<TokenAccountInfo> {
    const { pubkey } = await this.getAssociatedTokenAccountContext(
      receiver,
      validateAccountExist
    );
    const token = new Token(
      this.connection,
      this.mint,
      TOKEN_PROGRAM_ID,
      null as unknown as Signer // signer is required but isn't used
    );
    return await token.getAccountInfo(pubkey);
  }

  async getAssociatedTokenAccountContext(
    receiver: PublicKey,
    validation: AccountValidation | null
  ) {
    let pubkey = await Token.getAssociatedTokenAddress(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      this.mint,
      receiver,
      true
    );
    const account = await this.connection.getAccountInfo(pubkey);
    if (validation !== null) {
      validation(account, "Associated Token Account");
    }

    return { pubkey, account };
  }

  async getTokenPoolAccountContext(validation: AccountValidation | null) {
    const seed = getTokenPoolSeed(this.programId, this.vestingName);
    return this.getAccountContext(
      seed,
      TOKEN_PROGRAM_ID,
      validation,
      "Token Pool Account"
    );
  }

  async getVestingTypeAccountContext(validation: AccountValidation | null) {
    const seed = getVestingTypeSeed(this.programId, this.vestingName);
    return this.getAccountContext(
      seed,
      this.programId,
      validation,
      "Vesting Type Account"
    );
  }

  async getVestingAccountContext(
    tokenPoolPubkey: PublicKey,
    receiver: PublicKey,
    validation: AccountValidation | null
  ) {
    const seed = getVestingSeed(this.programId, tokenPoolPubkey, receiver);
    return this.getAccountContext(
      seed,
      this.programId,
      validation,
      "Vesting Account"
    );
  }

  async getAccountContext(
    seed: string,
    programId: PublicKey,
    validation: AccountValidation | null,
    name: string
  ) {
    const pubkey = await PublicKey.createWithSeed(
      this.creator,
      seed,
      programId
    );
    const account = await this.connection.getAccountInfo(pubkey);
    if (validation !== null) {
      validation(account, name);
    }

    return {
      seed,
      pubkey,
      account,
    };
  }

  async getVestingStatistic(receiver: PublicKey): Promise<VestingStatistic> {
    const now = Math.floor(Date.now() / 1000);
    const vestingType = await this.getVestingType();

    if (!vestingType.token_pool || !vestingType.vesting_schedule)
      throw Error("Deserialization error");

    const vesting = await this.getVesting(vestingType.token_pool, receiver);

    if (
      !vesting.total_tokens ||
      !vesting.withdrawn_tokens ||
      !vesting.token_account ||
      !vesting.vesting_type_account
    )
      throw Error("Deserialization error");

    const allTokens = vesting.total_tokens;
    const availableToWithdrawTokens =
      vesting.calculate_available_to_withdraw_amount(
        vestingType.vesting_schedule,
        now
      );
    const unlockedTokens = vesting.withdrawn_tokens.add(
      availableToWithdrawTokens
    );
    return new VestingStatistic(
      allTokens,
      unlockedTokens,
      availableToWithdrawTokens,
      vesting.withdrawn_tokens,
      vesting.token_account,
      vesting.vesting_type_account
    );
  }

  async getVestingTypeStatistic(): Promise<VestingTypeStatistic> {
    const vestingType = await this.getVestingType();
    const { pubkey: tokenPoolPubkey } = await this.getTokenPoolAccountContext(
      validateAccountExist
    );
    const tokensInTokenPool = await this.connection.getTokenAccountBalance(
      tokenPoolPubkey
    );

    if (
      !vestingType.token_pool ||
      !vestingType.vesting_schedule ||
      !vestingType.locked_tokens_amount ||
      !vestingType.administrator
    )
      throw Error("Deserialization error");

    return new VestingTypeStatistic(
      vestingType.vesting_schedule,
      vestingType.locked_tokens_amount,
      vestingType.administrator,
      vestingType.token_pool,
      new BN(tokensInTokenPool.value.amount)
    );
  }
}
