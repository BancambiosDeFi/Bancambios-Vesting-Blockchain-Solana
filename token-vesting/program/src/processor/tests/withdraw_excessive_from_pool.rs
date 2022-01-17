use crate::instruction::VestingInstruction;
use crate::state::{LinearVesting, VestingTypeAccount, MAX_VESTINGS};
use crate::state::{VestingAccount, VestingSchedule};

use chrono::Utc;
use solana_program::instruction::InstructionError;
use solana_program::program_pack::Pack;
use solana_program::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    rent::Rent,
};
use solana_program_test::ProgramTest;
use solana_sdk::transport::TransportError;
use solana_sdk::{
    signature::Keypair, signature::Signer, system_instruction, transaction::Transaction,
};
use spl_associated_token_account::{create_associated_token_account, get_associated_token_address};
use spl_token::state::Account as TokenAccount;
use spl_token::{
    self,
    instruction::{initialize_account, initialize_mint, mint_to},
};

use super::common::{add_account, AbstractTestContext, ErrorChecker};

type TestContext = AbstractTestContext<KeyPairs>;

fn default_add_accounts(program_test: &mut ProgramTest, program_id: Pubkey, keypairs: &KeyPairs) {
    add_account::<VestingTypeAccount>(program_test, program_id, &keypairs.vesting_type, true);
    add_account::<VestingAccount>(program_test, program_id, &keypairs.vesting, true);
}

struct KeyPairs {
    mint_authority: Keypair,
    mint: Keypair,
    vesting_type: Keypair,
    vesting: Keypair,
    token_pool: Keypair,
    receiver: Keypair,
    no_admin: Keypair,
}

impl Default for KeyPairs {
    fn default() -> Self {
        Self {
            mint_authority: Keypair::new(),
            mint: Keypair::new(),
            vesting_type: Keypair::new(),
            vesting: Keypair::new(),
            token_pool: Keypair::new(),
            receiver: Keypair::new(),
            no_admin: Keypair::new(),
        }
    }
}

#[tokio::test]
async fn test() {
    withdraw_excessive_from_pool(100, 10, 0, true)
        .await
        .unwrap();
    withdraw_excessive_from_pool(10000, 5, 10, true)
        .await
        .unwrap();
    withdraw_excessive_from_pool(800, 700, 50, true)
        .await
        .unwrap();
    withdraw_excessive_from_pool(1000, 1000, 0, true)
        .await
        .unwrap();
    withdraw_excessive_from_pool(19, 10, 9, true).await.unwrap();
    withdraw_excessive_from_pool(300, 10, 0, true)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_transfer_more_than_exist() {
    let result = withdraw_excessive_from_pool(10, 100, 0, true).await;
    ErrorChecker::from(result).check(InstructionError::Custom(6));
}

#[tokio::test]
async fn test_transfer_without_administrator() {
    let result = withdraw_excessive_from_pool(100, 10, 0, false).await;
    ErrorChecker::from(result).check(InstructionError::Custom(4));
}

#[tokio::test]
async fn test_transfer_more_than_unlocked() {
    let result = withdraw_excessive_from_pool(100, 11, 90, true).await;
    ErrorChecker::from(result).check(InstructionError::Custom(6));
}

async fn withdraw_excessive_from_pool(
    token_pool_amount: u64,
    amount_to_transfer: u64,
    locked_tokens_amount: u64,
    administrator: bool,
) -> Result<(), TransportError> {
    let mut test_context = TestContext::new(default_add_accounts).await;

    //create mint and token pool
    init_token_accounts(&mut test_context, token_pool_amount).await;

    //check token pool amount
    let token_pool = test_context.keypairs.token_pool.pubkey().clone();
    let token_pool_data = get_associated_token_data(&mut test_context, token_pool, true).await;
    assert_eq!(token_pool_data.amount, token_pool_amount);

    //create vesting type
    call_create_vesting_type(
        &mut test_context,
        &construct_default_vesting_schedule(),
        None,
    )
    .await
    .unwrap();

    //create associated token account for admin
    let owner = test_context.payer.pubkey().clone();
    call_create_associated_account(&mut test_context, owner)
        .await
        .unwrap();

    //create associated token account for receiver
    let owner = test_context.keypairs.receiver.pubkey().clone();
    call_create_associated_account(&mut test_context, owner)
        .await
        .unwrap();

    //create vesting
    call_create_vesting_account(&mut test_context, locked_tokens_amount)
        .await
        .unwrap();

    //create associated token account for no admin
    let owner = test_context.keypairs.no_admin.pubkey().clone();
    call_create_associated_account(&mut test_context, owner)
        .await
        .unwrap();

    //transfer
    call_transfer(&mut test_context, amount_to_transfer, administrator).await?;

    //check administrator amount
    let admin = test_context.payer.pubkey().clone();
    let associated_account_data = get_associated_token_data(&mut test_context, admin, false).await;
    assert_eq!(associated_account_data.amount, amount_to_transfer);

    //check token pool amount
    let token_pool = test_context.keypairs.token_pool.pubkey().clone();
    let token_pool_data = get_associated_token_data(&mut test_context, token_pool, true).await;
    assert_eq!(
        token_pool_data.amount,
        token_pool_amount - amount_to_transfer
    );

    //check no admin amount
    let no_admin = test_context.keypairs.no_admin.pubkey().clone();
    let no_admin_data = get_associated_token_data(&mut test_context, no_admin, false).await;
    assert_eq!(no_admin_data.amount, 0);
    Ok(())
}

async fn init_token_accounts(test_context: &mut TestContext, amount: u64) {
    let TestContext {
        banks_client,
        recent_blockhash,
        payer,
        keypairs:
            KeyPairs {
                token_pool,
                mint,
                mint_authority,
                ..
            },
        ..
    } = test_context;

    banks_client
        .process_transaction(mint_init_transaction(
            &payer,
            &mint,
            &mint_authority,
            recent_blockhash.clone(),
        ))
        .await
        .unwrap();

    banks_client
        .process_transaction(create_token_account_transaction(
            &payer,
            &mint,
            &mint_authority,
            recent_blockhash.clone(),
            &token_pool,
            &payer.pubkey(),
            amount,
        ))
        .await
        .unwrap();
}

async fn get_associated_token_data(
    test_context: &mut TestContext,
    pubkey: Pubkey,
    associated: bool,
) -> TokenAccount {
    let TestContext {
        keypairs: KeyPairs { mint, .. },
        ..
    } = test_context;
    let account_data = TokenAccount::unpack(
        &mut test_context
            .banks_client
            .get_account(if associated {
                pubkey
            } else {
                get_associated_token_address(&pubkey, &mint.pubkey())
            })
            .await
            .unwrap()
            .unwrap()
            .data[..],
    )
    .unwrap();
    account_data
}

pub fn create_vesting_instruction(
    vesting_program_id: &Pubkey,
    signer: &Pubkey,
    vesting_type: &Pubkey,
    vesting: &Pubkey,
    receiver: &Pubkey,
    token_pool: &Pubkey,
    total_tokens: u64,
) -> Instruction {
    let data = VestingInstruction::CreateVestingAccount { total_tokens }.pack();
    let accounts = vec![
        AccountMeta::new(*signer, true),
        AccountMeta::new(*vesting_type, false),
        AccountMeta::new(*vesting, false),
        AccountMeta::new_readonly(*receiver, false),
        AccountMeta::new_readonly(*token_pool, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    Instruction {
        program_id: *vesting_program_id,
        accounts,
        data,
    }
}

pub fn withdraw_excessive_from_pool_instruction(
    vesting_program_id: &Pubkey,
    signer: &Pubkey,
    associated_account: &Pubkey,
    pda: &Pubkey,
    token_pool: &Pubkey,
    vesting_type: &Pubkey,
    amount: u64,
) -> Instruction {
    let data = VestingInstruction::WithdrawExcessiveFromPool { amount }.pack();
    let accounts = vec![
        AccountMeta::new(*signer, true),
        AccountMeta::new(*associated_account, false),
        AccountMeta::new_readonly(*pda, false),
        AccountMeta::new(*token_pool, false),
        AccountMeta::new_readonly(*vesting_type, false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ];
    Instruction {
        program_id: *vesting_program_id,
        accounts,
        data,
    }
}

//transactions
fn withdraw_excessive_from_pool_transaction(
    program_id: Pubkey,
    payer: &Keypair,
    signer: &Keypair,
    associated_account: &Pubkey,
    pda: &Pubkey,
    vesting_type: &Keypair,
    token_pool: &Keypair,
    recent_blockhash: Hash,
    amount: u64,
) -> Transaction {
    let init_instruction = [withdraw_excessive_from_pool_instruction(
        &program_id,
        &signer.pubkey(),
        &associated_account,
        &pda,
        &token_pool.pubkey(),
        &vesting_type.pubkey(),
        amount,
    )];
    let mut transaction = Transaction::new_with_payer(&init_instruction, Some(&payer.pubkey()));
    transaction.partial_sign(&[payer, signer], recent_blockhash);
    transaction
}

fn create_vesting_transaction(
    program_id: Pubkey,
    payer: &Keypair,
    vesting_type: &Keypair,
    vesting: &Keypair,
    receiver: &Pubkey,
    token_pool: &Keypair,
    recent_blockhash: Hash,
    total_tokens: u64,
) -> Transaction {
    let init_instruction = create_vesting_instruction(
        &program_id,
        &payer.pubkey(),
        &vesting_type.pubkey(),
        &vesting.pubkey(),
        &receiver,
        &token_pool.pubkey(),
        total_tokens,
    );
    let mut transaction = Transaction::new_with_payer(&[init_instruction], Some(&payer.pubkey()));
    transaction.partial_sign(&[payer], recent_blockhash);
    transaction
}

fn create_associated_token_account_transaction(
    payer: &Keypair,
    owner: &Pubkey,
    mint: &Keypair,
    recent_blockhash: Hash,
) -> Transaction {
    let init_instruction = create_associated_token_account(&payer.pubkey(), &owner, &mint.pubkey());
    let mut transaction = Transaction::new_with_payer(&[init_instruction], Some(&payer.pubkey()));
    transaction.partial_sign(&[payer], recent_blockhash);
    transaction
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

fn create_token_account_transaction(
    payer: &Keypair,
    mint: &Keypair,
    mint_authority: &Keypair,
    recent_blockhash: Hash,
    token_account: &Keypair,
    token_account_owner: &Pubkey,
    amount: u64,
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
        mint_to(
            &spl_token::id(),
            &mint.pubkey(),
            &token_account.pubkey(),
            &mint_authority.pubkey(),
            &[&mint_authority.pubkey()],
            amount,
        )
        .unwrap(),
    ];
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&payer.pubkey()));
    transaction.partial_sign(&[payer, token_account, mint_authority], recent_blockhash);
    transaction
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
            None
        )
        .unwrap()
        .build()
        .unwrap()
}

//
async fn call_create_vesting_type(
    test_context: &mut TestContext,
    vesting_schedule: &VestingSchedule,
    accounts: Option<Vec<AccountMeta>>,
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
    let accounts = accounts.unwrap_or(vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(vesting_type.pubkey(), false),
        AccountMeta::new(token_pool.pubkey(), false),
        AccountMeta::new_readonly(spl_token::id(), false),
    ]);
    let instruction = Instruction {
        program_id: *program_id,
        accounts,
        data,
    };

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer.pubkey()));
    transaction.partial_sign(&[payer as &Keypair], recent_blockhash.clone());
    banks_client.process_transaction(transaction).await
}

async fn call_create_associated_account(
    test_context: &mut TestContext,
    owner: Pubkey,
) -> Result<(), TransportError> {
    let TestContext {
        banks_client,
        recent_blockhash,
        payer,
        keypairs: KeyPairs { mint, .. },
        ..
    } = test_context;
    banks_client
        .process_transaction(create_associated_token_account_transaction(
            &payer,
            &owner,
            &mint,
            *recent_blockhash,
        ))
        .await
}

async fn call_create_vesting_account(
    test_context: &mut TestContext,
    locked_tokens_amount: u64,
) -> Result<(), TransportError> {
    let TestContext {
        banks_client,
        recent_blockhash,
        payer,
        keypairs:
            KeyPairs {
                vesting_type,
                vesting,
                token_pool,
                receiver,
                mint,
                ..
            },
        program_id,
    } = test_context;
    banks_client
        .process_transaction(create_vesting_transaction(
            *program_id,
            &payer,
            &vesting_type,
            &vesting,
            &get_associated_token_address(&receiver.pubkey(), &mint.pubkey()),
            &token_pool,
            *recent_blockhash,
            locked_tokens_amount,
        ))
        .await
}

async fn call_transfer(
    test_context: &mut TestContext,
    amount_to_transfer: u64,
    administrator: bool,
) -> Result<(), TransportError> {
    let TestContext {
        banks_client,
        recent_blockhash,
        payer,
        keypairs:
            KeyPairs {
                mint,
                vesting_type,
                token_pool,
                no_admin,
                ..
            },
        program_id,
    } = test_context;
    let (pda, _bump_seed) =
        Pubkey::find_program_address(&[vesting_type.pubkey().as_ref()], &program_id);
    banks_client
        .process_transaction(withdraw_excessive_from_pool_transaction(
            *program_id,
            &payer,
            if administrator { &payer } else { &no_admin },
            &if administrator {
                get_associated_token_address(&payer.pubkey(), &mint.pubkey())
            } else {
                get_associated_token_address(&no_admin.pubkey(), &mint.pubkey())
            },
            &pda,
            &vesting_type,
            &token_pool,
            *recent_blockhash,
            amount_to_transfer,
        ))
        .await
}
