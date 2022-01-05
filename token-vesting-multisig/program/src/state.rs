use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use spl_token::instruction::MAX_SIGNERS;

#[derive(Default, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct Signers {
    pub is_initialized: bool,                   //1
    pub current_signers: [Pubkey; MAX_SIGNERS], //32 * 11
    pub require_signers: [Pubkey; MAX_SIGNERS], //32 * 11
    pub require_number: u8,                     //1
    pub all_number: u8,                         //1
} //707 bytes
