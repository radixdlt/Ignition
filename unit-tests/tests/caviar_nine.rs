use caviarnine_adapter::*;

#[test]
fn selection_of_bins_simple_case_behaves_as_expected() {
    assert_eq!(
        SelectedBins::select(100, 10, 4),
        SelectedBins {
            active_bin: 100,
            lower_bins: vec![90, 80],
            higher_bins: vec![110, 120]
        }
    )
}

#[test]
fn selection_of_bins_left_skew_is_compensated_for_on_the_right() {
    assert_eq!(
        SelectedBins::select(20, 10, 6),
        SelectedBins {
            active_bin: 20,
            lower_bins: vec![10, 0],
            higher_bins: vec![30, 40, 50, 60]
        }
    )
}

#[test]
fn selection_of_bins_right_skew_is_compensated_for_on_the_left() {
    assert_eq!(
        SelectedBins::select(53980, 10, 6),
        SelectedBins {
            active_bin: 53980,
            lower_bins: vec![53970, 53960, 53950, 53940],
            higher_bins: vec![53990, 54000]
        }
    )
}

#[test]
fn selection_of_bins_bin_size_too_large_cant_fulfill_desired_bin_count() {
    assert_eq!(
        SelectedBins::select(27000, 54000, 6),
        SelectedBins {
            active_bin: 27000,
            lower_bins: vec![],
            higher_bins: vec![]
        }
    )
}

#[test]
fn selection_of_bins_bin_size_too_large_can_select_some_bins_if_bin_span_allows(
) {
    assert_eq!(
        SelectedBins::select(54000, 30000, 6),
        SelectedBins {
            active_bin: 54000,
            lower_bins: vec![24000],
            higher_bins: vec![]
        }
    )
}
