pub mod change_vesting_type_schedule;
pub mod close_vesting_account;
pub mod create_multisig;
pub mod create_vesting_account;
pub mod create_vesting_type;
pub mod sign_devesting;
#[cfg(test)]
mod tests;
pub mod withdraw_excessive_from_pool;
pub mod withdraw_from_vesting;
use crate::instruction::VestingInstruction;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};
pub struct Processor {}

impl Processor {
    pub fn instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("Beginning processing");
        let instruction = VestingInstruction::unpack(instruction_data)?;
        msg!("Instruction unpacked");
        match instruction {
            VestingInstruction::CreateVestingType {
                token_count,
                vesting_count,
                vestings,
            } => {
                msg!("Instruction: Create Vesting Type");
                Self::create_vesting_type(
                    program_id,
                    accounts,
                    token_count,
                    &vestings[..vesting_count as usize],
                )
            }
            VestingInstruction::CreateVestingAccount { total_tokens } => {
                msg!("Instruction: Create Vesting");
                Self::create_vesting_account(program_id, accounts, total_tokens)
            }
            VestingInstruction::WithdrawFromVesting { amount } => {
                msg!("Instruction: Withdraw From Vesting");
                Self::withdraw_from_vesting(program_id, accounts, amount)
            }
            VestingInstruction::WithdrawExcessiveFromPool { amount } => {
                msg!("Instruction: Withdraw Excessive From Pool");
                Self::withdraw_excessive_from_pool(program_id, accounts, amount)
            }
            VestingInstruction::ChangeVestingTypeSchedule {
                token_count,
                vesting_count,
                vestings,
            } => {
                msg!("Instruction: Change Vesting Type Schedule");
                panic!("Changing vesting type is forbidden")
                // Self::change_vesting_type_schedule(
                //     program_id, accounts,
                //     token_count,
                //     &vestings[..vesting_count as usize],
                // )
            }
            VestingInstruction::CreateMultisig => {
                Processor::create_multisig(program_id, accounts, instruction_data)
            }
            VestingInstruction::SignDevesting => {
                Processor::sign_devesting(program_id, accounts, instruction_data)
            }
        }
    }
}
