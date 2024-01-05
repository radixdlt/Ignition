//! A module implementing the logic for the selection of the bins to contribute
//! liquidity to based on the current active bin, the bin span, and the maximum
//! number of allowed bins.

/// Ticks are in the range [0, 54000].
const MAXIMUM_TICK_VALUE: usize = 54000;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SelectedBins {
    pub active_bin: u32,
    pub lower_bins: Vec<u32>,
    pub higher_bins: Vec<u32>,
}

/// Selects the bins that the positions should be made out of.
///
/// Given the pool's active bin, bin span, and the preferred number of bins we
/// wish to have, this function determines which bins the liquidity should go
/// to without determining the exact formation of the liquidity (e.g., flat or
/// triangle). This function is specifically suited to handle edge cases when
/// selecting the bin that could lead to failures, and handles such cases in a
/// correct graceful manner.
///
/// In simple cases where the active bin is in the middle of the bin range and
/// the bin span is small enough, the selection of the lower and higher bins is
/// simple enough: preferred number of bins (divided by 2) to the left and the
/// same on the right. This gives the caller the number of bins that they wish
/// to have on the left and the right of the active bin.
///
/// There are cases however where the number of lower bins can't be equal to the
/// number of higher bins. Specifically, cases when the active bin's value is
/// too small or too large or cases when the bin span is too large. In such
/// cases, this function attempts to compensate the other side. As an example,
/// if we wish to have 10 bins and can only have 2 lower bins, then the higher
/// bins will have 8, thus achieving the preferred number of lower and higher
/// bins specified by the caller.
///
/// There are cases when the proffered number of bins can not be achieved by the
/// function, specifically, cases when the bin span is too large that any bin to
/// the right or the left will be outside of the range is allowed bins. In such
/// cases, this function returns a number of left and right bins that is less
/// than the desired.
///
/// # Examples
///
/// This section has examples with concrete numbers to to explain the behavior
/// of this function better.
///
/// ## Example 1: Simple Case
///
/// * `active_bin`: 100
/// * `bin_span`: 10
/// * `preferred_total_number_of_higher_and_lower_bins`: 4
///
/// This function will return the following:
///
/// * `active_bin`: 100
/// * `lower_bins`: [90, 80]
/// * `higher_bins`: [110, 120]
///
/// ## Example 2: Left Skew
///
/// * `active_bin`: 20
/// * `bin_span`: 10
/// * `preferred_total_number_of_higher_and_lower_bins`: 6
///
/// This function will return the following:
///
/// * `active_bin`: 20
/// * `lower_bins`: [10, 0]
/// * `higher_bins`: [30, 40, 50, 60]
///
/// At this currently active bin, there can only exist 2 bins on the lower side.
/// Thus, these bins are selected and left's remaining share of the bins is
/// given to the right. This results in a total of 6 bins.
///
/// ## Example 3: Right Skew
///
/// * `active_bin`: 53980
/// * `bin_span`: 10
/// * `preferred_total_number_of_higher_and_lower_bins`: 6
///
/// This function will return the following:
///
/// * `active_bin`: 53980
/// * `lower_bins`: [53970, 53960, 53950, 53940]
/// * `higher_bins`: [53990, 54000]
///
/// At this currently active bin, there can only exist 2 bins on the higher
/// side. Thus, these bins are selected and right's remaining share of the bins
/// is given to the left. This results in a total of 6 bins.
///
/// Example 4: Bin Size too large
///
/// * `active_bin`: 27000
/// * `bin_span`: 54000
/// * `preferred_total_number_of_higher_and_lower_bins`: 6
///
/// This function will return the following:
///
/// * `active_bin`: 27000
/// * `lower_bins`: []
/// * `higher_bins`: []
///
/// Given this pool's bin span, we can not get any bins that are lower or higher
/// and so we return just the active bin and return no lower or higher bins.
///
/// Example 4: Bin Size too large with a skew
///
/// * `active_bin`: 54000
/// * `bin_span`: 30000
/// * `preferred_total_number_of_higher_and_lower_bins`: 6
///
/// This function will return the following:
///
/// * `active_bin`: 54000
/// * `lower_bins`: [24000]
/// * `higher_bins`: []
///
/// # Arguments:
///
/// * `active_bin`: [`u32`] - The pool's currently active bin.
/// * `bin_span`: [`u32`] - The span between each bin and another or the
/// distance between them.
/// * `preferred_total_number_of_higher_and_lower_bins`: [`u32`] - The total
/// number of bins the caller wishes to have on the right and the left (summed).
/// As detailed above, this may or may not be achieved depending on the pool's
/// current bin and bin span.
///
/// # Returns:
///
/// [`SelectedBins`] - A struct with the bins that have been selected by this
/// function.
pub fn select_bins(
    active_bin: u32,
    bin_span: u32,
    preferred_total_number_of_higher_and_lower_bins: u32,
) -> SelectedBins {
    let mut selected_bins = SelectedBins {
        active_bin,
        higher_bins: vec![],
        lower_bins: vec![],
    };

    let mut remaining = preferred_total_number_of_higher_and_lower_bins;

    let mut forward_counter = BoundedU32::<0, MAXIMUM_TICK_VALUE>(active_bin);
    let mut backward_counter = BoundedU32::<0, MAXIMUM_TICK_VALUE>(active_bin);

    while remaining > 0 {
        let mut forward_counter_incremented = false;
        let mut backward_counter_decremented = false;

        if forward_counter.checked_add_assign(bin_span).is_some() {
            remaining -= 1;
            selected_bins.higher_bins.push(forward_counter.0);
            forward_counter_incremented = true;
        }
        if remaining > 0
            && backward_counter.checked_sub_assign(bin_span).is_some()
        {
            remaining -= 1;
            selected_bins.lower_bins.push(backward_counter.0);
            backward_counter_decremented = true;
        }

        if !forward_counter_incremented && !backward_counter_decremented {
            break;
        }
    }

    selected_bins
}

struct BoundedU32<const MIN: usize, const MAX: usize>(u32);

impl<const MIN: usize, const MAX: usize> BoundedU32<MIN, MAX> {
    pub fn checked_add_assign(&mut self, other: impl Into<u32>) -> Option<()> {
        if let Some(value) = self.checked_add(other) {
            *self = value;
            Some(())
        } else {
            None
        }
    }

    pub fn checked_sub_assign(&mut self, other: impl Into<u32>) -> Option<()> {
        if let Some(value) = self.checked_sub(other) {
            *self = value;
            Some(())
        } else {
            None
        }
    }

    pub fn checked_add(&self, other: impl Into<u32>) -> Option<Self> {
        let this = self.0;
        let other = other.into();

        if let Some(result) = this.checked_add(other) {
            if result as usize > MAX {
                None
            } else {
                Some(Self(result))
            }
        } else {
            None
        }
    }

    pub fn checked_sub(&self, other: impl Into<u32>) -> Option<Self> {
        let this = self.0;
        let other = other.into();

        if let Some(result) = this.checked_sub(other) {
            if (result as usize).lt(&MIN) {
                None
            } else {
                Some(Self(result))
            }
        } else {
            None
        }
    }
}
