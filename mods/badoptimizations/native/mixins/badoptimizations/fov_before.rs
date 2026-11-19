// The special-case's exact inputs in Starling 09fa16c come from
// horizon-client-render::camera::field_of_view_modifier.
fn badoptimizations_shortcut_disabled_fov(
    player: &crate::camera::FovPlayer,
    first_person: bool,
    effect_scale: f32,
) -> core::ops::ControlFlow<f32> {
    let using_spyglass = matches!(
        player.using,
        Some((crate::camera::UseItem::Spyglass, _))
    );
    match crate::camera::badoptimizations_fov::zero_effects_fov(
        effect_scale,
        first_person,
        using_spyglass,
    ) {
        Some(value) => core::ops::ControlFlow::Break(value),
        None => core::ops::ControlFlow::Continue(()),
    }
}
