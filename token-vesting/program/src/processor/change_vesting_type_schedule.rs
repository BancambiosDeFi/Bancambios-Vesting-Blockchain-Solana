use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use std::convert::TryFrom;

use super::Processor;
use crate::{
    error::VestingError,
    state::{VestingSchedule, VestingTypeAccount},
    utils::write_to_storage,
};

#[derive(Clone, Copy)]
struct Accounts<'a, 'b> {
    signer: &'a AccountInfo<'b>,
    vesting_type: &'a AccountInfo<'b>,
}

impl<'a, 'b> TryFrom<&'a [AccountInfo<'b>]> for Accounts<'a, 'b> {
    type Error = ProgramError;

    fn try_from(value: &'a [AccountInfo<'b>]) -> Result<Self, Self::Error> {
        let account_info_iter = &mut value.iter();

        let signer = next_account_info(account_info_iter)?;
        let vesting_type = next_account_info(account_info_iter)?;

        if !signer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(Accounts {
            signer,
            vesting_type,
        })
    }
}

impl Processor {
    pub fn change_vesting_type_schedule(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        initial_unlock: u64,
        start_time: u64,
        end_time: u64,
        unlock_period: u64,
        cliff: u64,
    ) -> ProgramResult {
        let Accounts {
            signer,
            vesting_type,
        } = Accounts::try_from(accounts)?;

        let new_vesting_schedule = VestingSchedule {
            initial_unlock,
            start_time,
            end_time,
            unlock_period,
            cliff,
        };

        // check if the old schedule exists
        let mut vesting_type_data =
            VestingTypeAccount::try_from_slice(&vesting_type.data.borrow())?;
        if !vesting_type_data.is_initialized {
            return Err(VestingError::NotInitialized.into());
        }

        // check administrator
        if &vesting_type_data.administrator != signer.key {
            return Err(VestingError::NotAdministrator.into());
        }

        // check data for new schedule
        if !new_vesting_schedule.is_valid() {
            return Err(VestingError::ScheduleIsNotValid.into());
        }

        // change the old schedule to the new one
        vesting_type_data.vesting_schedule = new_vesting_schedule;

        write_to_storage(vesting_type_data, vesting_type)
    }
}
