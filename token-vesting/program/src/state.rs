use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{native_token::lamports_to_sol, pubkey::Pubkey};
use spl_token::instruction::MAX_SIGNERS;

#[derive(Default, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct VestingSchedule {
    pub initial_unlock: u64, //8
    pub start_time: u64,     //8
    pub end_time: u64,       //8
    pub unlock_period: u64,  //8
    pub cliff: u64,          //8
} //40 bytes

#[derive(Default, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct VestingTypeAccount {
    pub is_initialized: bool,              //1
    pub vesting_schedule: VestingSchedule, //40
    pub locked_tokens_amount: u64,         //8
    pub administrator: Pubkey,             //32
    pub token_pool: Pubkey,                //32
} //113 bytes

#[derive(Default, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct RequiredSigners {
    pub is_initialized: bool,                   //1
    pub require_signers: [Pubkey; MAX_SIGNERS], //32 * 11
    pub require_number: u8,                     //1
    pub all_number: u8,                         //1
    pub vesting_type_account: Pubkey,           //32
} //387 bytes

#[derive(Default, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct CurrentSigners {
    pub is_initialized: bool,                 //1
    pub current_signers: [bool; MAX_SIGNERS], //1 * 11
    pub vesting_account: Pubkey,              //32
} //44 bytes

#[derive(Default, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct VestingAccount {
    pub is_initialized: bool,         //1
    pub total_tokens: u64,            //8
    pub withdrawn_tokens: u64,        //8
    pub token_account: Pubkey,        //32
    pub vesting_type_account: Pubkey, //32
} //81 bytes

impl VestingSchedule {
    fn initial_unlock_f64(&self) -> f64 {
        lamports_to_sol(self.initial_unlock)
    }

    pub fn is_valid(&self) -> bool {
        let initial_unlock_f64 = self.initial_unlock_f64();
        self.unlock_period > 0
            && self.start_time < self.end_time
            && self.cliff >= self.start_time
            && self.cliff < self.end_time
            && initial_unlock_f64 >= 0.0
            && initial_unlock_f64 <= 1.0
    }

    pub fn calculate_unlocked_part(&self, now: u64) -> f64 {
        let initial_unlock_f64 = self.initial_unlock_f64();
        if now < self.start_time {
            return 0.0;
        }
        if now < self.cliff {
            return initial_unlock_f64;
        }
        if now > self.end_time {
            return 1.0;
        }

        let mut total_unlocks_count = (self.end_time - self.cliff) / self.unlock_period;
        total_unlocks_count += 1; // for unlock immideately at the end of a cliff
        if (self.end_time - self.cliff) % self.unlock_period > 0 {
            total_unlocks_count += 1; // for a last non-full period
        }
        let part_per_unlock = (1.0 - initial_unlock_f64) / (total_unlocks_count as f64);
        let mut elapsed_unlocks = (now - self.cliff) / self.unlock_period;
        elapsed_unlocks += 1; // unlock immideately at the end of a cliff

        initial_unlock_f64 + part_per_unlock * (elapsed_unlocks as f64)
    }
}

impl VestingAccount {
    pub fn calculate_available_to_withdraw_amount(
        &self,
        schedule: &VestingSchedule,
        now: u64,
    ) -> u64 {
        let unlocked_part = schedule.calculate_unlocked_part(now);
        let unlocked_amount = (unlocked_part * (self.total_tokens as f64)) as u64;
        let unlocked_amount = unlocked_amount.min(self.total_tokens); // safeguard check
        unlocked_amount.saturating_sub(self.withdrawn_tokens)
    }
}

#[cfg(test)]
mod tests {
    use solana_program::native_token::sol_to_lamports;

    use super::*;

    fn construct_test_data() -> (VestingAccount, VestingSchedule) {
        let vesting = VestingAccount {
            total_tokens: 1_000_000,
            withdrawn_tokens: 100_000,
            ..Default::default()
        };

        let schedule = VestingSchedule {
            initial_unlock: sol_to_lamports(0.2),
            start_time: 1_000_000,
            end_time: 2_000_000,
            unlock_period: 400_000,
            cliff: 1_100_000,
        };
        assert!(schedule.is_valid());

        (vesting, schedule)
    }

    #[test]
    fn test_withdraw_amount_before_vesting_start() {
        let (vesting, schedule) = construct_test_data();
        assert_eq!(
            vesting.calculate_available_to_withdraw_amount(&schedule, 900_000),
            0
        );
    }

    #[test]
    fn test_withdraw_amount_after_vesting_end() {
        let (vesting, schedule) = construct_test_data();
        assert_eq!(
            vesting.calculate_available_to_withdraw_amount(&schedule, 2_100_000),
            900_000
        );
    }

    #[test]
    fn test_withdraw_amount_before_cliff_end() {
        let (vesting, schedule) = construct_test_data();
        assert_eq!(
            vesting.calculate_available_to_withdraw_amount(&schedule, 1_050_000),
            100_000
        );
    }

    #[test]
    fn test_withdraw_amount_after_cliff_end() {
        let (vesting, schedule) = construct_test_data();
        assert_eq!(
            vesting.calculate_available_to_withdraw_amount(&schedule, 1_150_000),
            300_000
        );
    }

    #[test]
    fn test_withdraw_amount_after_first_period_unlock() {
        let (vesting, schedule) = construct_test_data();
        assert_eq!(
            vesting.calculate_available_to_withdraw_amount(&schedule, 1_550_000),
            500_000
        );
    }

    #[test]
    fn test_withdraw_amount_before_last_unlock() {
        let (vesting, schedule) = construct_test_data();
        assert_eq!(
            vesting.calculate_available_to_withdraw_amount(&schedule, 1_950_000),
            700_000
        );
    }
}
