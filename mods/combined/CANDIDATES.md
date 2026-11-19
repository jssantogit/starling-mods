# Future source-mod ports: compatibility audit required

Only ports of existing optimization mods are in scope. Do not implement a new Rust optimization outside a traceable mod technique. Before coding, confirm the source license allows adaptation and identify an actual missing operation in pinned Starling. This document is **planning only** and makes no performance claims.

| Mod source | Technique to inspect | Current decision |
|---|---|---|
| [ImmediatelyFast](https://github.com/RaphiMC/ImmediatelyFast) | Batching of immediate-mode rendering and reductions in redundant framebuffer switches | Candidate, but Starling already uses a custom wgpu renderer rather than Java's immediate-mode stack; require proof of an equivalent missing path before porting |
| [Particle Core](https://modrinth.com/mod/particle-core) | Particle culling and batching | Candidate, but overlaps BadOptimizations' empty-particle fast path; must identify a separate equivalent and protect visual parity |
| [FerriteCore](https://github.com/malte0811/FerriteCore) | Shared blockstate/model memory | Candidate only if Rust Starling's memory layout has equivalent duplication; desktop memory results are not transferable |
| [More Culling](https://modrinth.com/mod/moreculling) | Additional visibility rules | Do **not** merge: upstream author states permission is required to merge into clients or other mods; also overlaps Starling's existing leaf and entity culling |
| Entity Culling / Cull Leaves | Entity occlusion and foliage culling | Deprioritized as already implemented upstream |

The candidate list is not a commitment; source and runtime audits determine what is safe. Current combined build includes only BadOptimizations and Greedy Meshing.
