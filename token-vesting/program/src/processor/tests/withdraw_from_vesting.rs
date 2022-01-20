use chrono::Utc;

use solana_program::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
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
    instruction::{initialize_account, initialize_mint, mint_to},
};

use crate::instruction::VestingInstruction;
use crate::state::{LinearVesting, VestingAccount, VestingSchedule, VestingTypeAccount};

use super::common::{
    add_account, deserialize_account, deserialize_token_account, AbstractTestContext,
};

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

fn construct_default_vesting_schedule(tokens: u64) -> VestingSchedule {
    let dt = Utc::now();
    let timestamp = dt.timestamp() as u64;
    VestingSchedule::with_tokens(tokens)
        .legacy(
            timestamp - 200,
            timestamp + 10,
            100,
            timestamp - 110,
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

async fn call_withdraw_from_vesting(
    test_context: &mut TestContext,
    amount: u64,
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

    let data = VestingInstruction::WithdrawFromVesting { amount }.pack();
    let (pda, _bump_seed) =
        Pubkey::find_program_address(&[vesting_type.pubkey().as_ref()], program_id);

    let mut accounts = vec![
        AccountMeta::new(vesting_type.pubkey(), false),
        AccountMeta::new(vesting.pubkey(), false),
        AccountMeta::new(token_account.pubkey(), false),
        AccountMeta::new(token_pool.pubkey(), false),
        AccountMeta::new(pda, false),             // pda
        AccountMeta::new(spl_token::id(), false), // token program
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
async fn test_successful_withdraw_from_vesting() {
    let mut test_context = TestContext::new(default_add_accounts).await;
    init_token_accounts(&mut test_context, 500).await;

    let tokens = 100;

    call_create_vesting_type(
        &mut test_context,
        &construct_default_vesting_schedule(tokens),
        vec![],
    )
    .await
    .unwrap();
    call_create_vesting(&mut test_context, tokens, vec![])
        .await
        .unwrap();

    call_withdraw_from_vesting(&mut test_context, 40, vec![])
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
    assert_eq!(vesting_data.withdrawn_tokens, 40);

    let vesting_type_data =
        deserialize_account::<VestingTypeAccount>(&mut banks_client, vesting_type.pubkey()).await;
    assert_eq!(vesting_type_data.locked_tokens_amount, 60);

    let token_account_data =
        deserialize_token_account(&mut banks_client, token_account.pubkey()).await;
    assert_eq!(token_account_data.amount, 40);
}
