use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError::{self, InvalidInstructionData};

use crate::state::{LinearVesting, MAX_VESTINGS};

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum VestingInstruction {
    /// Initializes Vesting Type Account and sets up signer as administrator
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[signer]` The fee payer account (future administrator)
    ///   1. `[writable]` Account to be initialized as Vesting Type Account
    ///   2. `[writable]` Token Account to be transferred as Pool Token Account
    ///   3. `[]` Token program account
    CreateVestingType {
        token_count: u64,
        vesting_count: u8,
        vestings: [(u64, LinearVesting); MAX_VESTINGS],
    },

    /// Creates Vesting Account for specific Vesting Type Account
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[signer]` The fee payer account (administrator)
    ///   1. `[writable]` Vesting Type Account
    ///   2. `[writable]` Account to be initialized as Vesting Account
    ///   3. `[]` Vesting receiver token account
    ///   4. `[]` Pool Token Account for this Vesting Type Account
    CreateVestingAccount { total_tokens: u64 },

    /// Calculates tokens using data from Vesting Type Account and Vesting Account,
    /// and transfers them to Associated Token Account in Vesting Account
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[signer]` The fee payer account
    ///   1. `[writable]` Vesting Type Account
    ///   2. `[]` Vesting Account
    ///   3. `[]` Token Program Account
    WithdrawFromVesting { amount: u64 },

    /// Calculates non-locked tokens using data from Vesting Type Account and Pool Token Account,
    /// and Transfers from Pool Token Account to administrator wallet
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[signer]` The fee payer account (administrator)
    ///   1. `[writable]` Vesting receiver token account (associated account)
    ///   2. `[]` PDA
    ///   3. `[writable]` Pool Token Account for this Vesting Type Account
    ///   4. `[]` Vesting Type Account
    ///   5. `[]` Token Program Account
    WithdrawExcessiveFromPool { amount: u64 },

    /// Changes Vesting Type Account schedule settings
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[signer]` The fee payer account (administrator)
    ///   1. `[writable]` Vesting Type Account
    ChangeVestingTypeSchedule {
        token_count: u64,
        vesting_count: u8,
        vestings: [(u64, LinearVesting); MAX_VESTINGS],
    },

    /// Create multisig
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[signer]` The fee payer account (administrator)
    ///   1. `[writable]` Vesting Type Account
    ///   2. `[writable]` Multisig Account
    ///   3. `[writable]` Signers Account
    CreateMultisig,

    /// Sign devesting
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[signer]` The fee payer account
    ///   1. `[writable]` Current Signers Account
    ///   2. `[writable]` Required Signers Account
    ///   3. `[writable]` Vesting Account which will be deleted
    ///   4. `[writable]` Vesting Type Account
    SignDevesting,
}

impl VestingInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        Self::try_from_slice(input).or(Err(InvalidInstructionData))
    }

    pub fn pack(&self) -> Vec<u8> {
        self.try_to_vec().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_instruction_packing() {
        let mut vestings: [(u64, LinearVesting); MAX_VESTINGS] = Default::default();
        vestings[0] = (400_000, LinearVesting::cliff(20));
        vestings[1] = (600_000, LinearVesting::new(50, 50, 3));
        let original_create = VestingInstruction::CreateVestingType {
            token_count: 1_000_000,
            vesting_count: 2,
            vestings,
        };
        let packed_create = original_create.pack();
        let unpacked_create = VestingInstruction::unpack(&packed_create).unwrap();
        assert_eq!(original_create, unpacked_create);

        let original_unlock = VestingInstruction::CreateVestingAccount { total_tokens: 400 };
        assert_eq!(
            original_unlock,
            VestingInstruction::unpack(&original_unlock.pack()).unwrap()
        );

        let original_init = VestingInstruction::WithdrawExcessiveFromPool { amount: 4000 };
        assert_eq!(
            original_init,
            VestingInstruction::unpack(&original_init.pack()).unwrap()
        );

        let original_change = VestingInstruction::WithdrawExcessiveFromPool { amount: 10 };
        assert_eq!(
            original_change,
            VestingInstruction::unpack(&original_change.pack()).unwrap()
        );
        let original_change = VestingInstruction::ChangeVestingTypeSchedule {
            token_count: 1_000_000,
            vesting_count: 2,
            vestings,
        };
        assert_eq!(
            original_change,
            VestingInstruction::unpack(&original_change.pack()).unwrap()
        );
    }
}
