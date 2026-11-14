// Native near-section Greedy Meshing adapter for the six face directions.
// This remains an experimental 2x2-only port, not a shader-pack-safe release.
// It runs after Starling's normal mesh construction, leaving ineligible
// geometry untouched and never extending beyond one 16x16 section.

use super::{Mesher, MeshScratch, OPAQUE_LAYER, SectionMesh, TerrainVertex, pi};
use std::collections::{BTreeMap, HashMap};
use super::greedymeshing_core::{self, FacePlane};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Signature {
    normal: i8,
    layer: u16,
    color: u32,
    light: u16,
    corners: [[u8; 2]; 4],
    uv: [[u16; 2]; 4],
}

#[derive(Clone, Copy, Default)]
struct Cell {
    count: u8,
    source: Option<usize>,
}

struct Plane {
    cells: [Cell; 256],
}

impl Default for Plane {
    fn default() -> Self {
        Self {
            cells: [Cell::default(); 256],
        }
    }
}

/// All axis-aligned planar quads, including ineligible quads, contribute
/// coverage. A solid quad and a translucent quad in the same cell cannot
/// cause a merge, even if their normals point in opposite directions.
fn coverage(q: &[TerrainVertex; 4]) -> Option<FacePlane> {
    greedymeshing_core::project_quad(&q.map(|v| v.pos))
}

/// A candidate must be a single complete quad from a full, opaque cube,
/// facing either sign of X/Y/Z, with uniform light and matching four-corner
/// UV coordinates. A rotated texture is fine if all cells have the same
/// orientation; noncanonical sprite sections are not.
fn candidate(
    q: &[TerrainVertex; 4],
    face: FacePlane,
    mesher: &Mesher<'_>,
    scratch: &MeshScratch,
) -> Option<Signature> {
    let first = q[0];
    if first.layer & OPAQUE_LAYER == 0
        || q.iter().any(|v| {
            v.layer != first.layer || v.color != first.color || v.light != first.light
        })
    {
        return None;
    }

    let block = face.block_coords()?;
    let state_id = *scratch.padded.blocks.get(pi(block[0], block[1], block[2]))?;
    let state = mesher.tables.info.get(state_id as usize)?;
    if !state.opaque_cube
        || state.block_entity
        || state.offset.is_some()
        || state.emission > 0
        || state.tint_count != 0
        || state.leaves
        || state.pane
        || !state.model
    {
        return None;
    }

    let corners = face.unit_corners(&q.map(|v| v.pos))?;
    let uv = q.map(|v| v.uv);
    greedymeshing_core::expand_2x2_uv(&corners, &uv, 1, 1)?;
    Some(Signature {
        normal: face.sign,
        layer: first.layer,
        color: first.color,
        light: first.light,
        corners,
        uv,
    })
}

/// Occupancy is shared for *both* normals on one physical plane. This rejects
/// double-sided blocks, decals and coplanar overlays instead of swallowing
/// or reordering them in the replacement geometry.
fn occupy(plane: &mut Plane, face: FacePlane, eligible: Option<usize>) {
    for v in face.v0..face.v1 {
        for u in face.u0..face.u1 {
            let cell = &mut plane.cells[usize::from(v) * 16 + usize::from(u)];
            cell.count = cell.count.saturating_add(1);
            cell.source = if cell.count == 1 { eligible } else { None };
        }
    }
}

/// Native post-mesh compaction. Starling still performs its own per-section
/// visibility, block-light bookkeeping, sprite list and orientation sorting.
pub(super) fn compact(mesh: &mut SectionMesh, mesher: &Mesher<'_>, scratch: &MeshScratch) {
    if mesh.solid.len() < 8
        || mesher.shader_pack_lighting
        || mesher.separate_ao
        || !mesher.translucent_layers.is_empty()
        || !mesher.render.connected.is_empty()
    {
        return;
    }

    let solid = mesh.solid.as_chunks::<4>().0;
    let mut planes: BTreeMap<(u8, i32), Plane> = BTreeMap::new();
    let mut signatures: HashMap<usize, Signature> = HashMap::new();

    for (index, quad) in solid.iter().enumerate() {
        let Some(face) = coverage(quad) else {
            continue;
        };
        let sig = candidate(quad, face, mesher, scratch);
        if let Some(sig) = sig {
            signatures.insert(index, sig);
        }
        occupy(
            planes.entry((face.axis, face.plane)).or_default(),
            face,
            sig.map(|_| index),
        );
    }
    // Non-solid material at the exact same plane always prevents a merge.
    for quad in mesh.translucent.as_chunks::<4>().0 {
        if let Some(face) = coverage(quad) {
            occupy(planes.entry((face.axis, face.plane)).or_default(), face, None);
        }
    }

    let mut dropped = vec![false; solid.len()];
    let mut replacements = HashMap::<usize, [TerrainVertex; 4]>::new();

    for ((axis, plane_at), plane) in planes {
        let mut keys = [None; 256];
        let mut intern = HashMap::<Signature, u64>::new();
        for (cell_index, cell) in plane.cells.iter().enumerate() {
            if cell.count != 1 {
                continue;
            }
            let Some(sig) = cell.source.and_then(|index| signatures.get(&index)) else {
                continue;
            };
            let id = if let Some(&id) = intern.get(sig) {
                id
            } else {
                let next = intern.len() as u64;
                intern.insert(*sig, next);
                next
            };
            keys[cell_index] = Some(id);
        }

        for rect in greedymeshing_core::merge_plane_limited(&keys, 2, 2) {
            if rect.width == 1 && rect.height == 1 {
                continue;
            }
            let first_cell = usize::from(rect.row) * 16 + usize::from(rect.col);
            let Some(first_index) = plane.cells[first_cell].source else {
                continue;
            };
            let Some(signature) = signatures.get(&first_index) else {
                continue;
            };
            let Some(uv) = greedymeshing_core::expand_2x2_uv(
                &signature.corners,
                &signature.uv,
                rect.width,
                rect.height,
            ) else {
                continue;
            };
            let mut quad = solid[first_index];
            let (u_axis, v_axis) = match axis {
                0 => (1, 2),
                1 => (0, 2),
                2 => (0, 1),
                _ => unreachable!(),
            };
            for i in 0..4 {
                quad[i].pos[usize::from(axis)] = plane_at as f32;
                quad[i].pos[u_axis] = f32::from(rect.col)
                    + f32::from(signature.corners[i][0]) * f32::from(rect.width);
                quad[i].pos[v_axis] = f32::from(rect.row)
                    + f32::from(signature.corners[i][1]) * f32::from(rect.height);
                quad[i].uv = uv[i];
            }
            replacements.insert(first_index, quad);
            for v in rect.row..rect.row + rect.height {
                for u in rect.col..rect.col + rect.width {
                    let cell = usize::from(v) * 16 + usize::from(u);
                    if let Some(other) = plane.cells[cell].source {
                        if other != first_index {
                            dropped[other] = true;
                        }
                    }
                }
            }
        }
    }
    if replacements.is_empty() {
        return;
    }
    let mut merged = Vec::with_capacity(mesh.solid.len());
    for (index, quad) in solid.iter().enumerate() {
        if !dropped[index] {
            merged.extend_from_slice(replacements.get(&index).unwrap_or(quad));
        }
    }
    mesh.solid = merged;
}
