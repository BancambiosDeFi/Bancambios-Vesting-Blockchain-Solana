use chrono::Utc;

// use borsh::BorshSerialize;
use solana_program::{
    hash::Hash,
    instruction::{AccountMeta, Instruction, InstructionError},
    pubkey::Pubkey,
    rent::Rent,
};
use solana_program_test::ProgramTest;
use solana_sdk::{
    signature::Keypair, signature::Signer, system_instruction, transaction::Transaction,
    transport::TransportError,
};
use spl_token::{
    self,
    instruction::{initialize_account, initialize_mint},
};

use crate::state::{LinearVesting, ScheduleBuilderError, VestingTypeAccount, MAX_VESTINGS};
use crate::{instruction::VestingInstruction, state::VestingSchedule};

use super::common::{add_account, deserialize_account, AbstractTestContext, ErrorChecker};

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
    VestingSchedule::with_tokens(1000)
        .legacy(
            timestamp + 100,
            timestamp + 200,
            10,
            timestamp + 120,
            0,
            None,
        )
        .unwrap()
        .build()
        .unwrap()
}

fn construct_new_vesting_schedule(
    start_time: u64,
    end_time: u64,
    unlock_period: u64,
    cliff: u64,
    initial_unlock: u64,
    tokens: u64,
) -> Result<VestingSchedule, ScheduleBuilderError> {
    let dt = Utc::now();
    let timestamp = dt.timestamp() as u64;
    VestingSchedule::with_tokens(tokens)
        .legacy(
            timestamp + start_time,
            timestamp + end_time,
            unlock_period,
            timestamp + cliff,
            initial_unlock,
            None,
        )
        .and_then(|x| x.build())
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

    let mut vestings: [(u64, LinearVesting); MAX_VESTINGS] = Default::default();
    vestings[..vesting_schedule.vestings().len()].copy_from_slice(vesting_schedule.vestings());

    let data = VestingInstruction::CreateVestingType {
        token_count: vesting_schedule.token_count(),
        vesting_count: vesting_schedule.vestings().len() as u8,
        vestings,
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

async fn call_change_vesting_type_schedule(
    test_context: &mut TestContext,
    new_vesting_schedule: &VestingSchedule,
    account_overrides: Vec<(usize, AccountMeta)>,
) -> Result<(), TransportError> {
    let TestContext {
        program_id,
        banks_client,
        recent_blockhash,
        payer,
        keypairs: KeyPairs { vesting_type, .. },
    } = test_context;

    let mut vestings: [(u64, LinearVesting); MAX_VESTINGS] = Default::default();
    vestings[..new_vesting_schedule.vestings().len()]
        .copy_from_slice(new_vesting_schedule.vestings());

    let data = VestingInstruction::ChangeVestingTypeSchedule {
        token_count: new_vesting_schedule.token_count(),
        vesting_count: new_vesting_schedule.vestings().len() as u8,
        vestings,
    }
    .pack();
    let mut accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(vesting_type.pubkey(), false),
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
async fn test_successful_change_vesting_type_schedule() {
    let mut test_context = TestContext::new(default_add_accounts).await;
    init_token_accounts(&mut test_context).await;

    let vesting_schedule = construct_default_vesting_schedule();
    call_create_vesting_type(&mut test_context, &vesting_schedule, vec![])
        .await
        .unwrap();

    let new_vesting_schedule =
        construct_new_vesting_schedule(200, 400, 20, 240, 200, 1000).unwrap();
    call_change_vesting_type_schedule(&mut test_context, &new_vesting_schedule, vec![])
        .await
        .unwrap();

    let TestContext {
        mut banks_client,
        keypairs: KeyPairs { vesting_type, .. },
        ..
    } = test_context;

    let vesting_type_data =
        deserialize_account::<VestingTypeAccount>(&mut banks_client, vesting_type.pubkey()).await;
    assert_eq!(vesting_type_data.vesting_schedule, new_vesting_schedule);
}

#[tokio::test]
async fn test_change_vesting_type_schedule_with_invalid_schedule() {
    let mut test_context = TestContext::new(default_add_accounts).await;
    init_token_accounts(&mut test_context).await;

    let vesting_schedule = construct_default_vesting_schedule();
    call_create_vesting_type(&mut test_context, &vesting_schedule, vec![])
        .await
        .unwrap();

    let new_vesting_schedule = construct_new_vesting_schedule(200, 199, 20, 240, 200, 1000);
    assert_eq!(
        new_vesting_schedule,
        Err(ScheduleBuilderError::InvalidTimeInterval)
    );
}

#[tokio::test]
async fn test_change_vesting_type_schedule_with_uninitialized_account() {
    let mut test_context = TestContext::new(default_add_accounts).await;
    init_token_accounts(&mut test_context).await;

    let new_vesting_schedule =
        construct_new_vesting_schedule(200, 400, 20, 240, 200, 1000).unwrap();
    let result =
        call_change_vesting_type_schedule(&mut test_context, &new_vesting_schedule, vec![]).await;

    ErrorChecker::from(result).check(InstructionError::Custom(5));
}

#[ignore = "Requires multiple signers!"]
#[tokio::test]
async fn test_change_vesting_type_schedule_without_administrator() {}
