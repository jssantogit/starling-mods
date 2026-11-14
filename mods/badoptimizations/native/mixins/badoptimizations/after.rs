// RETURN of Renderer::update_lightmap. Cache only the original method's
// finished result. The HEAD cancellation also passes through RETURN safely.
#[allow(clippy::too_many_arguments)]
fn badoptimizations_after_lightmap(
    result: (),
    renderer: &mut crate::gfx::Renderer,
    sky: &crate::environment::Sky,
    darkness: f32,
    gamma: f32,
    night_vision: f32,
    flicker: f32,
    boss_darken: f32,
) {
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
    let pixels = renderer.lightmap_pixels;
    let uploaded = renderer.lightmap_uploaded;
    renderer
        .__horizon_mixins_badoptimizations_lightmap_cache_mut()
        .remember(key, &pixels, uploaded);
    result
}
