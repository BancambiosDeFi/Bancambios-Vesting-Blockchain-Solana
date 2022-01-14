use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};
use spl_token::state::Account as TokenAccount;
use std::convert::TryFrom;

use super::Processor;
use crate::{
    error::VestingError,
    state::{LinearVesting, VestingSchedule, VestingTypeAccount},
    utils::write_to_storage,
};

#[derive(Clone, Copy)]
struct Accounts<'a, 'b> {
    signer: &'a AccountInfo<'b>,
    vesting_type: &'a AccountInfo<'b>,
    token_pool: &'a AccountInfo<'b>,
    token_program: &'a AccountInfo<'b>,
}

impl<'a, 'b> TryFrom<&'a [AccountInfo<'b>]> for Accounts<'a, 'b> {
    type Error = ProgramError;

    fn try_from(value: &'a [AccountInfo<'b>]) -> Result<Self, Self::Error> {
        let account_info_iter = &mut value.iter();

        let signer = next_account_info(account_info_iter)?;
        let vesting_type = next_account_info(account_info_iter)?;
        let token_pool = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        if !signer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(Accounts {
            signer,
            vesting_type,
            token_pool,
            token_program,
        })
    }
}

impl Processor {
    pub fn create_vesting_type(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        token_count: u64,
        vestings: &[(u64, LinearVesting)],
    ) -> ProgramResult {
        let accounts = Accounts::try_from(accounts)?;

        let vesting_schedule = VestingSchedule::new(token_count, &vestings);
        check_and_initialize_vesting_type(accounts, vesting_schedule)?;
        check_and_transfer_token_pool(program_id, accounts)
    }
}

fn check_and_initialize_vesting_type(
    accounts: Accounts,
    vesting_schedule: VestingSchedule,
) -> ProgramResult {
    let Accounts {
        signer,
        vesting_type,
        token_pool,
        ..
    } = accounts;

    let mut vesting_type_data = VestingTypeAccount::try_from_slice(&vesting_type.data.borrow())?;
    if vesting_type_data.is_initialized {
        return Err(VestingError::AlreadyInitialized.into());
    }

    let rent = Rent::get()?;
    if !rent.is_exempt(vesting_type.lamports(), vesting_type.data_len()) {
        return Err(VestingError::NotRentExempt.into());
    }

    if !vesting_schedule.is_valid() {
        return Err(VestingError::ScheduleIsNotValid.into());
    }

    vesting_type_data.is_initialized = true;
    vesting_type_data.vesting_schedule = vesting_schedule;
    vesting_type_data.locked_tokens_amount = 0;
    vesting_type_data.administrator = *signer.key;
    vesting_type_data.token_pool = *token_pool.key;

    write_to_storage(vesting_type_data, vesting_type)
}

fn check_and_transfer_token_pool(program_id: &Pubkey, accounts: Accounts) -> ProgramResult {
    let Accounts {
        signer,
        vesting_type,
        token_pool,
        token_program,
    } = accounts;

    if token_program.key != &spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    if token_pool.owner != token_program.key {
        return Err(ProgramError::IncorrectProgramId);
    }

    TokenAccount::unpack(&token_pool.data.borrow())?; // checks that account is initialized
    let (pda, _bump_seed) = Pubkey::find_program_address(&[vesting_type.key.as_ref()], program_id);
    let owner_change_ix = spl_token::instruction::set_authority(
        token_program.key,
        token_pool.key,
        Some(&pda),
        spl_token::instruction::AuthorityType::AccountOwner,
        signer.key,
        &[signer.key],
    )?;

    invoke(
        &owner_change_ix,
        &[token_pool.clone(), signer.clone(), token_program.clone()],
    )
}
