//! Greedy face-rectangle sweep, adapted from Greedy Meshing Mod (Sierra, MIT).
//! This core is NOT yet hooked into Starling's 26.3 renderer.

/// A 2D rectangle of identical visible faces in one 16x16 direction/plane.
/// Units are blocks; never extend across the section boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub col: u8,
    pub row: u8,
    pub width: u8,
    pub height: u8,
    pub key: u64,
}

/// Convert eligible visible faces into maximal non-overlapping rectangles.
/// None means invisible or ineligible: these faces must stay on the caller's
/// original rendering path. A key must include material and lighting constraints.
pub fn merge_plane(_cells: &[Option<u64>; 256]) -> Vec<Rect> {
    Vec::new() // TDD RED: tests first; implement in the next commit.
}
