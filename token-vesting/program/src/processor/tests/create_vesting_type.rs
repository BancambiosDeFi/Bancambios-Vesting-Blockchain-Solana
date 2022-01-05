use chrono::Utc;

use borsh::BorshSerialize;
use solana_program::{
    hash::Hash,
    instruction::{AccountMeta, Instruction, InstructionError},
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
};
use solana_program_test::ProgramTest;
use solana_sdk::{
    account::Account, signature::Keypair, signature::Signer, system_instruction,
    transaction::Transaction, transport::TransportError,
};
use spl_token::{
    self,
    instruction::{initialize_account, initialize_mint},
    state::Account as TokenAccount,
};

use crate::state::VestingTypeAccount;
use crate::{instruction::VestingInstruction, state::VestingSchedule};

use super::common::{
    add_account, deserialize_account, deserialize_token_account, AbstractTestContext, ErrorChecker,
};

struct KeyPairs {
    mint: Keypair,
    vesting_type: Keypair,
    token_pool: Keypair,
}

impl Default for KeyPairs {
    fn default() -> Self {
        Self {
            mint: Keypair::new(),
            vesting_type: Keypair::new(),
            token_pool: Keypair::new(),
        }
    }
}

type TestContext = AbstractTestContext<KeyPairs>;

fn default_add_accounts(program_test: &mut ProgramTest, program_id: Pubkey, keypairs: &KeyPairs) {
    add_account::<VestingTypeAccount>(program_test, program_id, &keypairs.vesting_type, true);
}

fn mint_init_transaction(
    payer: &Keypair,
    mint: &Keypair,
    mint_authority: &Keypair,
    recent_blockhash: Hash,
) -> Transaction {
    let instructions = [
        system_instruction::create_account(
            &payer.pubkey(),
            &mint.pubkey(),
            Rent::default().minimum_balance(82),
            82,
            &spl_token::id(),
        ),
        initialize_mint(
            &spl_token::id(),
            &mint.pubkey(),
            &mint_authority.pubkey(),
            None,
            0,
        )
        .unwrap(),
    ];
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));
    transaction.partial_sign(&[payer, mint], recent_blockhash);
    transaction
}

fn create_token_account(
    payer: &Keypair,
    mint: &Keypair,
    recent_blockhash: Hash,
    token_account: &Keypair,
    token_account_owner: &Pubkey,
) -> Transaction {
    let instructions = [
        system_instruction::create_account(
            &payer.pubkey(),
            &token_account.pubkey(),
            Rent::default().minimum_balance(165),
            165,
            &spl_token::id(),
        ),
        initialize_account(
            &spl_token::id(),
            &token_account.pubkey(),
            &mint.pubkey(),
            token_account_owner,
        )
        .unwrap(),
    ];
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));
    transaction.partial_sign(&[payer, token_account], recent_blockhash);
    transaction
}

async fn init_token_accounts(test_context: &mut TestContext) {
    let TestContext {
        banks_client,
        recent_blockhash,
        payer,
        keypairs: KeyPairs {
            token_pool, mint, ..
        },
        ..
    } = test_context;

    banks_client
        .process_transaction(mint_init_transaction(
            &payer,
            &mint,
            &payer,
            recent_blockhash.clone(),
        ))
        .await
        .unwrap();

    banks_client
        .process_transaction(create_token_account(
            &payer,
            &mint,
            recent_blockhash.clone(),
            &token_pool,
            &payer.pubkey(),
        ))
        .await
        .unwrap();
}

fn construct_default_vesting_schedule() -> VestingSchedule {
    let dt = Utc::now();
    let timestamp = dt.timestamp() as u64;
    VestingSchedule {
        start_time: timestamp + 100,
        end_time: timestamp + 200,
        unlock_period: 10,
        cliff: timestamp + 120,
        initial_unlock: 0,
    }
}

async fn call_create_vesting_type(
    test_context: &mut TestContext,
    vesting_schedule: &VestingSchedule,
    account_overrides: Vec<(usize, AccountMeta)>,
) -> Result<(), TransportError> {
    let TestContext {
        program_id,
        banks_client,
        recent_blockhash,
        payer,
        keypairs:
            KeyPairs {
                vesting_type,
                token_pool,
                ..
            },
    } = test_context;

    let data = VestingInstruction::CreateVestingType {
        start_time: vesting_schedule.start_time,
        end_time: vesting_schedule.end_time,
        unlock_period: vesting_schedule.unlock_period,
        cliff: vesting_schedule.cliff,
        initial_unlock: vesting_schedule.initial_unlock,
    }
    .pack();
    let mut accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(vesting_type.pubkey(), false),
        AccountMeta::new(token_pool.pubkey(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    for (index, account_info) in account_overrides.into_iter() {
        accounts[index] = account_info;
    }
    let instruction = Instruction {
        program_id: *program_id,
        accounts,
        data,
    };

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.partial_sign(&[payer as &Keypair], recent_blockhash.clone());
    banks_client.process_transaction(transaction).await
}

#[tokio::test]
async fn test_successful_create_vesting_type() {
    let mut test_context = TestContext::new(default_add_accounts).await;
    init_token_accounts(&mut test_context).await;

    let vesting_schedule = construct_default_vesting_schedule();
    call_create_vesting_type(&mut test_context, &vesting_schedule, vec![])
        .await
        .unwrap();

    let TestContext {
        program_id,
        mut banks_client,
        payer,
        keypairs:
            KeyPairs {
                vesting_type,
                token_pool,
                ..
            },
        ..
    } = test_context;

    let vesting_type_data =
        deserialize_account::<VestingTypeAccount>(&mut banks_client, vesting_type.pubkey()).await;
    assert_eq!(
        vesting_type_data,
        VestingTypeAccount {
            is_initialized: true,
            vesting_schedule,
            locked_tokens_amount: 0,
            administrator: payer.pubkey(),
            token_pool: token_pool.pubkey(),
        }
    );

    let token_pool_data = deserialize_token_account(&mut banks_client, token_pool.pubkey()).await;
    let (pda, _bump_seed) =
        Pubkey::find_program_address(&[(&vesting_type.pubkey()).as_ref()], &program_id);
    assert_eq!(token_pool_data.owner, pda);
}

#[tokio::test]
async fn test_create_vesting_type_without_sign() {
    let mut test_context = TestContext::new(default_add_accounts).await;
    init_token_accounts(&mut test_context).await;

    let vesting_schedule = construct_default_vesting_schedule();
    let new_caller = Keypair::new();
    let overrides = vec![(0, AccountMeta::new(new_caller.pubkey(), false))];
    let result = call_create_vesting_type(&mut test_context, &vesting_schedule, overrides).await;
    ErrorChecker::from(result).check(InstructionError::MissingRequiredSignature);
}

#[tokio::test]
async fn test_create_vesting_type_with_already_initialized_account() {
    let mut test_context = TestContext::new(default_add_accounts).await;
    init_token_accounts(&mut test_context).await;
    let vesting_schedule = construct_default_vesting_schedule();
    call_create_vesting_type(&mut test_context, &vesting_schedule, vec![])
        .await
        .unwrap();

    let new_pool = Keypair::new();
    create_token_account(
        &test_context.payer,
        &test_context.keypairs.mint,
        test_context.recent_blockhash,
        &new_pool,
        &test_context.payer.pubkey(),
    );

    let overrides = vec![(2, AccountMeta::new(new_pool.pubkey(), false))];
    let result = call_create_vesting_type(&mut test_context, &vesting_schedule, overrides).await;

    ErrorChecker::from(result).check(InstructionError::Custom(0));
}

#[tokio::test]
async fn test_create_vesting_type_without_rent_exempt() {
    fn add_non_rent_exempt(
        program_test: &mut ProgramTest,
        program_id: Pubkey,
        keypairs: &KeyPairs,
    ) {
        let data = VestingTypeAccount::default().try_to_vec().unwrap();
        let rent = Rent::default();
        program_test.add_account(
            keypairs.vesting_type.pubkey(),
            Account {
                lamports: rent.minimum_balance(data.len()) - 1,
                owner: program_id,
                data,
                ..Account::default()
            },
        );
    }
    let mut test_context = TestContext::new(add_non_rent_exempt).await;
    init_token_accounts(&mut test_context).await;

    let vesting_schedule = construct_default_vesting_schedule();
    let result = call_create_vesting_type(&mut test_context, &vesting_schedule, vec![]).await;

    ErrorChecker::from(result).check(InstructionError::Custom(1));
}

#[tokio::test]
async fn test_create_vesting_type_with_invalid_schedule() {
    let mut test_context = TestContext::new(default_add_accounts).await;
    init_token_accounts(&mut test_context).await;

    let mut vesting_schedule = construct_default_vesting_schedule();
    vesting_schedule.end_time = vesting_schedule.start_time - 1;
    let result = call_create_vesting_type(&mut test_context, &vesting_schedule, vec![]).await;

    ErrorChecker::from(result).check(InstructionError::Custom(2));
}

#[tokio::test]
async fn test_create_vesting_type_with_invalid_token_program() {
    let mut test_context = TestContext::new(default_add_accounts).await;
    init_token_accounts(&mut test_context).await;

    let vesting_schedule = construct_default_vesting_schedule();
    let overrides = vec![(3, AccountMeta::new_readonly(test_context.program_id, false))];
    let result = call_create_vesting_type(&mut test_context, &vesting_schedule, overrides).await;
    ErrorChecker::from(result).check(InstructionError::IncorrectProgramId);
}

#[tokio::test]
async fn test_create_vesting_type_with_invalid_token_pool_account_owner() {
    let fake_pool = Keypair::new();
    let add_with_fake_pool =
        |program_test: &mut ProgramTest, program_id: Pubkey, keypairs: &KeyPairs| {
            default_add_accounts(program_test, program_id, keypairs);
            add_account::<VestingTypeAccount>(program_test, program_id, &fake_pool, true);
        };
    let mut test_context = TestContext::new(add_with_fake_pool).await;
    init_token_accounts(&mut test_context).await;

    let vesting_schedule = construct_default_vesting_schedule();
    let overrides = vec![(2, AccountMeta::new(fake_pool.pubkey(), false))];
    let result = call_create_vesting_type(&mut test_context, &vesting_schedule, overrides).await;
    ErrorChecker::from(result).check(InstructionError::IncorrectProgramId);
}

#[tokio::test]
async fn test_create_vesting_type_with_uninitialized_token_pool() {
    let uninitialized_pool = Keypair::new();
    let add_with_uninitialized_pool =
        |program_test: &mut ProgramTest, program_id: Pubkey, keypairs: &KeyPairs| {
            default_add_accounts(program_test, program_id, keypairs);
            program_test.add_account(
                uninitialized_pool.pubkey(),
                Account {
                    lamports: 1000000000,
                    owner: spl_token::id(),
                    data: vec![0; TokenAccount::LEN],
                    ..Account::default()
                },
            );
        };
    let mut test_context = TestContext::new(add_with_uninitialized_pool).await;
    init_token_accounts(&mut test_context).await;

    let vesting_schedule = construct_default_vesting_schedule();
    let overrides = vec![(2, AccountMeta::new(uninitialized_pool.pubkey(), false))];
    let result = call_create_vesting_type(&mut test_context, &vesting_schedule, overrides).await;
    ErrorChecker::from(result).check(InstructionError::UninitializedAccount);
}
