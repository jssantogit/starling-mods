// Conservative native near-chunk greedy meshing pass for Starling 26.3.
// Only *upward* exposed faces of full opaque cubes are eligible in this alpha.
// It merges up to 2x2 faces so the existing 16-bit near-section UV format
// and the built-in WGSL tiling sampler remain usable. Nothing uses global state.

use super::{Mesher, MeshScratch, OPAQUE_LAYER, SectionMesh, TerrainVertex, pi};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Signature {
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

/// Project any quad onto a horizontal full-integer plane and conservatively
/// mark all grid cells it covers, including overlays/cutouts/translucency.
/// This makes us skip cells with two or more drawn surfaces.
fn coverage(q: &[TerrainVertex; 4]) -> Option<(i32, usize, usize, usize, usize)> {
    let y = q[0].pos[1];
    if !y.is_finite() || q.iter().any(|v| v.pos[1] != y) {
        return None;
    }
    let plane = y as i32;
    if y != plane as f32 || !(1..=16).contains(&plane) {
        return None;
    }
    let min_x = q.iter().map(|v| v.pos[0]).fold(f32::INFINITY, f32::min);
    let max_x = q.iter().map(|v| v.pos[0]).fold(f32::NEG_INFINITY, f32::max);
    let min_z = q.iter().map(|v| v.pos[2]).fold(f32::INFINITY, f32::min);
    let max_z = q.iter().map(|v| v.pos[2]).fold(f32::NEG_INFINITY, f32::max);
    if ![min_x, max_x, min_z, max_z].iter().all(|v| v.is_finite())
        || min_x < 0.0
        || min_z < 0.0
        || max_x > 16.0
        || max_z > 16.0
        || max_x <= min_x
        || max_z <= min_z
    {
        return None;
    }
    let (x0, x1) = (min_x.floor() as usize, max_x.ceil() as usize);
    let (z0, z1) = (min_z.floor() as usize, max_z.ceil() as usize);
    (x0 < x1 && z0 < z1).then_some((plane, x0, x1, z0, z1))
}

/// Accept only the unmodified, fully opaque, evenly lit, canonical top face
/// of a section-local full cube, avoiding translucent, connected, tinted,
/// emissive, rotated or offset model content.
fn candidate(
    q: &[TerrainVertex; 4],
    plane: i32,
    x: usize,
    z: usize,
    mesher: &Mesher<'_>,
    scratch: &MeshScratch,
) -> Option<Signature> {
    let first = q[0];
    if first.layer & OPAQUE_LAYER == 0
        || q.iter().any(|v| v.layer != first.layer || v.color != first.color || v.light != first.light)
    {
        return None;
    }
    // Both triangles must face +Y. A reversed face would disappear with
    // backface culling if combined with normal top-facing geometry.
    let a = [q[1].pos[0] - q[0].pos[0], q[1].pos[2] - q[0].pos[2]];
    let b = [q[2].pos[0] - q[0].pos[0], q[2].pos[2] - q[0].pos[2]];
    if a[1] * b[0] - a[0] * b[1] <= 0.0 {
        return None;
    }
    let state_id = *scratch.padded.blocks.get(pi(x as i32, plane - 1, z as i32))?;
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
    let mut corners = [[0u8; 2]; 4];
    for (i, vertex) in q.iter().enumerate() {
        let (sx, sz) = (vertex.pos[0] - x as f32, vertex.pos[2] - z as f32);
        if (sx != 0.0 && sx != 1.0) || (sz != 0.0 && sz != 1.0) {
            return None;
        }
        corners[i] = [sx as u8, sz as u8];
    }
    let uv = q.map(|v| v.uv);
    super::greedymeshing_core::expand_2x2_uv(&corners, &uv, 1, 1)?;
    Some(Signature {
        layer: first.layer,
        color: first.color,
        light: first.light,
        corners,
        uv,
    })
}

fn occupy(
    plane: &mut Plane,
    (x0, x1, z0, z1): (usize, usize, usize, usize),
    eligible: Option<usize>,
) {
    for z in z0..z1 {
        for x in x0..x1 {
            let cell = &mut plane.cells[z * 16 + x];
            cell.count = cell.count.saturating_add(1);
            cell.source = if cell.count == 1 { eligible } else { None };
        }
    }
}

/// Called once after the normal mesher finishes, and before the caller sorts
/// solid quads into orientation buckets. This operates only on the mesh's
/// opaque solid list and leaves the original faces unchanged on every fallback.
pub(super) fn compact(mesh: &mut SectionMesh, mesher: &Mesher<'_>, scratch: &MeshScratch) {
    if mesh.solid.len() < 8
        || mesher.shader_pack_lighting
        || mesher.separate_ao
        || !mesher.translucent_layers.is_empty()
        || !mesher.render.connected.is_empty()
    {
        return;
    }
    let mut planes: BTreeMap<i32, Plane> = BTreeMap::new();
    let mut signatures: HashMap<usize, Signature> = HashMap::new();
    let solid = mesh.solid.as_chunks::<4>().0;

    for (index, q) in solid.iter().enumerate() {
        let Some((plane, x0, x1, z0, z1)) = coverage(q) else {
            continue;
        };
        let sig = if x1 - x0 == 1 && z1 - z0 == 1 {
            candidate(q, plane, x0, z0, mesher, scratch)
        } else {
            None
        };
        if let Some(sig) = sig {
            signatures.insert(index, sig);
        }
        occupy(
            planes.entry(plane).or_default(),
            (x0, x1, z0, z1),
            sig.map(|_| index),
        );
    }
    // A translucent or cutout overlay at the same integer plane cannot be
    // swallowed or reordered by the merging of its opaque base.
    for q in mesh.translucent.as_chunks::<4>().0 {
        if let Some((plane, x0, x1, z0, z1)) = coverage(q) {
            occupy(planes.entry(plane).or_default(), (x0, x1, z0, z1), None);
        }
    }

    let mut dropped = vec![false; solid.len()];
    let mut replacement: HashMap<usize, [TerrainVertex; 4]> = HashMap::new();
    for (plane_y, plane) in planes {
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
        for rect in super::greedymeshing_core::merge_plane_limited(&keys, 2, 2) {
            if rect.width == 1 && rect.height == 1 {
                continue;
            }
            let start = rect.row as usize * 16 + rect.col as usize;
            let Some(base_index) = plane.cells[start].source else {
                continue;
            };
            let Some(sig) = signatures.get(&base_index) else {
                continue;
            };
            let Some(uv) = super::greedymeshing_core::expand_2x2_uv(
                &sig.corners,
                &sig.uv,
                rect.width,
                rect.height,
            ) else {
                continue;
            };
            let mut quad = solid[base_index];
            for i in 0..4 {
                quad[i].pos = [
                    rect.col as f32 + f32::from(sig.corners[i][0]) * f32::from(rect.width),
                    plane_y as f32,
                    rect.row as f32 + f32::from(sig.corners[i][1]) * f32::from(rect.height),
                ];
                quad[i].uv = uv[i];
            }
            replacement.insert(base_index, quad);
            for z in rect.row..rect.row + rect.height {
                for x in rect.col..rect.col + rect.width {
                    let index = usize::from(z) * 16 + usize::from(x);
                    if let Some(source) = plane.cells[index].source {
                        if source != base_index {
                            dropped[source] = true;
                        }
                    }
                }
            }
        }
    }
    if replacement.is_empty() {
        return;
    }
    let mut merged = Vec::with_capacity(mesh.solid.len());
    for (index, quad) in solid.iter().enumerate() {
        if !dropped[index] {
            merged.extend_from_slice(replacement.get(&index).unwrap_or(quad));
        }
    }
    mesh.solid = merged;
}
