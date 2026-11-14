// RETURN hook on Mesher::mesh_reusing, after vanilla has built the full mesh.
fn greedymeshing_after_mesh(
    mut result: crate::mesher::SectionMesh,
    mesher: &crate::mesher::Mesher<'_>,
    scratch: &crate::mesher::MeshScratch,
) -> crate::mesher::SectionMesh {
    crate::mesher::greedymeshing_runtime::compact(&mut result, mesher, scratch);
    result
}
