// HEAD of Renderer::update_lightmap. Same key and same uploaded pixels:
// skip 256 pixel calculations; Starling's existing GPU upload cache remains.
fn badoptimizations_before_lightmap(
    renderer: &mut crate::gfx::Renderer,
    sky: &crate::environment::Sky,
    darkness: f32,
    gamma: f32,
    night_vision: f32,
    flicker: f32,
    boss_darken: f32,
) -> core::ops::ControlFlow<()> {
    let key = crate::gfx::badoptimizations::Key::new(
        sky.ambient_light_color,
        sky.sky_light_color,
        sky.block_light_tint,
        sky.sky_light_factor,
        darkness,
        gamma,
        night_vision,
        flicker,
        boss_darken,
        renderer.fullbright,
    );
    let renderer_id = renderer as *const crate::gfx::Renderer as usize;
    if crate::gfx::badoptimizations::can_skip(
        renderer_id, key, &renderer.lightmap_pixels, renderer.lightmap_uploaded,
    ) {
        core::ops::ControlFlow::Break(())
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
