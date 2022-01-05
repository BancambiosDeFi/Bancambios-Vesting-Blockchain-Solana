use std::str::FromStr;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    hash::Hash, instruction::InstructionError, program_pack::Pack, pubkey::Pubkey, rent::Rent,
};
use solana_program_test::{processor, BanksClient, ProgramTest};
use solana_sdk::{
    account::Account, signature::Keypair, signer::Signer, transaction::TransactionError,
    transport::TransportError,
};
use spl_token::state::Account as TokenAccount;

use crate::entrypoint::process_instruction;

pub struct ErrorChecker {
    result: Result<(), TransportError>,
}

impl ErrorChecker {
    pub fn check(&self, expected: InstructionError) {
        let no_error_msg = format!("Expected {}, found no error", expected);
        let wrapped_error = self.result.as_ref().err().expect(no_error_msg.as_str());
        if let TransportError::TransactionError(TransactionError::InstructionError(
            0,
            inner_error,
        )) = wrapped_error
        {
            assert_eq!(&expected, inner_error);
        } else {
            panic!("Got not an instruction error: {}", wrapped_error);
        }
    }
}

impl From<Result<(), TransportError>> for ErrorChecker {
    fn from(result: Result<(), TransportError>) -> Self {
        Self { result }
    }
}

pub struct AbstractTestContext<Keys> {
    pub program_id: Pubkey,
    pub banks_client: BanksClient,
    pub recent_blockhash: Hash,
    pub payer: Keypair,
    pub keypairs: Keys,
}

impl<Keys> AbstractTestContext<Keys> {
    pub async fn new(add_accounts: impl FnOnce(&mut ProgramTest, Pubkey, &Keys) -> ()) -> Self
    where
        Keys: Default,
    {
        let program_id = Pubkey::from_str("GsuUkbKohL8sEa3k6sgs2icHk8MN1Cz8cwh4G1ZKB6P9").unwrap();

        let keypairs = Keys::default();

        let mut program_test =
            ProgramTest::new("vesting", program_id, processor!(process_instruction));

        add_accounts(&mut program_test, program_id, &keypairs);

        let (banks_client, payer, recent_blockhash) = program_test.start().await;

        Self {
            program_id,
            banks_client,
            recent_blockhash,
            payer,
            keypairs,
        }
    }
}

pub fn add_account<DataType: Default + BorshSerialize>(
    program_test: &mut ProgramTest,
    owner: Pubkey,
    account: &Keypair,
    rent_exempt: bool,
) {
    let data = DataType::default().try_to_vec().unwrap();
    let rent = Rent::default();
    program_test.add_account(
        account.pubkey(),
        Account {
            lamports: rent.minimum_balance(data.len()) - (if rent_exempt { 0 } else { 1 }),
            owner,
            data,
            ..Account::default()
        },
    );
}

pub async fn deserialize_account<DataType>(banks_client: &mut BanksClient, key: Pubkey) -> DataType
where
    DataType: BorshDeserialize,
{
    let account = banks_client.get_account(key).await.ok().flatten().unwrap();
    DataType::deserialize(&mut account.data.as_slice()).unwrap()
}

pub async fn deserialize_token_account(
    banks_client: &mut BanksClient,
    key: Pubkey,
) -> TokenAccount {
    let account = banks_client.get_account(key).await.ok().flatten().unwrap();
    TokenAccount::unpack(account.data.as_slice()).unwrap()
}
