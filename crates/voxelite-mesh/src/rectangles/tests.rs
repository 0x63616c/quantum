use super::*;

#[test]
fn rectangles_are_width_first_row_major_and_preserve_face_keys() {
    let mut mask = [1, 1, 2, 0, 1, 1, 2, 2, 1, 0, 2, 2];
    let mut rectangles = Vec::new();
    Mesh::greedy_rectangles(
        &mut mask,
        [4, 3],
        0,
        |_| true,
        |at, size, face| {
            rectangles.push((at, size, face));
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(
        rectangles,
        vec![
            ([0, 0], [2, 2], 1),
            ([2, 0], [1, 3], 2),
            ([3, 1], [1, 2], 2),
            ([0, 2], [1, 1], 1),
        ]
    );
    assert_eq!(mask, [0; 12]);
}

#[test]
fn discontinuities_stay_unit_sized_even_when_neighbor_keys_match() {
    // A face key may carry both a palette slot and per-corner shading.
    let flat = (7, [2, 2, 2, 2]);
    let varying = (7, [2, 3, 2, 3]);
    let other_flat = (7, [3, 3, 3, 3]);
    let empty = (0, [0; 4]);
    let mut mask = [flat, flat, varying, varying, other_flat, other_flat];
    let mut rectangles = Vec::new();
    Mesh::greedy_rectangles(
        &mut mask,
        [2, 3],
        empty,
        |(_, ao)| ao.iter().all(|a| *a == ao[0]),
        |at, size, face| {
            rectangles.push((at, size, face));
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(
        rectangles,
        vec![
            ([0, 0], [2, 1], flat),
            ([0, 1], [1, 1], varying),
            ([1, 1], [1, 1], varying),
            ([0, 2], [2, 1], other_flat),
        ]
    );
    assert_eq!(mask, [empty; 6]);
}

#[test]
fn malformed_masks_fail_without_consumption_or_emission() {
    for size in [[0, 2], [2, 0], [1, 1], [2, 2], [u16::MAX; 2]] {
        let mut mask = [1, 2];
        let error = Mesh::greedy_rectangles(
            &mut mask,
            size,
            0,
            |_| true,
            |_, _, _| panic!("invalid mask must not emit"),
        );
        assert_eq!(
            error,
            Err(MeshError::Invalid(
                "face mask dimensions do not match".into()
            ))
        );
        assert_eq!(mask, [1, 2]);
    }
    for size in [[0, 0], [0, 10], [10, 0]] {
        Mesh::greedy_rectangles(
            &mut [],
            size,
            0,
            |_| true,
            |_, _, _| panic!("empty mask must not emit"),
        )
        .unwrap();
    }
}

#[test]
fn emission_error_stops_before_consuming_the_next_rectangle() {
    let mut mask = [1, 1, 2, 2, 3, 3];
    let mut emitted = Vec::new();
    let failure = MeshError::Limit("test budget".into());
    let result = Mesh::greedy_rectangles(
        &mut mask,
        [6, 1],
        0,
        |_| true,
        |_, _, face| {
            emitted.push(face);
            if face == 2 {
                Err(failure.clone())
            } else {
                Ok(())
            }
        },
    );
    assert_eq!(result, Err(failure));
    assert_eq!(emitted, [1, 2]);
    assert_eq!(mask, [0, 0, 0, 0, 3, 3]);
}

#[test]
fn maximum_extent_is_preserved_without_truncation() {
    let mut mask = vec![256u16; usize::from(u16::MAX)];
    let mut emitted = Vec::new();
    Mesh::greedy_rectangles(
        &mut mask,
        [u16::MAX, 1],
        0,
        |_| true,
        |at, size, slot| {
            emitted.push((at, size, slot));
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(emitted, [([0, 0], [u16::MAX, 1], 256)]);
}
