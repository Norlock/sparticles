# Sparticles
Particle system written in rust + WGPU + EGUI

## Roadmap
- [ ] Performance test gpu
- [x] Export (Post fx / animation settings).
- [ ] Seperate post fx settings per spawner
- [ ] Create noise texture for effects
  - [ ] Create these textures in gui

## Gui
- [ ] Show all possible diffuse textures in map
- [ ] Preview mode of textures

## Post fx
- [ ] Depth of view
- [ ] Motion blur
- [ ] Being able to extend Fx State bind groups as long as the order is preserved so you have more
flexibility 
- [ ] Create preview texture and register with egui to display post fx 

## Post fx overhaul
- [ ] Create one bindgrouplayout + 2 (ping pong) bindgroups
- [ ] Textures are stored in the bind group and determined how many have available
- [ ] Downscaled is just the same fx texture but uses only top left
- [ ] Simplify API and make only one post fx trait + register trait
- [ ] Noise textures are updated once in a while
- [ ] Split gaussian again in hor ver

