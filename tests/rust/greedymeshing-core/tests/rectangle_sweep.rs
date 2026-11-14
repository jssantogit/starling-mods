use starling_greedymeshing_core::{Rect, merge_plane};

fn reconstruct(original: &[Option<u64>; 256], rects: &[Rect]) {
    let mut actual = [None; 256];
    let mut occupied = [false; 256];
    for rect in rects {
        assert!(rect.width > 0 && rect.height > 0);
        assert!((rect.col as usize) + (rect.width as usize) <= 16);
        assert!((rect.row as usize) + (rect.height as usize) <= 16);
        for row in rect.row..rect.row + rect.height {
            for col in rect.col..rect.col + rect.width {
                let index = row as usize * 16 + col as usize;
                assert!(!occupied[index], "overlapping rectangles at {index}");
                occupied[index] = true;
                assert_eq!(original[index], Some(rect.key));
                actual[index] = Some(rect.key);
            }
        }
    }
    assert_eq!(&actual, original, "the merge lost or invented a face");
}

#[test]
fn empty_plane_emits_no_faces() {
    let faces = [None; 256];
    assert!(merge_plane(&faces).is_empty());
}

#[test]
fn one_uniform_plane_is_one_quad() {
    let faces = [Some(7); 256];
    let rects = merge_plane(&faces);
    assert_eq!(rects, vec![Rect { col: 0, row: 0, width: 16, height: 16, key: 7 }]);
    reconstruct(&faces, &rects);
}

#[test]
fn adjacent_faces_of_different_materials_do_not_merge() {
    let mut faces = [None; 256];
    for row in 0..16 {
        for col in 0..16 {
            faces[row * 16 + col] = Some(if col < 8 { 10 } else { 11 });
        }
    }
    let rects = merge_plane(&faces);
    assert_eq!(rects.len(), 2);
    reconstruct(&faces, &rects);
}

#[test]
fn ineligible_cells_and_holes_are_never_fabricated() {
    let mut faces = [Some(3); 256];
    faces[0] = None;
    faces[15] = None;
    faces[9 * 16 + 9] = None;
    let rects = merge_plane(&faces);
    reconstruct(&faces, &rects);
}

#[test]
fn a_zero_key_is_a_valid_material() {
    let faces = [Some(0); 256];
    let rects = merge_plane(&faces);
    assert_eq!(rects.len(), 1);
    reconstruct(&faces, &rects);
}

#[test]
fn does_not_drop_256_distinct_faces_on_a_single_plane() {
    let faces = std::array::from_fn(|index| Some(index as u64));
    let rects = merge_plane(&faces);
    assert_eq!(rects.len(), 256);
    reconstruct(&faces, &rects);
}

#[test]
fn checkerboard_does_not_merge_nonadjacent_same_materials() {
    let faces = std::array::from_fn(|index| {
        let row = index / 16;
        let col = index % 16;
        Some(((row + col) & 1) as u64)
    });
    let rects = merge_plane(&faces);
    assert_eq!(rects.len(), 256);
    reconstruct(&faces, &rects);
}

#[test]
fn seeded_property_sweep_never_loses_or_overlaps_faces() {
    let mut state = 0x1234_5678_9abc_def0u64;
    for iteration in 0..128 {
        let faces = std::array::from_fn(|_| {
            // Deterministic xorshift64; no external dependencies or randomness.
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            match state % 7 {
                0 | 1 => None,
                k => Some(k + (iteration % 3) as u64),
            }
        });
        reconstruct(&faces, &merge_plane(&faces));
    }
}
