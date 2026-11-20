// RETURN hook: preserve Starling's existing world/shader pass distributions
// and attach Exycraft's optional process measurements once per report sample.
fn exycraft_after_renderer_diagnostic_pass_times(
    mut result: serde_json::Value,
    renderer: &Renderer,
) -> serde_json::Value {
    exycraft_probe::enrich(&mut result, Some(renderer.info.name.as_str()));
    result
}
