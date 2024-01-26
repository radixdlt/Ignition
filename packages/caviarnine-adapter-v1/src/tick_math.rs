use scrypto::prelude::*;

pub const BASE: Decimal = dec!(1.0005);
pub const MIN_TICK: u32 = 0;
pub const MAX_TICK: u32 = 54000;

/// Converts a tick to spot price through checked math.
pub fn tick_to_spot(tick: u32) -> Option<Decimal> {
    if tick > MAX_TICK {
        None
    } else {
        BASE.checked_powi((tick as i64).checked_sub(27000)?.checked_mul(2)?)
    }
}
