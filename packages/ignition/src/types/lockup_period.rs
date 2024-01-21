use radix_engine_derive::*;
use scrypto::prelude::*;
use std::ops::*;

use humantime::format_duration;

/// A type used for the lockup period that can be creates from various time
/// durations and that implements display in the desired way.
#[derive(Clone, Copy, ScryptoSbor, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[sbor(transparent)]
pub struct LockupPeriod(u64);

impl LockupPeriod {
    pub const fn from_seconds(seconds: u64) -> Self {
        Self(seconds)
    }

    pub const fn from_minutes(minutes: u64) -> Self {
        Self::from_seconds(minutes * 60)
    }

    pub const fn from_hours(hours: u64) -> Self {
        Self::from_minutes(hours * 60)
    }

    pub const fn from_days(days: u64) -> Self {
        Self::from_hours(days * 24)
    }

    pub const fn from_weeks(weeks: u64) -> Self {
        Self::from_days(weeks * 7)
    }

    // One month approx 30.44 days
    pub const fn from_months(months: u64) -> Self {
        Self::from_seconds(months * 2_630_016)
    }

    pub const fn seconds(&self) -> &u64 {
        &self.0
    }
}

impl Display for LockupPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&format_duration(std::time::Duration::new(self.0, 0)), f)
    }
}

impl Debug for LockupPeriod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} seconds", self.0)
    }
}

impl Deref for LockupPeriod {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
