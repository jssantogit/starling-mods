// The cache's complete key follows the inputs actually read by
// Renderer::update_lightmap in Starling 09fa16c, not Java's older lightmap.
// No engine dependency: this file compiles standalone for regression tests.
const LIGHTMAP_BYTES: usize = 16 * 16 * 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Key {
    bits: [u32; 16],
}

impl Key {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        ambient: [f32; 3],
        sky_color: [f32; 3],
        block_tint: [f32; 3],
        sky_factor: f32,
        darkness: f32,
        gamma: f32,
        night_vision: f32,
        flicker: f32,
        boss_darken: f32,
        fullbright: bool,
    ) -> Self {
        let mut bits = [0u32; 16];
        bits[..3].copy_from_slice(&ambient.map(f32::to_bits));
        bits[3..6].copy_from_slice(&sky_color.map(f32::to_bits));
        bits[6..9].copy_from_slice(&block_tint.map(f32::to_bits));
        bits[9..].copy_from_slice(&[
            sky_factor.to_bits(),
            darkness.to_bits(),
            gamma.to_bits(),
            night_vision.to_bits(),
            flicker.to_bits(),
            boss_darken.to_bits(),
            u32::from(fullbright),
        ]);
        Self { bits }
    }
}

pub(crate) struct Cache {
    renderer_id: Option<usize>,
    key: Option<Key>,
    pixels: [u8; LIGHTMAP_BYTES],
}

impl Default for Cache {
    fn default() -> Self {
        Self {
            renderer_id: None,
            key: None,
            pixels: [0u8; LIGHTMAP_BYTES],
        }
    }
}

impl Cache {
    // A new instance or an uninitialized GPU texture MUST run the original.
    // Observing output bytes protects against mutations by other render hooks.
    pub(crate) fn can_skip(&self, renderer_id: usize, key: Key, pixels: &[u8; LIGHTMAP_BYTES], uploaded: bool) -> bool {
        uploaded && self.renderer_id == Some(renderer_id) && self.key == Some(key) && &self.pixels == pixels
    }

    pub(crate) fn remember(&mut self, renderer_id: usize, key: Key, pixels: &[u8; LIGHTMAP_BYTES], uploaded: bool) {
        if uploaded {
            self.renderer_id = Some(renderer_id);
            self.key = Some(key);
            self.pixels.copy_from_slice(pixels);
        } else {
            self.renderer_id = None;
            self.key = None;
        }
    }
}

// Last-used renderer per thread: a second renderer only causes a cache miss.
// No global mutex or extra field in an async-constructed Renderer.
thread_local! {
    static LAST: std::cell::RefCell<Cache> = std::cell::RefCell::new(Cache::default());
}

pub(crate) fn can_skip(renderer_id: usize, key: Key, pixels: &[u8; LIGHTMAP_BYTES], uploaded: bool) -> bool {
    LAST.with(|cache| cache.borrow().can_skip(renderer_id, key, pixels, uploaded))
}

pub(crate) fn remember(renderer_id: usize, key: Key, pixels: &[u8; LIGHTMAP_BYTES], uploaded: bool) {
    LAST.with(|cache| cache.borrow_mut().remember(renderer_id, key, pixels, uploaded))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> Key {
        Key::new(
            [0.1, 0.2, 0.3],
            [0.4, 0.5, 0.6],
            [0.7, 0.8, 0.9],
            1.0,
            0.0,
            0.5,
            0.0,
            0.12,
            0.0,
            false,
        )
    }

    #[test]
    fn first_render_and_unuploaded_texture_must_run() {
        let mut cache = Cache::default();
        let pixels = [255u8; LIGHTMAP_BYTES];
        assert!(!cache.can_skip(100, key(), &pixels, false));
        assert!(!cache.can_skip(100, key(), &pixels, true));
        cache.remember(100, key(), &pixels, true);
        assert!(!cache.can_skip(100, key(), &pixels, false));
    }

    #[test]
    fn repeat_with_identical_inputs_and_uploaded_pixels_can_skip() {
        let mut cache = Cache::default();
        let pixels = [12u8; LIGHTMAP_BYTES];
        cache.remember(100, key(), &pixels, true);
        assert!(cache.can_skip(100, key(), &pixels, true));
    }

    #[test]
    fn each_input_change_invalidates_key() {
        let original = key();
        for index in 0..16 {
            let mut changed = original;
            changed.bits[index] ^= 1;
            assert_ne!(original, changed, "key element {index} must invalidate");
            let mut cache = Cache::default();
            let pixels = [0; LIGHTMAP_BYTES];
            cache.remember(100, original, &pixels, true);
            assert!(!cache.can_skip(100, changed, &pixels, true));
        }
    }

    #[test]
    fn key_preserves_exact_float_bits() {
        let mut changed = key();
        changed.bits[10] = (-0.0f32).to_bits();
        assert_ne!(key(), changed);
    }

    #[test]
    fn unexpected_pixel_mutation_forces_recalculation() {
        let mut cache = Cache::default();
        let first = [3u8; LIGHTMAP_BYTES];
        let mut changed = first;
        changed[42] = 4;
        cache.remember(100, key(), &first, true);
        assert!(!cache.can_skip(100, key(), &changed, true));
    }

    #[test]
    fn renderer_instances_have_independent_state() {
        let a = Cache::default();
        let mut b = Cache::default();
        let pixels = [7u8; LIGHTMAP_BYTES];
        b.remember(100, key(), &pixels, true);
        assert!(!a.can_skip(100, key(), &pixels, true));
        assert!(b.can_skip(100, key(), &pixels, true));
    }

    #[test]
    fn invalidated_upload_resets_previous_key() {
        let mut cache = Cache::default();
        let pixels = [17u8; LIGHTMAP_BYTES];
        cache.remember(100, key(), &pixels, true);
        cache.remember(100, key(), &pixels, false);
        assert!(!cache.can_skip(100, key(), &pixels, true));
    }
    #[test]
    fn a_different_renderer_identity_never_reuses_a_lightmap() {
        let mut cache = Cache::default();
        let pixels = [5u8; LIGHTMAP_BYTES];
        cache.remember(100, key(), &pixels, true);
        assert!(!cache.can_skip(200, key(), &pixels, true));
        cache.remember(200, key(), &pixels, true);
        assert!(!cache.can_skip(100, key(), &pixels, true));
        assert!(cache.can_skip(200, key(), &pixels, true));
    }

    #[test]
    fn thread_local_last_renderer_identity_changes_on_switch() {
        let pixels = [42u8; LIGHTMAP_BYTES];
        remember(100, key(), &pixels, true);
        assert!(can_skip(100, key(), &pixels, true));
        remember(200, key(), &pixels, true);
        assert!(!can_skip(100, key(), &pixels, true));
        assert!(can_skip(200, key(), &pixels, true));
    }

}
