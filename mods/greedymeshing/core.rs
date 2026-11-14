// Standalone 16x16 face-rectangle sweep, adapted from Greedy Meshing Mod
// (Sierra, MIT): https://github.com/programmer1o1/GreedyMeshingMod.
//
// Only geometry grouping is implemented. Starling's near terrain renderer
// is not yet wired to this code; material keys, lighting, atlas tiling, AO,
// CTM and shader-pack compatibility must be solved before emitting GPU quads.

/// A merged rectangle of *identically keyed* faces in one 16x16 plane.
///
/// One rectangle stands in for width * height original unit faces. Positions
/// are unsigned block offsets, never crossing a section or face-plane boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub col: u8,
    pub row: u8,
    pub width: u8,
    pub height: u8,
    pub key: u64,
}

/// Merge one plane of eligible faces into non-overlapping rectangles.
///
/// The caller must supply all relevant rendering constraints in the merge key:
/// sprite, orientation/UV, material, light/AO, tint and layer. Each None is
/// excluded from the greedy path and MUST remain on the normal rendering path
/// if it represents an ineligible visible face.
///
/// An unbounded list of keys is supported: never silently discard materials
/// when there are many distinct faces in one plane.
pub fn merge_plane(cells: &[Option<u64>; 256]) -> Vec<Rect> {
    merge_plane_limited(cells, 16, 16)
}

/// As `merge_plane`, but cap rectangles to an explicit maximum extent.
///
/// Starling's near-section packed 16-bit UVs can represent a range of almost
/// two texture tiles, unlike its distant-LOD UVs. Limiting to 2×2 lets a
/// cautious first integration reuse the existing WGSL tiled sampler without
/// changing the shader layout. Larger rectangles need a separate UV encoding.
/// The width/height arguments are inclusive limits, both in 1..=16.
pub fn merge_plane_limited(cells: &[Option<u64>; 256], max_width: u8, max_height: u8) -> Vec<Rect> {
    assert!((1..=16).contains(&max_width));
    assert!((1..=16).contains(&max_height));
    let max_width = usize::from(max_width);
    let max_height = usize::from(max_height);
    let mut rectangles = Vec::new();
    // Only 16 columns per row: a u16 marks cells consumed by earlier rectangles.
    let mut consumed = [0u16; 16];

    for row in 0..16usize {
        let mut col = 0usize;
        while col < 16 {
            let bit = 1u16 << col;
            let Some(key) = cells[row * 16 + col] else {
                col += 1;
                continue;
            };
            if consumed[row] & bit != 0 {
                col += 1;
                continue;
            }

            // Same-key contiguous run to the right.
            let mut width = 1usize;
            while col + width < 16
                && width < max_width
                && consumed[row] & (1u16 << (col + width)) == 0
                && cells[row * 16 + col + width] == Some(key)
            {
                width += 1;
            }

            // Extend the entire run down only while *every* face still matches.
            let mut height = 1usize;
            'grow: while row + height < 16 && height < max_height {
                for offset in 0..width {
                    let c = col + offset;
                    if consumed[row + height] & (1u16 << c) != 0
                        || cells[(row + height) * 16 + c] != Some(key)
                    {
                        break 'grow;
                    }
                }
                height += 1;
            }

            // Compute in u32: a full 16-wide mask must not shift 1u16 by 16.
            let mask = (((1u32 << width) - 1) << col) as u16;
            for bits in &mut consumed[row..row + height] {
                *bits |= mask;
            }
            rectangles.push(Rect {
                col: col as u8,
                row: row as u8,
                width: width as u8,
                height: height as u8,
                key,
            });
            col += width;
        }
    }

    rectangles
}

/// Expand canonical unit-quad UV coordinates for at most a 2x2 merged face.
///
/// The four corners are the source quad's winding order, each with a logical
/// (column, row) offset of (0 or 1, 0 or 1). A full vanilla sprite spans 0..32768
/// packed steps per axis in a nearby-section TerrainVertex. For two blocks the
/// nominal endpoint is 65536, so it is saturated to 65535 in u16. The existing
/// terrain.wgsl tiled sampler repeats this 1.99997 coordinate correctly.
///
/// Partial/shifted/scaled/rotated-in-noncanonical ways must fall back to vanilla.
pub fn expand_2x2_uv(
    corners: &[[u8; 2]; 4],
    packed_uv: &[[u16; 2]; 4],
    width: u8,
    height: u8,
) -> Option<[[u16; 2]; 4]> {
    if !(1..=2).contains(&width) || !(1..=2).contains(&height) {
        return None;
    }
    let find = |target: [u8; 2]| corners.iter().position(|&p| p == target);
    let (i00, i10, i01, i11) = (find([0, 0])?, find([1, 0])?, find([0, 1])?, find([1, 1])?);
    if corners.iter().any(|p| p[0] > 1 || p[1] > 1) {
        return None;
    }
    if [i00, i10, i01, i11]
        .iter()
        .any(|&i| [i00, i10, i01, i11].iter().filter(|&&j| j == i).count() != 1)
    {
        return None;
    }
    const STEP: i32 = 32768;
    let mut result = [[0u16; 2]; 4];
    let mut axis_col = [false; 2];
    let mut axis_row = [false; 2];

    for channel in 0..2usize {
        if packed_uv
            .iter()
            .any(|uv| uv[channel] != 0 && uv[channel] != 32768)
        {
            return None;
        }
        let initial = i32::from(packed_uv[i00][channel]);
        let col = i32::from(packed_uv[i10][channel]) - initial;
        let row = i32::from(packed_uv[i01][channel]) - initial;
        let opposite = i32::from(packed_uv[i11][channel]);
        if opposite != initial + col + row {
            return None;
        }
        if (col.abs(), row.abs()) != (STEP, 0) && (col.abs(), row.abs()) != (0, STEP) {
            return None;
        }
        axis_col[channel] = col != 0;
        axis_row[channel] = row != 0;
        let offset = if col < 0 {
            (i32::from(width) - 1) * STEP
        } else {
            0
        } + if row < 0 {
            (i32::from(height) - 1) * STEP
        } else {
            0
        };
        for i in 0..4 {
            let c = i32::from(corners[i][0]);
            let r = i32::from(corners[i][1]);
            let v = i32::from(packed_uv[i][channel])
                + col * (i32::from(width) - 1) * c
                + row * (i32::from(height) - 1) * r
                + offset;
            if !(0..=65536).contains(&v) {
                return None;
            }
            result[i][channel] = v.min(65535) as u16;
        }
    }
    // A 90-degree rotation or mirror is fine; a collapsed/sheared texture isn't.
    if axis_col[0] == axis_col[1] || axis_row[0] == axis_row[1] {
        return None;
    }
    Some(result)
}


/// Conservative planar projection of a quad in a 16×16×16 section.
/// Axis 0 is X (U=Y,V=Z), axis 1 is Y (U=X,V=Z), axis 2 is Z (U=X,V=Y).
/// Both face windings are retained; opposite faces must not share a signature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FacePlane {
    pub axis: u8,
    pub plane: i32,
    pub sign: i8,
    pub u0: u8,
    pub u1: u8,
    pub v0: u8,
    pub v1: u8,
}

impl FacePlane {
    pub fn axes(&self) -> (usize, usize) {
        match self.axis {
            0 => (1, 2),
            1 => (0, 2),
            2 => (0, 1),
            _ => unreachable!("projection is constructed with one of 3 axes"),
        }
    }

    /// Position of the full-cube block behind a unit face, or None at the
    /// wrong section boundary. Only safe to read within the local 16³ block.
    pub fn block_coords(&self) -> Option<[i32; 3]> {
        if self.u1 - self.u0 != 1 || self.v1 - self.v0 != 1 {
            return None;
        }
        let (u, v) = self.axes();
        let mut p = [0i32; 3];
        p[usize::from(self.axis)] = self.plane - i32::from(self.sign > 0);
        p[u] = i32::from(self.u0);
        p[v] = i32::from(self.v0);
        p.iter().all(|&value| (0..16).contains(&value)).then_some(p)
    }

    /// Quad corners in exact local {0,1}², rejecting nonrectangular faces and
    /// duplicate corners. Ineligible faces stay on Starling's original path.
    pub fn unit_corners(&self, positions: &[[f32; 3]; 4]) -> Option<[[u8; 2]; 4]> {
        if self.u1 - self.u0 != 1 || self.v1 - self.v0 != 1 {
            return None;
        }
        let (u, v) = self.axes();
        let mut seen = 0u8;
        let mut out = [[0u8; 2]; 4];
        for (index, p) in positions.iter().enumerate() {
            let du = p[u] - f32::from(self.u0);
            let dv = p[v] - f32::from(self.v0);
            if (du != 0.0 && du != 1.0) || (dv != 0.0 && dv != 1.0) {
                return None;
            }
            let (u0, v0) = (du as u8, dv as u8);
            let bit = 1u8 << (u0 + 2 * v0);
            if seen & bit != 0 {
                return None;
            }
            seen |= bit;
            out[index] = [u0, v0];
        }
        (seen == 0b1111).then_some(out)
    }
}

/// Compute a horizontal/vertical cell plane for **all six** axis-aligned face
/// directions. The bounding cells of a non-unit quad are still returned, so
/// overlays and complex geometry can block unsafe merges.
pub fn project_quad(positions: &[[f32; 3]; 4]) -> Option<FacePlane> {
    if positions.iter().flatten().any(|value| !value.is_finite()) {
        return None;
    }
    let mut axis = None;
    for n in 0..3 {
        if positions.iter().all(|p| p[n] == positions[0][n]) {
            if axis.is_some() {
                return None; // degenerate point or line
            }
            axis = Some(n);
        }
    }
    let axis = axis?;
    let at = positions[0][axis];
    if at < 0.0 || at > 16.0 || at.fract() != 0.0 {
        return None;
    }
    let (u, v) = match axis {
        0 => (1, 2),
        1 => (0, 2),
        2 => (0, 1),
        _ => unreachable!(),
    };
    let low_u = positions.iter().map(|p| p[u]).fold(f32::INFINITY, f32::min);
    let high_u = positions.iter().map(|p| p[u]).fold(f32::NEG_INFINITY, f32::max);
    let low_v = positions.iter().map(|p| p[v]).fold(f32::INFINITY, f32::min);
    let high_v = positions.iter().map(|p| p[v]).fold(f32::NEG_INFINITY, f32::max);
    if low_u < 0.0 || high_u > 16.0 || low_v < 0.0 || high_v > 16.0
        || low_u >= high_u || low_v >= high_v
    {
        return None;
    }
    let a = [
        positions[1][0] - positions[0][0],
        positions[1][1] - positions[0][1],
        positions[1][2] - positions[0][2],
    ];
    let b = [
        positions[2][0] - positions[0][0],
        positions[2][1] - positions[0][1],
        positions[2][2] - positions[0][2],
    ];
    let normal = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ][axis];
    if normal == 0.0 {
        return None;
    }
    let (u0, u1, v0, v1) = (
        low_u.floor() as u8,
        high_u.ceil() as u8,
        low_v.floor() as u8,
        high_v.ceil() as u8,
    );
    if u0 == u1 || v0 == v1 {
        return None;
    }
    Some(FacePlane {
        axis: axis as u8,
        plane: at as i32,
        sign: if normal > 0.0 { 1 } else { -1 },
        u0, u1, v0, v1,
    })
}
