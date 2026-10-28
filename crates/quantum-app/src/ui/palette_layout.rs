//! Pure display-position math; palette mutations and stable identities stay in the domain.

pub(super) const COLUMNS: usize = 16;
pub(super) const CELL: f32 = quantum_ui::ControlHeight::Default.px();
pub(super) const GAP: f32 = quantum_ui::Space::S2.px();

/// Arrow traversal clamps at the first/last swatch, including partial final rows.
pub(super) fn navigate(index: usize, count: usize, columns: usize, key: &str) -> usize {
    let last = count.saturating_sub(1);
    let index = index.min(last);
    match key {
        "left" => index.saturating_sub(1),
        "right" => (index + 1).min(last),
        "up" => index.saturating_sub(columns.max(1)),
        "down" => (index + columns.max(1)).min(last),
        "home" => 0,
        "end" => last,
        _ => index,
    }
}

/// Insertion boundary nearest the pointer, in a wrapping grid including gaps.
pub(super) fn insertion(x: f32, y: f32, count: usize, columns: usize) -> usize {
    let columns = columns.max(1);
    let pitch = CELL + GAP;
    let row = (y.max(0.) / pitch).floor() as usize;
    let col = ((x.max(0.) + GAP / 2.) / pitch).round() as usize;
    row.saturating_mul(columns)
        .saturating_add(col.min(columns))
        .min(count)
}

/// Convert a boundary in the original order to MoveSwatch's final position.
pub(super) fn drop_position(from: usize, boundary: usize, count: usize) -> usize {
    boundary
        .saturating_sub(usize::from(boundary > from))
        .min(count.saturating_sub(1))
}

/// Sample a full palette evenly into at most sixteen thumbnail cells.
pub(super) fn thumbnail_indices(count: usize) -> Vec<usize> {
    let cells = count.min(16);
    (0..cells)
        .map(|i| {
            if cells == 1 {
                0
            } else {
                i * (count - 1) / (cells - 1)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn navigation_includes_partial_rows_and_empty_input() {
        assert_eq!(navigate(0, 19, 16, "left"), 0);
        assert_eq!(navigate(2, 19, 16, "up"), 0);
        assert_eq!(navigate(12, 19, 16, "down"), 18);
        assert_eq!(navigate(18, 19, 16, "up"), 2);
        assert_eq!(navigate(18, 19, 16, "right"), 18);
        assert_eq!(navigate(99, 0, 0, "down"), 0);
    }
    #[test]
    fn boundaries_include_halves_gaps_rows_and_append() {
        assert_eq!(insertion(-20., -10., 33, 16), 0);
        assert_eq!(insertion(4., 2., 33, 16), 0);
        assert_eq!(insertion(CELL / 2. - 0.01, 2., 33, 16), 0);
        assert_eq!(insertion(CELL / 2., 2., 33, 16), 1);
        assert_eq!(insertion(31., 2., 33, 16), 1);
        assert_eq!(insertion(34., 2., 33, 16), 1);
        assert_eq!(insertion(0., 36., 33, 16), 16);
        assert_eq!(insertion(999., 0., 33, 16), 16);
        assert_eq!(insertion(999., 999., 33, 16), 33);
        assert_eq!(insertion(0., 0., 0, 0), 0);
    }
    #[test]
    fn dropping_before_or_after_self_is_noop() {
        for count in 1..=256 {
            for from in 0..count {
                assert_eq!(drop_position(from, from, count), from);
                assert_eq!(drop_position(from, from + 1, count), from);
                assert_eq!(drop_position(from, count, count), count - 1);
                assert_eq!(drop_position(from, 0, count), 0);
            }
        }
        assert_eq!(drop_position(2, 6, 8), 5);
        assert_eq!(drop_position(6, 2, 8), 2);
    }
    #[test]
    fn every_drop_boundary_preserves_other_relative_positions() {
        for count in 1..=32 {
            for from in 0..count {
                for boundary in 0..=count {
                    let mut order: Vec<_> = (0..count).collect();
                    let value = order.remove(from);
                    let destination = drop_position(from, boundary, count);
                    order.insert(destination, value);
                    assert_eq!(order[destination], from);
                    assert!(
                        order
                            .iter()
                            .copied()
                            .filter(|i| *i != from)
                            .eq((0..count).filter(|i| *i != from))
                    );
                    if boundary <= from {
                        assert_eq!(destination, boundary);
                    } else {
                        assert_eq!(destination + 1, boundary);
                    }
                }
            }
        }
    }
    #[test]
    fn thumbnails_sample_both_ends_without_duplicates() {
        assert!(thumbnail_indices(0).is_empty());
        assert_eq!(thumbnail_indices(1), [0]);
        for count in 2..=256 {
            let indices = thumbnail_indices(count);
            assert_eq!(indices.len(), count.min(16));
            assert_eq!(indices.first(), Some(&0));
            assert_eq!(indices.last(), Some(&(count - 1)));
            assert!(indices.windows(2).all(|pair| pair[0] < pair[1]));
        }
    }
}
