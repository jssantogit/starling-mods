// The original camera modifier returns from the spyglass branch before the
// effect_scale interpolation. When effect_scale == 0, all other modifiers
// interpolate to exactly 1.0, so this shortcut preserves that ordering.
pub(crate) fn zero_effects_fov(
    effect_scale: f32,
    first_person: bool,
    using_spyglass: bool,
) -> Option<f32> {
    if effect_scale != 0.0 {
        return None;
    }
    Some(if first_person && using_spyglass {
        0.1
    } else {
        1.0
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_effects_resets_normal_fov() {
        assert_eq!(zero_effects_fov(0.0, true, false), Some(1.0));
        assert_eq!(zero_effects_fov(0.0, false, false), Some(1.0));
    }

    #[test]
    fn spyglass_stays_zoomed_in_first_person() {
        assert_eq!(zero_effects_fov(0.0, true, true), Some(0.1));
    }

    #[test]
    fn third_person_spyglass_uses_normal_fov() {
        assert_eq!(zero_effects_fov(0.0, false, true), Some(1.0));
    }

    #[test]
    fn nonzero_scale_runs_original_even_with_spyglass() {
        for scale in [0.01, 1.0, -0.1, f32::NAN] {
            assert_eq!(zero_effects_fov(scale, true, true), None);
        }
    }

    #[test]
    fn negative_zero_behaves_as_disabled_effects() {
        assert_eq!(zero_effects_fov(-0.0, true, false), Some(1.0));
        assert_eq!(zero_effects_fov(-0.0, true, true), Some(0.1));
    }
}
