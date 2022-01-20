use chrono::Utc;

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
    instruction::{initialize_account, initialize_mint, mint_to},
    state::{Account as TokenAccount, AccountState},
};

use crate::instruction::VestingInstruction;
use crate::state::{LinearVesting, VestingAccount, VestingSchedule, VestingTypeAccount};

use super::common::{add_account, deserialize_account, AbstractTestContext, ErrorChecker};

struct KeyPairs {
    mint: Keypair,
    vesting_type: Keypair,
    vesting: Keypair,
    token_account: Keypair,
    token_pool: Keypair,
}

impl Default for KeyPairs {
    fn default() -> Self {
        Self {
            mint: Keypair::new(),
            vesting_type: Keypair::new(),
            vesting: Keypair::new(),
            token_account: Keypair::new(),
            token_pool: Keypair::new(),
        }
    }
}

type TestContext = AbstractTestContext<KeyPairs>;

fn default_add_accounts(program_test: &mut ProgramTest, program_id: Pubkey, keypairs: &KeyPairs) {
    add_account::<VestingTypeAccount>(program_test, program_id, &keypairs.vesting_type, true);
    add_account::<VestingAccount>(program_test, program_id, &keypairs.vesting, true);
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

pub fn mint_to_token_account(
    payer: &Keypair,
    mint: &Keypair,
    to: &Pubkey,
    recent_blockhash: Hash,
    amount: u64,
) -> Transaction {
    let instructions = [mint_to(
        &spl_token::id(),
        &mint.pubkey(),
        to,
        &payer.pubkey(),
        &[&payer.pubkey()],
        amount,
    )
    .unwrap()];
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));
    transaction.partial_sign(&[payer], recent_blockhash);
    transaction
}

async fn init_token_accounts(test_context: &mut TestContext, tokens_in_pool: u64) {
    let TestContext {
        banks_client,
        recent_blockhash,
        payer,
        keypairs:
            KeyPairs {
                token_account,
                token_pool,
                mint,
                ..
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

    banks_client
        .process_transaction(mint_to_token_account(
            &payer,
            &mint,
            &token_pool.pubkey(),
            recent_blockhash.clone(),
            tokens_in_pool,
        ))
        .await
        .unwrap();

    banks_client
        .process_transaction(create_token_account(
            &payer,
            &mint,
            recent_blockhash.clone(),
            &token_account,
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

    let mut vestings: [(u64, LinearVesting); VestingSchedule::MAX_VESTINGS] = Default::default();
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

async fn call_create_vesting(
    test_context: &mut TestContext,
    total_tokens: u64,
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
                vesting,
                token_account,
                token_pool,
                ..
            },
    } = test_context;

    let data = VestingInstruction::CreateVestingAccount { total_tokens }.pack();
    let mut accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(vesting_type.pubkey(), false),
        AccountMeta::new(vesting.pubkey(), false),
        AccountMeta::new_readonly(token_account.pubkey(), false),
        AccountMeta::new(token_pool.pubkey(), false),
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
async fn test_successful_create_vesting_account() {
    let mut test_context = TestContext::new(default_add_accounts).await;
    init_token_accounts(&mut test_context, 500).await;

    call_create_vesting_type(
        &mut test_context,
        &construct_default_vesting_schedule(),
        vec![],
    )
    .await
    .unwrap();
    call_create_vesting(&mut test_context, 100, vec![])
        .await
        .unwrap();

    let TestContext {
        mut banks_client,
        keypairs:
            KeyPairs {
                vesting,
                vesting_type,
                token_account,
                ..
            },
        ..
    } = test_context;

    let vesting_data =
        deserialize_account::<VestingAccount>(&mut banks_client, vesting.pubkey()).await;
    assert_eq!(
        vesting_data,
        VestingAccount {
            is_initialized: true,
            total_tokens: 100,
            withdrawn_tokens: 0,
            token_account: token_account.pubkey(),
            vesting_type_account: vesting_type.pubkey(),
        }
    );

    let vesting_type_data =
        deserialize_account::<VestingTypeAccount>(&mut banks_client, vesting_type.pubkey()).await;
    assert_eq!(vesting_type_data.locked_tokens_amount, 100);
}

#[tokio::test]
async fn test_create_vesting_account_without_signed() {
    let mut test_context = TestContext::new(default_add_accounts).await;
    init_token_accounts(&mut test_context, 500).await;

    call_create_vesting_type(
        &mut test_context,
        &construct_default_vesting_schedule(),
        vec![],
    )
    .await
    .unwrap();
    let new_caller = Keypair::new();
    let overrides = vec![(0, AccountMeta::new(new_caller.pubkey(), false))];
    let result = call_create_vesting(&mut test_context, 100, overrides).await;
    ErrorChecker::from(result).check(InstructionError::MissingRequiredSignature);
}

#[tokio::test]
async fn test_create_vesting_with_already_initialized_account() {
    let mut test_context = TestContext::new(default_add_accounts).await;
    init_token_accounts(&mut test_context, 500).await;

    call_create_vesting_type(
        &mut test_context,
        &construct_default_vesting_schedule(),
        vec![],
    )
    .await
    .unwrap();
    call_create_vesting(&mut test_context, 100, vec![])
        .await
        .unwrap();

    let new_receiver = Keypair::new();
    test_context
        .banks_client
        .process_transaction(create_token_account(
            &test_context.payer,
            &test_context.keypairs.mint,
            test_context.recent_blockhash.clone(),
            &new_receiver,
            &test_context.payer.pubkey(),
        ))
        .await
        .unwrap();
    let overrides = vec![(3, AccountMeta::new_readonly(new_receiver.pubkey(), false))];
    let result = call_create_vesting(&mut test_context, 100, overrides).await;
    ErrorChecker::from(result).check(InstructionError::Custom(0));
}

#[tokio::test]
async fn test_create_vesting_with_non_rent_exempt_account() {
    fn add_accounts(program_test: &mut ProgramTest, program_id: Pubkey, keypairs: &KeyPairs) {
        add_account::<VestingTypeAccount>(program_test, program_id, &keypairs.vesting_type, true);
        add_account::<VestingAccount>(program_test, program_id, &keypairs.vesting, false);
    }

    let mut test_context = TestContext::new(add_accounts).await;
    init_token_accounts(&mut test_context, 500).await;

    call_create_vesting_type(
        &mut test_context,
        &construct_default_vesting_schedule(),
        vec![],
    )
    .await
    .unwrap();

    let result = call_create_vesting(&mut test_context, 100, vec![]).await;
    ErrorChecker::from(result).check(InstructionError::Custom(1));
}

#[tokio::test]
async fn test_create_vesting_with_non_initialized_vesting_type() {
    let mut test_context = TestContext::new(default_add_accounts).await;
    init_token_accounts(&mut test_context, 500).await;

    let result = call_create_vesting(&mut test_context, 100, vec![]).await;
    ErrorChecker::from(result).check(InstructionError::UninitializedAccount);
}

// TODO need two signers to check
// #[tokio::test]
// async fn test_create_vesting_if_not_administrator() {
// }

#[tokio::test]
async fn test_create_vesting_account_with_invalid_pool_account() {
    let another_vesting = Keypair::new();
    let another_pool = Keypair::new();

    let add_accounts = |program_test: &mut ProgramTest, program_id: Pubkey, keypairs: &KeyPairs| {
        default_add_accounts(program_test, program_id, keypairs);
        add_account::<VestingTypeAccount>(program_test, program_id, &another_vesting, true);
    };
    let mut test_context = TestContext::new(add_accounts).await;
    init_token_accounts(&mut test_context, 500).await;
    test_context
        .banks_client
        .process_transaction(create_token_account(
            &test_context.payer,
            &test_context.keypairs.mint,
            test_context.recent_blockhash.clone(),
            &another_pool,
            &test_context.payer.pubkey(),
        ))
        .await
        .unwrap();

    let overrides = vec![
        (1, AccountMeta::new(another_vesting.pubkey(), false)),
        (2, AccountMeta::new(another_pool.pubkey(), false)),
    ];
    call_create_vesting_type(
        &mut test_context,
        &construct_default_vesting_schedule(),
        overrides,
    )
    .await
    .unwrap();
    call_create_vesting_type(
        &mut test_context,
        &construct_default_vesting_schedule(),
        vec![],
    )
    .await
    .unwrap();

    let overrides = vec![(4, AccountMeta::new(another_pool.pubkey(), false))];
    let result = call_create_vesting(&mut test_context, 100, overrides).await;
    ErrorChecker::from(result).check(InstructionError::IncorrectProgramId);
}

#[tokio::test]
async fn test_create_vesting_account_with_non_spl_token() {
    let fake_token_account = Keypair::new();

    let add_accounts = |program_test: &mut ProgramTest, program_id: Pubkey, keypairs: &KeyPairs| {
        default_add_accounts(program_test, program_id, keypairs);
        let account = TokenAccount {
            state: AccountState::Initialized,
            ..Default::default()
        };
        let mut data = vec![0; TokenAccount::LEN];
        account.pack_into_slice(&mut data[..]);
        program_test.add_account(
            fake_token_account.pubkey(),
            Account {
                lamports: 10000000000,
                owner: program_id,
                data,
                ..Account::default()
            },
        );
    };
    let mut test_context = TestContext::new(add_accounts).await;
    init_token_accounts(&mut test_context, 500).await;

    call_create_vesting_type(
        &mut test_context,
        &construct_default_vesting_schedule(),
        vec![],
    )
    .await
    .unwrap();

    let overrides = vec![(
        4,
        AccountMeta::new_readonly(fake_token_account.pubkey(), false),
    )];
    let result = call_create_vesting(&mut test_context, 100, overrides).await;
    ErrorChecker::from(result).check(InstructionError::IncorrectProgramId);
}
