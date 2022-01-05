use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};
use spl_token::state::Account as TokenAccount;

use super::Processor;
use crate::{
    error::VestingError,
    state::{VestingAccount, VestingTypeAccount},
    utils::write_to_storage,
};

impl Processor {
    pub fn create_vesting_account(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        total_tokens: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let signer = next_account_info(account_info_iter)?;
        let vesting_type = next_account_info(account_info_iter)?;
        let vesting = next_account_info(account_info_iter)?;
        let token_account = next_account_info(account_info_iter)?;
        let token_pool = next_account_info(account_info_iter)?;

        if !signer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut vesting_type_data =
            VestingTypeAccount::try_from_slice(&vesting_type.data.borrow())?;
        let mut vesting_data = VestingAccount::try_from_slice(&vesting.data.borrow())?;
        let token_account_data = TokenAccount::unpack(&token_account.data.borrow())?;
        let token_pool_data = TokenAccount::unpack(&token_pool.data.borrow())?;

        validate_vesting(vesting, &vesting_data)?;
        validate_vesting_type(&vesting_type_data, signer)?;
        validate_token_pool(token_pool, &vesting_type_data)?;
        validate_token_account(token_account, &token_account_data, &token_pool_data)?;
        check_enough_tokens(&vesting_type_data, &token_pool_data, total_tokens)?;

        vesting_data.is_initialized = true;
        vesting_data.total_tokens = total_tokens;
        vesting_data.withdrawn_tokens = 0;
        vesting_data.token_account = *token_account.key;
        vesting_data.vesting_type_account = *vesting_type.key;
        write_to_storage(vesting_data, vesting)?;

        vesting_type_data.locked_tokens_amount += total_tokens;
        write_to_storage(vesting_type_data, vesting_type)
    }
}

fn validate_vesting(vesting: &AccountInfo, vesting_data: &VestingAccount) -> ProgramResult {
    if vesting_data.is_initialized {
        return Err(VestingError::AlreadyInitialized.into());
    }

    let rent = Rent::get()?;
    if !rent.is_exempt(vesting.lamports(), vesting.data_len()) {
        return Err(VestingError::NotRentExempt.into());
    }

    Ok(())
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
    token_account_data: &TokenAccount,
    token_pool_data: &TokenAccount,
) -> ProgramResult {
    if token_account.owner != &spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    if token_account_data.mint != token_pool_data.mint {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

fn check_enough_tokens(
    vesting_type_data: &VestingTypeAccount,
    token_pool_data: &TokenAccount,
    total_tokens: u64,
) -> ProgramResult {
    if vesting_type_data.locked_tokens_amount + total_tokens > token_pool_data.amount {
        return Err(VestingError::NotEnoughTokensInPool.into());
    }

    Ok(())
}
