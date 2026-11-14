// Compile the *actual* native adapter with minimal Starling-shaped mesh types.
// These tests exercise the real compaction loop, not a second implementation.
mod mesher {
    pub const OPAQUE_LAYER: u16 = 1 << 15;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct TerrainVertex {
        pub pos: [f32; 3],
        pub uv: [u16; 2],
        pub layer: u16,
        pub color: u32,
        pub light: u16,
    }

    #[derive(Default)]
    pub struct SectionMesh {
        pub solid: Vec<TerrainVertex>,
        pub translucent: Vec<TerrainVertex>,
    }

    pub struct Info {
        pub opaque_cube: bool,
        pub block_entity: bool,
        pub offset: Option<()>,
        pub emission: u8,
        pub tint_count: u8,
        pub leaves: bool,
        pub pane: bool,
        pub model: bool,
    }

    pub struct MeshTables {
        pub info: Vec<Info>,
    }

    pub struct RenderAssets {
        pub connected: Vec<u8>,
    }

    pub struct Mesher<'a> {
        pub shader_pack_lighting: bool,
        pub separate_ao: bool,
        pub translucent_layers: &'a [bool],
        pub render: &'a RenderAssets,
        pub tables: &'a MeshTables,
    }

    pub struct Padded {
        pub blocks: Vec<u32>,
    }

    pub struct MeshScratch {
        pub padded: Padded,
    }

    pub fn pi(x: i32, y: i32, z: i32) -> usize {
        (y as usize) * 256 + (z as usize) * 16 + (x as usize)
    }

    pub mod greedymeshing_core {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../mods/greedymeshing/core.rs"
        ));
    }

    pub mod greedymeshing_runtime {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../mods/greedymeshing/native/mixins/greedymeshing/runtime.rs"
        ));
    }

    pub fn compact_test(mesh: &mut SectionMesh, mesher: &Mesher<'_>, scratch: &MeshScratch) {
        greedymeshing_runtime::compact(mesh, mesher, scratch);
    }
}

use mesher::{MeshScratch, MeshTables, Mesher, OPAQUE_LAYER, Padded, RenderAssets, SectionMesh, TerrainVertex, Info};
use mesher::greedymeshing_core::project_quad;
use mesher::compact_test;

fn solid_state() -> Info {
    Info {
        opaque_cube: true,
        block_entity: false,
        offset: None,
        emission: 0,
        tint_count: 0,
        leaves: false,
        pane: false,
        model: true,
    }
}

fn make_quad(axis: usize, sign: i8, u: u8, v: u8, level: u8) -> [TerrainVertex; 4] {
    let uv: [[u8; 2]; 4] = if (sign > 0) == (axis != 1) {
        [[0, 0], [1, 0], [1, 1], [0, 1]]
    } else {
        [[0, 0], [0, 1], [1, 1], [1, 0]]
    };
    let (u_axis, v_axis) = match axis {
        0 => (1, 2),
        1 => (0, 2),
        _ => (0, 1),
    };
    std::array::from_fn(|i| {
        let mut pos = [0.0; 3];
        pos[axis] = level as f32;
        pos[u_axis] = f32::from(u + uv[i][0]);
        pos[v_axis] = f32::from(v + uv[i][1]);
        TerrainVertex {
            pos,
            uv: [u16::from(uv[i][0]) * 32768, u16::from(uv[i][1]) * 32768],
            layer: OPAQUE_LAYER | 7,
            color: 0xFF_99_88_77,
            light: 0xF0F0,
        }
    })
}

fn example(axis: usize, sign: i8, width: u8, height: u8) -> (SectionMesh, MeshScratch) {
    let mut mesh = SectionMesh::default();
    let mut scratch = MeshScratch {
        padded: Padded { blocks: vec![0; 4096] },
    };
    for v in 4..4 + height {
        for u in 2..2 + width {
            let face = make_quad(axis, sign, u, v, 7);
            let projection = project_quad(&face.map(|vertex| vertex.pos)).unwrap();
            let [x, y, z] = projection.block_coords().unwrap();
            scratch.padded.blocks[mesher::pi(x, y, z)] = 1;
            mesh.solid.extend_from_slice(&face);
        }
    }
    (mesh, scratch)
}

fn run(mesh: &mut SectionMesh, scratch: &MeshScratch, pack_lighting: bool, connected: bool) {
    let tables = MeshTables { info: vec![solid_state(), solid_state()] };
    let render = RenderAssets { connected: if connected { vec![1] } else { Vec::new() } };
    let mesher = Mesher {
        shader_pack_lighting: pack_lighting,
        separate_ao: false,
        translucent_layers: &[],
        render: &render,
        tables: &tables,
    };
    compact_test(mesh, &mesher, scratch);
}

#[test]
fn actual_adapter_merges_two_by_two_for_all_six_directions() {
    for axis in 0..3 {
        for sign in [-1, 1] {
            let (mut mesh, scratch) = example(axis, sign, 2, 2);
            assert_eq!(mesh.solid.len(), 16);
            run(&mut mesh, &scratch, false, false);
            assert_eq!(mesh.solid.len(), 4, "axis {axis} sign {sign}");
            let merged: [TerrainVertex; 4] = mesh.solid.clone().try_into().unwrap();
            let face = project_quad(&merged.map(|v| v.pos)).unwrap();
            assert_eq!(face.sign, sign);
            assert_eq!(face.axis as usize, axis);
            assert_eq!((face.u0, face.u1, face.v0, face.v1), (2, 4, 4, 6));
            assert!(merged.iter().any(|vertex| vertex.uv.contains(&65535)));
        }
    }
}

#[test]
fn opposite_normals_are_not_interleaved_into_one_quad() {
    let (mut mesh, mut scratch) = example(1, 1, 2, 1);
    let another = make_quad(1, -1, 4, 4, 7);
    let proj = project_quad(&another.map(|v| v.pos)).unwrap();
    let [x, y, z] = proj.block_coords().unwrap();
    scratch.padded.blocks[mesher::pi(x, y, z)] = 1;
    mesh.solid.extend_from_slice(&another);
    run(&mut mesh, &scratch, false, false);
    assert_eq!(mesh.solid.len(), 8, "one merged top plus one bottom face");
    let first_quad: [TerrainVertex; 4] = mesh.solid[0..4].try_into().unwrap();
    let first = project_quad(&first_quad.map(|vertex| vertex.pos)).is_some();
    assert!(first);
}

#[test]
fn coplanar_translucent_overlay_blocks_the_merge() {
    let (mut mesh, scratch) = example(1, 1, 2, 1);
    mesh.translucent.extend_from_slice(&make_quad(1, -1, 2, 4, 7));
    run(&mut mesh, &scratch, false, false);
    assert_eq!(mesh.solid.len(), 8, "do not swallow overlaid surface");
    assert_eq!(mesh.translucent.len(), 4);
}

#[test]
fn shader_lighting_or_connected_textures_keep_original_faces() {
    let (mut lit, scratch) = example(0, 1, 2, 2);
    run(&mut lit, &scratch, true, false);
    assert_eq!(lit.solid.len(), 16);
    let (mut ctm, scratch) = example(0, 1, 2, 2);
    run(&mut ctm, &scratch, false, true);
    assert_eq!(ctm.solid.len(), 16);
}
