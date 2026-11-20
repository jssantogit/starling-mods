// HEAD hook on Renderer::diagnostic_pass_times. Reproduce exactly the stock
// world and shader timer fields from pinned Starling, adding an Exycraft probe.
// This is only invoked for opt-in performance samples, not regular frames.
fn exycraft_before_diagnostic_pass_times(
    renderer: &Renderer,
) -> core::ops::ControlFlow<serde_json::Value> {
    let shader = match (&renderer.pack_post, &renderer.native) {
        (Some((_, Some(post))), _) => post.timing.as_ref(),
        (_, Some(native)) => native.timing.as_ref(),
        _ => None,
    };
    let mut result = serde_json::json!({
        "world": renderer.own_timing.as_ref().map(pack_timing::PassTiming::distributions),
        "shader": shader.map(pack_timing::PassTiming::distributions),
    });
    exycraft_probe::enrich(&mut result, Some(renderer.info.name.as_str()));
    core::ops::ControlFlow::Break(result)
}
