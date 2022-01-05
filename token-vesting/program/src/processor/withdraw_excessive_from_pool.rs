use std::convert::TryFrom;

use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
};
use spl_token::state::Account as TokenAccount;

use crate::{error::VestingError, state::VestingTypeAccount};

use super::Processor;
#[derive(Clone, Copy)]
struct Accounts<'a, 'b> {
    signer: &'a AccountInfo<'b>,
    associated_account: &'a AccountInfo<'b>,
    pda_account: &'a AccountInfo<'b>,
    token_pool: &'a AccountInfo<'b>,
    vesting_type: &'a AccountInfo<'b>,
    token_program: &'a AccountInfo<'b>,
}

impl<'a, 'b> TryFrom<&'a [AccountInfo<'b>]> for Accounts<'a, 'b> {
    type Error = ProgramError;

    fn try_from(value: &'a [AccountInfo<'b>]) -> Result<Self, Self::Error> {
        let account_info_iter = &mut value.iter();

        let signer = next_account_info(account_info_iter)?;
        let associated_account = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;
        let token_pool = next_account_info(account_info_iter)?;
        let vesting_type = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        if !signer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(Accounts {
            signer,
            associated_account,
            pda_account,
            token_pool,
            vesting_type,
            token_program,
        })
    }
}

impl Processor {
    pub fn withdraw_excessive_from_pool(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let Accounts {
            signer,
            associated_account,
            pda_account,
            token_pool,
            vesting_type,
            token_program,
        } = Accounts::try_from(accounts)?;

        let vesting_type_data = VestingTypeAccount::try_from_slice(&vesting_type.data.borrow())?;
        let token_pool_data = TokenAccount::unpack(&token_pool.data.borrow())?;

        let total_tokens = token_pool_data.amount;
        let unlocked_tokens = total_tokens - vesting_type_data.locked_tokens_amount;

        let (pda, bump_seed) =
            Pubkey::find_program_address(&[vesting_type.key.as_ref()], program_id);

        validate_vesting_type(&vesting_type_data, vesting_type, signer, program_id)?;
        validate_token_pool(
            token_pool,
            token_pool_data,
            &vesting_type_data,
            token_program,
        )?;
        validate_token_program(token_program)?;
        validate_amount(amount, unlocked_tokens)?;
        validate_pda_account(pda_account, &pda)?;

        let seed = &[vesting_type.key.as_ref(), &[bump_seed]];

        let transfer_tokens = spl_token::instruction::transfer(
            token_program.key,
            token_pool.key,
            associated_account.key,
            &pda,
            &[&pda],
            amount,
        )?;

        invoke_signed(
            &transfer_tokens,
            &[
                token_program.clone(),
                token_pool.clone(),
                associated_account.clone(),
                signer.clone(),
                vesting_type.clone(),
                pda_account.clone(),
            ],
            &[seed],
        )
    }
}

fn validate_vesting_type(
    vesting_type_data: &VestingTypeAccount,
    vesting_type: &AccountInfo,
    signer: &AccountInfo,
    program_id: &Pubkey,
) -> ProgramResult {
    if vesting_type.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

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
    token_pool_data: TokenAccount,
    vesting_type_data: &VestingTypeAccount,
    token_program: &AccountInfo,
) -> ProgramResult {
    if token_pool.key != &vesting_type_data.token_pool {
        return Err(ProgramError::IncorrectProgramId);
    }

    if token_pool.owner != token_program.key {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !token_pool_data.is_initialized() {
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

fn validate_token_program(token_program: &AccountInfo) -> ProgramResult {
    if token_program.key != &spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }
    Ok(())
}

fn validate_amount(amount: u64, unlocked_tokens: u64) -> ProgramResult {
    if amount > unlocked_tokens {
        return Err(VestingError::NotEnoughUnlockedTokensInPool.into());
    }
    Ok(())
}
