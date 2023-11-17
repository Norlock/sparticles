# Sparticles
GPU Particle system written in Rust + WGPU + EGUI

## Roadmap
- [x] Performance test gpu
- [x] Export (Post fx / animation settings).
- [ ] Seperate post fx settings per emitter
- [ ] Create noise texture for effects
  - [ ] Create these textures in gui
  - [ ] Create multiple at once using different color channels
- [x] Export shader to file
- [ ] Use rust-gpu, maybe?
- [ ] Being able to pause
- [x] Gui events to update
- [x] Fix recreate light emitter
- [ ] Able to import models + materials
  - [ ] Don't reload file if emitter recreate

## Gui
- [x] Show all possible diffuse textures in map
- [x] Preview mode of textures

## Post fx
- [x] Bloom
- [ ] Depth of view
- [ ] Motion blur

## Post fx overhaul
- [x] Create one bindgrouplayout + 2 (ping pong) bindgroups
- [x] Textures are stored in the bind group texture arrays 
- [x] Downscaled is just the same fx texture but uses only top left

- [x] Simplify API and make only one post fx trait + register trait
- [x] Split gaussian again in hor ver
- [ ] Displacement map
