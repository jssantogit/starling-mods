use starling_greedymeshing_core::{Rect, merge_plane, merge_plane_limited, expand_2x2_uv};

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
    assert_eq!(
        rects,
        vec![Rect {
            col: 0,
            row: 0,
            width: 16,
            height: 16,
            key: 7
        }]
    );
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

#[test]
fn small_rectangle_limit_keeps_minecraft_uvs_representable() {
    let faces = [Some(19u64); 256];
    let merged = merge_plane_limited(&faces, 2, 2);
    assert_eq!(merged.len(), 64);
    assert!(merged.iter().all(|q| q.width == 2 && q.height == 2));
    reconstruct(&faces, &merged);
}

#[test]
fn limited_sweep_can_handle_narrow_patterns() {
    let mut faces = [None; 256];
    for row in 0..16 {
        for col in 0..16 {
            faces[row * 16 + col] = Some((row / 3 + col / 5) as u64);
        }
    }
    let rects = merge_plane_limited(&faces, 2, 2);
    reconstruct(&faces, &rects);
}

#[test]
fn one_by_one_limit_preserves_each_unit_face() {
    let faces = [Some(0); 256];
    let rects = merge_plane_limited(&faces, 1, 1);
    assert_eq!(rects.len(), 256);
    reconstruct(&faces, &rects);
}


#[test]
fn builtin_sampler_can_repeat_two_tiles_without_extra_vertex_bits() {
    let corner = [[0, 0], [1, 0], [1, 1], [0, 1]];
    let original = [[0, 0], [32768, 0], [32768, 32768], [0, 32768]];
    let uv = expand_2x2_uv(&corner, &original, 2, 2).unwrap();
    assert_eq!(uv[0], [0, 0]);
    assert_eq!(uv[1], [65535, 0]); // 2.0 saturated at u16 max
    assert_eq!(uv[2], [65535, 65535]);
    assert_eq!(uv[3], [0, 65535]);
}

#[test]
fn mirrored_and_rotated_texture_orientation_is_kept() {
    let corner = [[0, 0], [1, 0], [1, 1], [0, 1]];
    let mirrored = [[32768, 0], [0, 0], [0, 32768], [32768, 32768]];
    let uv = expand_2x2_uv(&corner, &mirrored, 2, 2).unwrap();
    assert_eq!(uv, [[65535, 0], [0, 0], [0, 65535], [65535, 65535]]);
    let rotated = [[0, 0], [0, 32768], [32768, 32768], [32768, 0]];
    let uv = expand_2x2_uv(&corner, &rotated, 2, 2).unwrap();
    assert_eq!(uv, [[0, 0], [0, 65535], [65535, 65535], [65535, 0]]);
}

#[test]
fn noncanonical_uvs_do_not_merge() {
    let corner = [[0, 0], [1, 0], [1, 1], [0, 1]];
    let partial_sprite = [[0, 0], [16384, 0], [16384, 16384], [0, 16384]];
    assert!(expand_2x2_uv(&corner, &partial_sprite, 2, 2).is_none());
    let shifted = [[100, 0], [32868, 0], [32868, 32768], [100, 32768]];
    assert!(expand_2x2_uv(&corner, &shifted, 2, 2).is_none());
}

#[test]
fn invalid_corner_geometry_and_out_of_range_extents_are_rejected() {
    let duplicate = [[0, 0], [1, 0], [1, 0], [0, 1]];
    let uv = [[0, 0], [32768, 0], [32768, 32768], [0, 32768]];
    assert!(expand_2x2_uv(&duplicate, &uv, 2, 2).is_none());
    let corners = [[0, 0], [1, 0], [1, 1], [0, 1]];
    assert!(expand_2x2_uv(&corners, &uv, 3, 1).is_none());
    assert!(expand_2x2_uv(&corners, &uv, 0, 1).is_none());
}

#[test]
fn one_tile_uv_roundtrips_exactly() {
    let corner = [[0, 0], [1, 0], [1, 1], [0, 1]];
    let uv = [[32768, 0], [0, 0], [0, 32768], [32768, 32768]];
    assert_eq!(expand_2x2_uv(&corner, &uv, 1, 1), Some(uv));
}
