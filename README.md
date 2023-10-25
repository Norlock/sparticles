# Sparticles
Particle system written in rust + WGPU + EGUI

## Roadmap
- [ ] Performance test gpu
- [x] Export (Post fx / animation settings).
- [ ] Seperate post fx settings per emitter
- [ ] Create noise texture for effects
  - [ ] Create these textures in gui
- [ ] Export shader to file
- [ ] Use rust-gpu

## Gui
- [x] Show all possible diffuse textures in map
- [x] Preview mode of textures

## Post fx
- [ ] Depth of view
- [ ] Motion blur

## Post fx overhaul
- [x] Create one bindgrouplayout + 2 (ping pong) bindgroups
- [x] Textures are stored in the bind group texture arrays 
- [x] Downscaled is just the same fx texture but uses only top left
  - [ ] Downscale maybe just in one texture but use offset

- [x] Simplify API and make only one post fx trait + register trait
- [ ] Noise textures are updated once in a while
- [x] Split gaussian again in hor ver

