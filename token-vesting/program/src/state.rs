use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use spl_token::instruction::MAX_SIGNERS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WithStart;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WithoutStart;

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct LinearVesting<T = WithStart> {
    start_time: u64,                       // 8
    unlock_period: u64,                    // 8
    unlock_count: u8,                      // 1
    phantom: core::marker::PhantomData<T>, // 0
} // 17 bytes

impl LinearVesting {
    pub fn new(start_time: u64, unlock_period: u64, unlock_count: u8) -> LinearVesting<WithStart> {
        LinearVesting {
            start_time,
            unlock_period,
            unlock_count: unlock_count.max(1),
            phantom: core::marker::PhantomData,
        }
    }

    pub fn without_start(unlock_period: u64, unlock_count: u8) -> LinearVesting<WithoutStart> {
        LinearVesting::<WithoutStart> {
            start_time: 0,
            unlock_period,
            unlock_count: unlock_count.max(1),
            phantom: core::marker::PhantomData,
        }
    }

    pub fn cliff(start_time: u64) -> LinearVesting<WithStart> {
        LinearVesting::new(start_time, 0, 1)
    }

    pub fn remove_start(&self) -> LinearVesting<WithoutStart> {
        LinearVesting::<WithoutStart> {
            start_time: 0,
            unlock_count: self.unlock_count,
            unlock_period: self.unlock_period,
            phantom: core::marker::PhantomData,
        }
    }

    pub fn last(&self) -> u64 {
        self.start_time + self.unlock_period * (self.unlock_count - 1) as u64
    }

    pub fn available(&self, mut time: u64) -> f64 {
        if time < self.start_time {
            return 0.0;
        }
        if time >= self.last() {
            return 1.0;
        }
        time -= self.start_time;
        return self.part() * (time / self.unlock_period + 1) as f64;
    }

    pub fn start_time(&self) -> u64 {
        self.start_time
    }
}

impl<T> LinearVesting<T> {
    pub fn part(&self) -> f64 {
        1f64 / self.unlock_count as f64
    }

    pub fn unlock_count(&self) -> u8 {
        self.unlock_count
    }

    pub fn unlock_period(&self) -> u64 {
        self.unlock_period
    }
}

impl Default for LinearVesting {
    fn default() -> Self {
        LinearVesting::new(0, 0, 0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, BorshSerialize, BorshDeserialize)]
pub struct VestingSchedule {
    token_count: u64,                                                // 8
    vesting_count: u8,                                               // 1
    vestings: [(u64, LinearVesting); VestingSchedule::MAX_VESTINGS], // 25 * 16 = 400
} // 407 bvtes

impl VestingSchedule {
    pub const MAX_VESTINGS: usize = 16;

    pub fn new(total_tokens: u64, vestings: &[(u64, LinearVesting)]) -> VestingSchedule {
        assert!(vestings.len() <= VestingSchedule::MAX_VESTINGS);
        let mut vestings_: [(u64, LinearVesting); VestingSchedule::MAX_VESTINGS] =
            Default::default();
        vestings_[..vestings.len()].copy_from_slice(vestings);
        VestingSchedule {
            token_count: total_tokens,
            vesting_count: vestings.len() as u8,
            vestings: vestings_,
        }
    }

    pub fn available(&self, time: u64) -> u64 {
        let mut tokens = 0;
        for tv in self.vestings.iter() {
            if tv.1.start_time > time {
                break;
            }
            tokens += (tv.1.available(time) * tv.0 as f64) as u64
        }
        tokens
    }

    pub fn total_tokens(&self) -> u64 {
        self.token_count
    }

    pub fn is_valid(&self) -> bool {
        if self.vesting_count as usize > VestingSchedule::MAX_VESTINGS {
            return false;
        }

        // TODO: use slice::is_sorted_by when it becomes stable
        let mut is_sorted = true;
        for i in 1..self.vesting_count {
            if self.vestings[i as usize - 1].1.last() > self.vestings[i as usize].1.start_time {
                is_sorted = false;
                break;
            }
        }
        if !is_sorted {
            return false;
        }

        for i in self.vestings().iter() {
            if i.0 == 0 {
                return false;
            }
        }

        return true;
    }

    pub fn start_time(&self) -> u64 {
        self.vestings[0].1.start_time()
    }

    pub fn last(&self) -> u64 {
        self.vestings[self.vesting_count as usize - 1].1.last()
    }

    pub fn vestings(&self) -> &[(u64, LinearVesting)] {
        &self.vestings[..self.vesting_count as usize]
    }

    pub fn token_count(&self) -> u64 {
        self.token_count
    }
}

#[derive(Default, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct VestingTypeAccount {
    pub is_initialized: bool,              // 1
    pub vesting_schedule: VestingSchedule, // 416
    pub locked_tokens_amount: u64,         // 8
    pub administrator: Pubkey,             // 32
    pub token_pool: Pubkey,                // 32
} // 489 bytes

#[derive(Default, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct RequiredSigners {
    pub is_initialized: bool,                   // 1
    pub require_signers: [Pubkey; MAX_SIGNERS], // 32 * 11
    pub require_number: u8,                     // 1
    pub all_number: u8,                         // 1
    pub vesting_type_account: Pubkey,           // 32
} // 387 bytes

#[derive(Default, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct CurrentSigners {
    pub is_initialized: bool,                 // 1
    pub current_signers: [bool; MAX_SIGNERS], // 1 * 11
    pub vesting_account: Pubkey,              // 32
} // 44 bytes

#[derive(Default, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct VestingAccount {
    pub is_initialized: bool,         // 1
    pub total_tokens: u64,            // 8
    pub withdrawn_tokens: u64,        // 8
    pub token_account: Pubkey,        // 32
    pub vesting_type_account: Pubkey, // 32
} // 81 bytes

impl VestingAccount {
    pub fn calculate_available_to_withdraw_amount(
        &self,
        schedule: &VestingSchedule,
        now: u64,
    ) -> u64 {
        let unlocked_amount = schedule.available(now);
        let unlocked_amount = unlocked_amount.min(self.total_tokens); // safeguard check
        unlocked_amount.saturating_sub(self.withdrawn_tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::{ScheduleBuilder, ScheduleBuilderError};

    #[test]
    fn test_builder_success() {
        let cliff = 20_000;
        let offseted_by = 30_000;
        let standalone = 200_000;

        let schedule = ScheduleBuilder::with_tokens(1_000_000)
            .cliff(cliff, Some(100_000))
            .offseted_by(
                offseted_by,
                LinearVesting::without_start(10_000, 3),
                Some(100_000),
            )
            .map(|x| x.offseted(LinearVesting::without_start(20_000, 5), Some(100_000)))
            .and_then(|x| match x {
                Err(e) => Err(e),
                Ok(x) => Ok(x.add(LinearVesting::new(standalone, 10_000, 2), None)),
            })
            .and_then(|x| x.build());
        assert!(schedule.is_ok());

        let schedule = schedule.unwrap();
        assert_eq!(schedule.total_tokens(), 1_000_000);
        assert_eq!(
            &schedule.vestings[..schedule.vesting_count as usize],
            &[
                (100_000, LinearVesting::cliff(cliff)),
                (100_000, LinearVesting::new(cliff + offseted_by, 10_000, 3)),
                (
                    100_000,
                    LinearVesting::new(cliff + offseted_by + 10_000 * (3 - 1) + 20_000, 20_000, 5)
                ),
                (700_000, LinearVesting::new(standalone, 10_000, 2)),
            ]
        )
    }

    #[test]
    fn test_builder_failure_offset() {
        let schedule = ScheduleBuilder::with_tokens(1_000_000).offseted_by(
            10_000,
            LinearVesting::without_start(10_000, 3),
            None,
        );
        assert_eq!(schedule, Err(ScheduleBuilderError::EmptyBuilder))
    }

    #[test]
    fn test_builder_failure_remaining_tokens() {
        let schedule = ScheduleBuilder::with_tokens(1_000_000)
            .cliff(10_000, Some(100_000))
            .build();
        assert_eq!(
            schedule,
            Err(ScheduleBuilderError::InvalidTokenAmountUsed((
                1_000_000, 100_000
            )))
        )
    }

    #[test]
    fn test_builder_failure_unsorted_vestings() {
        let schedule = ScheduleBuilder::with_tokens(1_000_000)
            .add(LinearVesting::new(10_000, 10_000, 3), Some(100_000))
            .add(LinearVesting::new(20_000, 10_000, 3), None)
            .build();
        assert_eq!(schedule, Err(ScheduleBuilderError::VestingsNotSorted))
    }

    #[test]
    fn test_builder_failure_zero_token() {
        let schedule = ScheduleBuilder::with_tokens(1_000_000)
            .add(LinearVesting::new(10_000, 10_000, 3), None)
            .add(LinearVesting::new(50_000, 10_000, 3), None)
            .build();
        assert_eq!(schedule, Err(ScheduleBuilderError::ZeroTokens))
    }

    #[test]
    fn test_builder_failure_too_many_vestings() {
        let mut builder = ScheduleBuilder::with_tokens(1_000_000);
        for i in 0..VestingSchedule::MAX_VESTINGS {
            builder = builder.cliff(i as u64 * 100, Some(100));
        }
        builder = builder
            .offseted_by(100, LinearVesting::without_start(0, 1), None)
            .unwrap();
        let schedule = builder.build();
        assert_eq!(schedule, Err(ScheduleBuilderError::TooManyVestings))
    }

    #[test]
    fn test_builder_failure_initial_unlock_too_big() {
        let schedule = ScheduleBuilder::with_tokens(1_000_000).legacy(
            10_000,
            12_000,
            100,
            11_000,
            10_000,
            Some(10_000),
        );
        assert_eq!(schedule, Err(ScheduleBuilderError::InitialUnlockTooBig))
    }

    #[test]
    fn test_builder_failure_invalid_end_time() {
        let schedule = ScheduleBuilder::with_tokens(1_000_000).legacy(
            10_000,
            1_000,
            10_000,
            10_000,
            100_000,
            Some(10_000),
        );
        assert_eq!(schedule, Err(ScheduleBuilderError::InvalidTimeInterval))
    }

    #[test]
    fn test_builder_failure_invalid_cliff() {
        let schedule = ScheduleBuilder::with_tokens(1_000_000).legacy(
            10_000,
            1_000,
            100_000,
            100_000,
            10_000,
            Some(10_000),
        );
        assert_eq!(schedule, Err(ScheduleBuilderError::InvalidTimeInterval))
    }

    #[test]
    fn test_vesting_cliff_available_tokens() {
        let start_time = 100;
        let vesting = LinearVesting::cliff(100);

        assert_eq!(vesting.last(), start_time);
        assert_eq!(vesting.part(), 1.0);

        assert_eq!(vesting.available(u64::MIN), 0.0);
        assert_eq!(vesting.available(start_time - 10), 0.0);
        assert_eq!(vesting.available(start_time), 1.0);
        assert_eq!(vesting.available(start_time + 10), 1.0);
        assert_eq!(vesting.available(u64::MAX), 1.0);
    }

    #[test]
    fn test_vesting_available_tokens() {
        let start_time = 100;
        let period = 10;
        let unlocks = 7;
        let vesting = LinearVesting::new(start_time, period, unlocks);

        assert_eq!(vesting.last(), start_time + period * (unlocks - 1) as u64);
        assert_eq!(vesting.part(), 1.0 / unlocks as f64);

        assert_eq!(vesting.available(u64::MIN), 0.0);

        let almost_eq = |a: f64, b: f64| (a - b).abs() < 0.0001;

        for i in 1..=unlocks {
            let time = start_time + (i - 1) as u64 * 10;
            assert!(almost_eq(
                vesting.available(time - period / 2),
                vesting.part() * i as f64 - vesting.part()
            ));
            assert!(almost_eq(
                vesting.available(time),
                vesting.part() * i as f64
            ));
            assert!(almost_eq(
                vesting.available(time + period / 2),
                vesting.part() * i as f64
            ));
        }
        assert_eq!(vesting.available(u64::MAX), 1.0);
    }

    #[test]
    fn test_schedule_available_tokens() {
        let total_tokens = 1_000_000;
        let cliff1 = total_tokens * 6 / 100;
        let cliff2 = total_tokens * 9 / 100;

        const MINUTE: u64 = 60;
        const HOUR: u64 = 60 * MINUTE;
        const DAY: u64 = 24 * HOUR;
        const WEEK: u64 = 7 * DAY;
        const MONTH: u64 = 4 * WEEK;

        let now = 1_000_000;
        let tge = now + 20 * DAY;
        let listing = tge + 10 * DAY;

        let schedule = ScheduleBuilder::with_tokens(1_000_000)
            .cliff(listing, Some(cliff1))
            .cliff(listing + 6 * MONTH, Some(cliff2))
            .offseted_by(6 * MONTH, LinearVesting::without_start(2 * MONTH, 6), None)
            .and_then(|x| x.build());
        assert!(schedule.is_ok());
        let schedule = schedule.unwrap();

        assert_eq!(schedule.available(u64::MIN), 0);

        assert_eq!(schedule.available(listing - 1 * MONTH), 0);
        assert_eq!(schedule.available(listing - 1 * DAY), 0);
        assert_eq!(schedule.available(listing - 1), 0);

        assert_eq!(schedule.available(listing), cliff1);
        assert_eq!(schedule.available(listing + 1), cliff1);
        assert_eq!(schedule.available(listing + 1 * WEEK), cliff1);
        assert_eq!(schedule.available(listing + 1 * WEEK + 1 * DAY), cliff1);

        assert_eq!(schedule.available(listing + 6 * MONTH), cliff1 + cliff2);
        assert_eq!(
            schedule.available(listing + 6 * MONTH + 1 * DAY),
            cliff1 + cliff2
        );
        assert_eq!(
            schedule.available(listing + 6 * MONTH + 1 * WEEK),
            cliff1 + cliff2
        );

        assert_eq!(
            schedule.available(listing + 6 * MONTH + 6 * MONTH),
            cliff1 + cliff2 + (total_tokens - cliff1 - cliff2) / 6
        );
        assert_eq!(
            schedule.available(listing + 6 * MONTH + 6 * MONTH + 1 * MONTH),
            cliff1 + cliff2 + (total_tokens - cliff1 - cliff2) / 6
        );

        assert_eq!(
            schedule.available(listing + 6 * MONTH + 6 * MONTH + 2 * MONTH),
            cliff1 + cliff2 + (total_tokens - cliff1 - cliff2) * 2 / 6
        );
        assert_eq!(
            schedule.available(listing + 6 * MONTH + 6 * MONTH + 4 * MONTH),
            cliff1 + cliff2 + (total_tokens - cliff1 - cliff2) * 3 / 6
        );
        assert_eq!(
            schedule.available(listing + 6 * MONTH + 6 * MONTH + 6 * MONTH),
            cliff1 + cliff2 + (total_tokens - cliff1 - cliff2) * 4 / 6
        );
        assert_eq!(
            schedule.available(listing + 6 * MONTH + 6 * MONTH + 8 * MONTH),
            cliff1 + cliff2 + (total_tokens - cliff1 - cliff2) * 5 / 6
        );
        assert_eq!(
            schedule.available(listing + 6 * MONTH + 6 * MONTH + 10 * MONTH),
            total_tokens
        );

        assert_eq!(schedule.available(u64::MAX), total_tokens);
    }

    fn construct_test_data() -> (VestingAccount, VestingSchedule) {
        let total_tokens = 1_000_000;
        let vesting = VestingAccount {
            total_tokens,
            withdrawn_tokens: 100_000,
            ..Default::default()
        };

        let schedule = ScheduleBuilder::with_tokens(total_tokens)
            .cliff(1_000_000, Some(200_000))
            .cliff(1_100_000, Some(200_000))
            .add(LinearVesting::new(1_400_000, 400_000, 3), None)
            .ending_at(2_000_000)
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(schedule.vestings[0].0, 200_000);
        assert_eq!(schedule.vestings[1].0, 200_000);
        assert_eq!(schedule.vestings[2].0, 400_000);
        assert_eq!(schedule.vestings[2].1.unlock_count, 2);
        assert_eq!(schedule.vestings[3].0, 200_000);
        assert!(schedule.last() == 2_000_000);
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
