use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

use crate::{
    state::{VestingAccount, VestingTypeAccount},
    utils::write_to_storage,
};

use super::Processor;

impl Processor {
    pub fn close_vesting_account(accounts: (&AccountInfo, &AccountInfo)) -> ProgramResult {
        let (vesting_type, vesting) = accounts;

        let mut vesting_type_data =
            VestingTypeAccount::try_from_slice(&vesting_type.data.borrow())?;
        let mut vesting_data = VestingAccount::try_from_slice(&vesting.data.borrow())?;

        validate_vesting_type(&vesting_type_data)?;
        validate_vesting(&vesting_data, vesting_type)?;

        vesting_type_data.locked_tokens_amount -=
            vesting_data.total_tokens - vesting_data.withdrawn_tokens;
        write_to_storage(vesting_type_data, vesting_type)?;

        vesting_data = Default::default();
        write_to_storage(vesting_data, vesting)?;

        // reducing amount of lamports to 0 for deleting account
        let vesting_type_starting_lamports = vesting_type.lamports();
        **vesting_type.lamports.borrow_mut() = vesting_type_starting_lamports
            .checked_add(vesting.lamports())
            .ok_or(ProgramError::IncorrectProgramId)?;
        **vesting.lamports.borrow_mut() = 0;

        Ok(())
    }
}

fn validate_vesting(vesting_data: &VestingAccount, vesting_type: &AccountInfo) -> ProgramResult {
    if !vesting_data.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    if &vesting_data.vesting_type_account != vesting_type.key {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

fn validate_vesting_type(vesting_type_data: &VestingTypeAccount) -> ProgramResult {
    if !vesting_type_data.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    Ok(())
}
