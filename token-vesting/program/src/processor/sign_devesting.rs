use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::instruction::MAX_SIGNERS;

use crate::{
    error::VestingError,
    state::{CurrentSigners, RequiredSigners},
    utils::write_to_storage,
};

use super::Processor;

impl Processor {
    pub fn sign_devesting(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        _instruction_data: &[u8],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let signer = next_account_info(account_info_iter)?;
        let signers_account = next_account_info(account_info_iter)?;
        let required_signers_account = next_account_info(account_info_iter)?;
        let deleted_vesting = next_account_info(account_info_iter)?;
        let vesting_type = next_account_info(account_info_iter)?;

        let required_signers_data =
            RequiredSigners::try_from_slice(&required_signers_account.data.borrow())?;

        let mut signers_data = CurrentSigners::try_from_slice(&signers_account.data.borrow())?;

        let index = required_signers_data
            .require_signers
            .iter()
            .position(|required_signer| required_signer == signer.key)
            .ok_or(ProgramError::MissingRequiredSignature)?;

        validate_required_signers(&required_signers_data, vesting_type)?;
        validate_current_signers(&signers_data, deleted_vesting)?;
        validate_signer(signer, &signers_data, index)?;

        let closing_vesting = validate_signers(
            &signers_data.current_signers,
            required_signers_data.require_number,
        );

        signers_data.current_signers[index] = true;
        
        if closing_vesting {
            Processor::close_vesting_account((vesting_type, deleted_vesting))?;
            let vesting_type_starting_lamports = vesting_type.lamports();
            **vesting_type.lamports.borrow_mut() = vesting_type_starting_lamports
                .checked_add(signers_account.lamports())
                .ok_or(ProgramError::IncorrectProgramId)?;
            **signers_account.lamports.borrow_mut() = 0;
        } else {
            write_to_storage(signers_data, signers_account)?;
        }
        Ok(())
    }
}

pub fn validate_signers(current_signers: &[bool], require: u8) -> bool {
    let num_signers = current_signers
        .iter()
        .fold(0, |sum, sign| if *sign { sum + 1 } else { sum });
    if num_signers < require {
        return false;
    }
    true
}

fn validate_required_signers(
    required_signers_data: &RequiredSigners,
    vesting_type: &AccountInfo,
) -> ProgramResult {
    if !required_signers_data.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    if *vesting_type.key != required_signers_data.vesting_type_account {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

fn validate_current_signers(
    current_signers_data: &CurrentSigners,
    devesting_account: &AccountInfo,
) -> ProgramResult {
    if !current_signers_data.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    if *devesting_account.key != current_signers_data.vesting_account {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

fn validate_signer(
    signer: &AccountInfo,
    signers_data: &CurrentSigners,
    index: usize,
) -> ProgramResult {
    if !signer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if signers_data.current_signers[index] {
        return Err(VestingError::DevestingAlreadySigned.into());
    }

    Ok(())
}
