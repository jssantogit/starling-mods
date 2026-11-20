// RETURN hook: augment only the JSON emitted by Starling's opt-in recorder.
// Do not modify the game, renderer, settings, or the existing recorder.
fn exycraft_after_diagnostic_context(
    mut result: serde_json::Value,
    game: &Game<'_>,
) -> serde_json::Value {
    let gpu_name = game.renderer.as_ref().map(|renderer| renderer.info.name.as_str());
    super::exycraft_probe::enrich(&mut result, gpu_name);
    result
}
