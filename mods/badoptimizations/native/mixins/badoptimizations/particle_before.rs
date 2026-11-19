// ParticleEngine.render only writes vertices from self.particles.
// Empty engines can bypass camera quaternion and ParticleLightCache setup.
fn badoptimizations_skip_empty_particles(
    particles: &crate::particles::ParticleEngine,
) -> core::ops::ControlFlow<()> {
    if particles.is_empty() {
        core::ops::ControlFlow::Break(())
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
