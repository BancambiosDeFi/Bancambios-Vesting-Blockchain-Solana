use crate::state::{LinearVesting, VestingSchedule, WithoutStart};

#[derive(Debug, PartialEq)]
pub struct ScheduleBuilder {
    token_count: u64,
    used_tokens: u64,
    vestings: Vec<(u64, LinearVesting)>,
}

#[derive(Debug, PartialEq)]
pub enum ScheduleBuilderError {
    /// Builder has no vestings added
    EmptyBuilder,

    /// The total tokens in vesting scheule and used tokens differ. The tuple of
    /// (expected tokens, used tokens) is returned.
    InvalidTokenAmountUsed((u64, u64)),

    /// Vestings were added not sequentially.
    VestingsNotSorted,

    /// All vestings should be associated with non-zero amount of unlockable tokens.
    ZeroTokens,

    /// Maximum of `MAX_VESTINGS` is allowed
    TooManyVestings,

    /// Initial unlock value is bigger than provided tokens
    InitialUnlockTooBig,

    /// Start time is bigger than end time
    InvalidTimeInterval,
}

impl ScheduleBuilder {
    pub fn with_tokens(token_count: u64) -> ScheduleBuilder {
        ScheduleBuilder {
            token_count,
            used_tokens: 0,
            vestings: Vec::new(),
        }
    }

    fn use_tokens(&mut self, tokens: u64) {
        self.used_tokens += tokens;
    }

    fn available_tokens(&self) -> u64 {
        if self.used_tokens >= self.token_count {
            0
        } else {
            self.token_count - self.used_tokens
        }
    }

    fn remove_last(&mut self) -> Option<(u64, LinearVesting)> {
        let last = self.vestings.pop();
        last.map(|x| self.used_tokens -= x.0);
        last
    }

    pub fn add(mut self, vesting: LinearVesting, tokens: Option<u64>) -> ScheduleBuilder {
        let tokens = tokens.unwrap_or(self.available_tokens());
        self.use_tokens(tokens);

        self.vestings.push((tokens, vesting));
        self
    }

    pub fn cliff(self, time: u64, tokens: Option<u64>) -> ScheduleBuilder {
        self.add(LinearVesting::cliff(time), tokens)
    }

    pub fn offseted_by(
        self,
        offset: u64,
        vesting: LinearVesting<WithoutStart>,
        tokens: Option<u64>,
    ) -> Result<ScheduleBuilder, ScheduleBuilderError> {
        match self.vestings.last() {
            None => Err(ScheduleBuilderError::EmptyBuilder),
            Some(&x) => Ok(self.add(
                LinearVesting::new(
                    x.1.last() + offset,
                    vesting.unlock_period(),
                    vesting.unlock_count(),
                ),
                tokens,
            )),
        }
    }

    pub fn offseted(
        self,
        vesting: LinearVesting<WithoutStart>,
        tokens: Option<u64>,
    ) -> Result<ScheduleBuilder, ScheduleBuilderError> {
        self.offseted_by(vesting.unlock_period(), vesting, tokens)
    }

    pub fn ending_at(mut self, end_time: u64) -> Result<ScheduleBuilder, ScheduleBuilderError> {
        if self.vestings.len() == 0 {
            return Err(ScheduleBuilderError::EmptyBuilder);
        }
        let last_vesting = self.vestings.last().unwrap();
        if end_time >= last_vesting.1.last() {
            Ok(self)
        } else {
            let last_vesting = self.remove_last().unwrap();
            let new_unlock_count = 1
                + ((end_time - last_vesting.1.start_time()) / last_vesting.1.unlock_period()) as u8;
            assert!(new_unlock_count < last_vesting.1.unlock_count());

            let linear_tokens =
                last_vesting.0 * new_unlock_count as u64 / last_vesting.1.unlock_count() as u64;
            let cliff_tokens = last_vesting.0 - linear_tokens;

            Ok(self
                .add(
                    LinearVesting::new(
                        last_vesting.1.start_time(),
                        last_vesting.1.unlock_period(),
                        new_unlock_count,
                    ),
                    Some(linear_tokens),
                )
                .cliff(end_time, Some(cliff_tokens)))
        }
    }

    pub fn legacy(
        self,
        start_time: u64,
        end_time: u64,
        unlock_period: u64,
        cliff: u64,
        initial_unlock_tokens: u64,
        tokens: Option<u64>,
    ) -> Result<ScheduleBuilder, ScheduleBuilderError> {
        if start_time >= end_time {
            return Err(ScheduleBuilderError::InvalidTimeInterval);
        }
        let tokens = tokens.unwrap_or(self.available_tokens());
        if initial_unlock_tokens >= tokens {
            return Err(ScheduleBuilderError::InitialUnlockTooBig);
        }
        if cliff < start_time && cliff > end_time {
            return Err(ScheduleBuilderError::InvalidTimeInterval);
        }

        let mut builder = if initial_unlock_tokens > 0 {
            self.cliff(start_time, Some(initial_unlock_tokens))
        } else {
            self
        };
        let mut remaining_tokens = tokens - initial_unlock_tokens;

        let mut total_linear_unlocks: u8 = 1 + ((end_time - start_time) / unlock_period) as u8;
        if (end_time - start_time) % unlock_period != 0 {
            total_linear_unlocks += 1;
        }

        let unlocks_before_cliff: u8 = 1 + ((cliff - start_time) / unlock_period) as u8;
        if unlocks_before_cliff > 0 {
            let tokens_at_cliff =
                remaining_tokens * unlocks_before_cliff as u64 / total_linear_unlocks as u64;
            remaining_tokens -= tokens_at_cliff;
            builder = builder.cliff(cliff, Some(tokens_at_cliff))
        }

        let first_linear_unlock = cliff + cliff % unlock_period;
        builder
            .add(
                LinearVesting::new(
                    first_linear_unlock,
                    unlock_period,
                    total_linear_unlocks - unlocks_before_cliff,
                ),
                Some(remaining_tokens),
            )
            .ending_at(end_time)
    }

    pub fn build(self) -> Result<VestingSchedule, ScheduleBuilderError> {
        if self.token_count != self.used_tokens {
            return Err(ScheduleBuilderError::InvalidTokenAmountUsed((
                self.token_count,
                self.used_tokens,
            )));
        }

        if self.vestings.len() > VestingSchedule::MAX_VESTINGS {
            return Err(ScheduleBuilderError::TooManyVestings);
        }

        // TODO: use slice::is_sorted_by when it becomes stable
        let mut is_sorted = true;
        for i in 1..self.vestings.len() {
            if self.vestings[i - 1].1.last() > self.vestings[i].1.start_time() {
                is_sorted = false;
                break;
            }
        }
        if !is_sorted {
            return Err(ScheduleBuilderError::VestingsNotSorted);
        }

        for i in self.vestings.iter() {
            if i.0 == 0 {
                return Err(ScheduleBuilderError::ZeroTokens);
            }
        }

        Ok(VestingSchedule::new(self.token_count, &self.vestings))
    }
}
