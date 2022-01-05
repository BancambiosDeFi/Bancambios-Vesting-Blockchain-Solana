use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

use crate::{
    error::VestingError,
    state::{VestingAccount, VestingTypeAccount},
    utils::write_to_storage,
};

use super::Processor;

impl Processor {
    pub fn withdraw_from_vesting(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let vesting_type = next_account_info(account_info_iter)?;
        let vesting = next_account_info(account_info_iter)?;
        let token_account = next_account_info(account_info_iter)?;
        let token_pool = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        let mut vesting_type_data =
            VestingTypeAccount::try_from_slice(&vesting_type.data.borrow())?;
        let mut vesting_data = VestingAccount::try_from_slice(&vesting.data.borrow())?;
        let (pda, bump_seed) =
            Pubkey::find_program_address(&[vesting_type.key.as_ref()], program_id);

        validate_vesting_type(&vesting_type_data)?;
        validate_vesting(&vesting_data, vesting_type)?;
        validate_token_account(token_account, &vesting_data)?;
        validate_token_pool(token_pool, &vesting_type_data)?;
        validate_pda_account(pda_account, &pda)?;
        validate_token_program_account(token_program)?;
        check_enough_tokens_to_withdraw(&vesting_data, &vesting_type_data, amount)?;

        let transfer_tokens_ix = spl_token::instruction::transfer(
            token_program.key,
            token_pool.key,
            token_account.key,
            &pda,
            &[&pda],
            amount,
        )?;
        invoke_signed(
            &transfer_tokens_ix,
            &[
                token_pool.clone(),
                token_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[vesting_type.key.as_ref(), &[bump_seed]]],
        )?;

        vesting_data.withdrawn_tokens += amount;
        write_to_storage(vesting_data, vesting)?;

        vesting_type_data.locked_tokens_amount -= amount;
        write_to_storage(vesting_type_data, vesting_type)
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

fn validate_token_pool(
    token_pool: &AccountInfo,
    vesting_type_data: &VestingTypeAccount,
) -> ProgramResult {
    if token_pool.key != &vesting_type_data.token_pool {
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

fn validate_token_account(
    token_account: &AccountInfo,
    vesting_data: &VestingAccount,
) -> ProgramResult {
    if token_account.key != &vesting_data.token_account {
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

fn validate_pda_account(pda_account: &AccountInfo, pda: &Pubkey) -> ProgramResult {
    if pda_account.key != pda {
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

fn validate_token_program_account(token_program: &AccountInfo) -> ProgramResult {
    if token_program.key != &spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    Ok(())
}

fn check_enough_tokens_to_withdraw(
    vesting_data: &VestingAccount,
    vesting_type_data: &VestingTypeAccount,
    amount: u64,
) -> ProgramResult {
    let now = Clock::get()?.unix_timestamp as u64;
    let available_to_withdraw = vesting_data
        .calculate_available_to_withdraw_amount(&vesting_type_data.vesting_schedule, now);
    if available_to_withdraw < amount {
        Err(VestingError::NotEnoughUnlockedTokens.into())
    } else {
        Ok(())
    }
}
