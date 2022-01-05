use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::Multisig;

use crate::{
    error::VestingError,
    state::{RequiredSigners, VestingTypeAccount},
    utils::write_to_storage,
};

use super::Processor;

impl Processor {
    pub fn create_multisig(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        _instruction_data: &[u8],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let signer = next_account_info(account_info_iter)?;
        let vesting_type = next_account_info(account_info_iter)?;
        let multisig_account = next_account_info(account_info_iter)?;
        let required_signers_account = next_account_info(account_info_iter)?;

        if !signer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let vesting_type_data = VestingTypeAccount::try_from_slice(&vesting_type.data.borrow())?;
        let mut required_signers_data =
            RequiredSigners::try_from_slice(&required_signers_account.data.borrow())?;
        let multisig_data = Multisig::unpack(&multisig_account.data.borrow())?;

        validate_signers(&required_signers_data)?;
        validate_vesting_type(&vesting_type_data, signer)?;

        required_signers_data.vesting_type_account = *vesting_type.key;
        required_signers_data.all_number = multisig_data.n;
        required_signers_data.require_number = multisig_data.m;
        required_signers_data.require_signers = multisig_data.signers;
        write_to_storage(required_signers_data, required_signers_account)
    }
}

fn validate_vesting_type(
    vesting_type_data: &VestingTypeAccount,
    signer: &AccountInfo,
) -> ProgramResult {
    if !vesting_type_data.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    if &vesting_type_data.administrator != signer.key {
        return Err(VestingError::NotAdministrator.into());
    }

    Ok(())
}

fn validate_signers(signers_data: &RequiredSigners) -> ProgramResult {
    if signers_data.is_initialized {
        return Err(VestingError::AlreadyInitialized.into());
    }

    Ok(())
}
