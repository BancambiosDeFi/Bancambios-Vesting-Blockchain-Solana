use solana_program::msg;
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone)]
pub enum VestingError {
    #[error("Initialized account is already initialized!")]
    AlreadyInitialized,

    #[error("Account isn't rent exempt!")]
    NotRentExempt,

    #[error("Passed vesting schedule is not valid!")]
    ScheduleIsNotValid,

    #[error("Not enough tokens in pool to create vesting!")]
    NotEnoughTokensInPool,

    #[error("Not an administrator of a given Vesting Type!")]
    NotAdministrator,

    #[error("Initialized account has not been initialized yet!")]
    NotInitialized,

    #[error("Not enough unlocked tokens in pool!")]
    NotEnoughUnlockedTokensInPool,

    #[error("Not enough unlocked tokens to withdraw!")]
    NotEnoughUnlockedTokens,

    #[error("Devesting has already signed by account !")]
    DevestingAlreadySigned,
}

impl From<VestingError> for ProgramError {
    fn from(e: VestingError) -> Self {
        msg!(&e.to_string());
        ProgramError::Custom(e as u32)
    }
}
