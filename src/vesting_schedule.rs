use crate::types::SEC_IN_DAY;
use scrypto::prelude::*;

#[derive(ScryptoSbor)]
pub struct VestingSchedule {
    pub start_epoch: i64,
    pub cliff_epoch: Option<i64>,
    pub end_epoch: i64,
    pub vest_interval: i64,
    pub amount: Decimal,
    pub withdrawn: Decimal,
    pub cancel_epoch: Option<i64>,
    pub is_check_join: bool,
}

impl VestingSchedule {
    pub fn new(
        start_epoch: i64,
        cliff_epoch: Option<i64>,
        end_epoch: i64,
        vest_interval: i64,
        amount: Decimal,
        is_check_join: bool,
    ) -> Self {
        Self::check_schedule(&start_epoch, &cliff_epoch, &end_epoch, &vest_interval);

        Self {
            start_epoch,
            cliff_epoch,
            end_epoch,
            vest_interval,
            amount,
            withdrawn: dec!(0),
            cancel_epoch: None,
            is_check_join,
        }
    }

    pub fn get_vested(&self) -> Decimal {
        let curr_epoch = match self.cancel_epoch {
            Some(v) => v,
            None => Self::get_curr_epoch(),
        };
        let cutoff_epoch = match self.cliff_epoch {
            Some(c_epoch) => c_epoch,
            None => self.start_epoch,
        };
        if curr_epoch >= self.end_epoch {
            return self.amount;
        }
        if curr_epoch <= cutoff_epoch {
            return dec!("0");
        }

        // vest in intervals, eg. every 7 days
        let interval = self.vest_interval * SEC_IN_DAY;
        let elapsed_time = curr_epoch - self.start_epoch;
        let elapsed_intervals: i64 = elapsed_time / interval; // integer division round down

        // divide total amount by total time & multiply by interval
        let vest_time = self.end_epoch - self.start_epoch;
        let vest_per_interval: Decimal = (self.amount / vest_time) * interval; // divide before multiply

        let total_vested: Decimal = vest_per_interval * elapsed_intervals;
        total_vested
    }

    pub fn get_unvested(&self) -> Decimal {
        self.amount - self.get_vested()
    }

    pub fn check_join(&self) {
        if self.is_check_join {
            assert!(
                self.start_epoch >= Self::get_curr_epoch(),
                "[Check Join]: Past start date"
            )
        }
    }

    pub fn check_schedule(
        start_epoch: &i64,
        cliff_epoch: &Option<i64>,
        end_epoch: &i64,
        vest_interval: &i64,
    ) {
        match cliff_epoch {
            Some(c_epoch) => assert!(
                end_epoch >= c_epoch && c_epoch >= start_epoch && start_epoch > &0i64,
                "[Check Schedule]: Cliff must be after start and before end"
            ),
            None => assert!(
                end_epoch >= start_epoch && start_epoch > &0i64,
                "[Check Schedule]: End must be after start"
            ),
        };
        assert!(vest_interval > &0i64, "[Check Schedule]: No Interval");
    }

    pub fn get_curr_epoch() -> i64 {
        Clock::current_time(TimePrecision::Second).seconds_since_unix_epoch
    }
}
