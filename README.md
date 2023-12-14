# Sparticles
GPU Particle system written in Rust + WGPU + EGUI

## Showcase

https://github.com/Norlock/sparticles/assets/7510943/b0bfde2b-b44d-40fb-b88d-0c0e979627ab

## Roadmap
- [x] Performance test gpu
- [x] Export (Post fx / animation settings).
- [ ] Seperate post fx settings per emitter
- [ ] Create noise texture for effects
  - [ ] Create these textures in gui
  - [ ] Create multiple at once using different color channels
- [x] Export shader to file
- [ ] Use rust-gpu, maybe?
- [x] Being able to pause
- [x] Gui events to update
- [x] Fix recreate light emitter
- [x] Able to import models + materials
  - [x] Don't reload file if emitter recreate
- [ ] Create custom materials
- [ ] Debug params (to print imports for example)
- [ ] Improve materials (KHR, HDR)
- [x] Async emitters
- [ ] Better events system
- [ ] Update

## Gui
- [x] Show all possible diffuse textures in map
- [x] Preview mode of textures
- [x] Load models from GUI.
- [x] Seperate GUI views

## Post fx
- [x] Bloom
- [ ] Depth of view
- [ ] Motion blur
- [ ] Particle trails
- [ ] Displacement map
