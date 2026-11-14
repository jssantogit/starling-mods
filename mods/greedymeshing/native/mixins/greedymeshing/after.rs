// RETURN hook on Mesher::mesh_reusing, after vanilla has built the full mesh.
fn greedymeshing_after_mesh(
    mut result: SectionMesh,
    mesher: &Mesher<'_>,
    scratch: &MeshScratch,
) -> SectionMesh {
    crate::mesher::greedymeshing_runtime::compact(&mut result, mesher, scratch);
    result
}
