use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::{error::TokenError, instruction::MAX_SIGNERS, state::Multisig};

use crate::state::Signers;

pub struct Processor {}

impl Processor {
    fn create(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        _instruction_data: &[u8],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let signer = next_account_info(account_info_iter)?;
        let multisig_account = next_account_info(account_info_iter)?;
        let signers_account = next_account_info(account_info_iter)?;

        let mut signers = Signers::try_from_slice(&signers_account.data.borrow())?;
        if signers.is_initialized {
            return Err(ProgramError::UninitializedAccount);
        }
        let multisig = Multisig::unpack(&multisig_account.data.borrow())?;
        signers.all_number = multisig.n;
        signers.require_number = multisig.m;
        signers.require_signers = multisig.signers;
        signers.current_signers = Default::default();
        write_to_storage(signers, signers_account)
    }

    fn sign(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        _instruction_data: &[u8],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let signer = next_account_info(account_info_iter)?;
        let signers_account = next_account_info(account_info_iter)?;
        let account_which_vesting_will_be_deleted = next_account_info(account_info_iter)?;
        let vesting_type_account = next_account_info(account_info_iter)?;

        if !signer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let mut signers = Signers::try_from_slice(&signers_account.data.borrow())?;
        if !signers.is_initialized {
            return Err(ProgramError::UninitializedAccount);
        }
        signers.current_signers[signers.current_signers.len()] = *signer.key;
        validate_owner(
            program_id,
            signers_account,
            &signers.current_signers,
            &signers.require_signers,
            signers.all_number,
            signers.require_number,
        )?;

        // TODO: invoke transaction or call function

        signers.current_signers = Default::default();
        write_to_storage(signers, signers_account)
    }
}

pub fn validate_owner(
    program_id: &Pubkey,
    multisig_info: &AccountInfo,
    current_signers: &[Pubkey],
    require_signers: &[Pubkey],
    all: u8,
    require: u8,
) -> ProgramResult {
    if program_id != multisig_info.owner {
        return Err(ProgramError::InvalidAccountData);
    }
    let mut num_signers = 0;
    let mut matched = [false; MAX_SIGNERS];
    for signer in require_signers.iter() {
        for (position, key) in current_signers[0..all as usize].iter().enumerate() {
            if key == signer && !matched[position] {
                matched[position] = true;
                num_signers += 1;
            }
        }
    }
    if num_signers < require {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(())
}

pub fn write_to_storage<T>(data: T, account: &AccountInfo) -> ProgramResult
where
    T: BorshSerialize,
{
    let bytes = data.try_to_vec()?;
    let mut storage = account.try_borrow_mut_data()?;
    storage[0..bytes.len()].clone_from_slice(&bytes);

    Ok(())
}
