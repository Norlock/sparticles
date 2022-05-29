# sparticles

## Architecture

* 1 buffer that is new each time particles are spawned. And is only passed in for compute 1 frame.
* 1 buffer that holds all the particles, this buffer is not fully filled. It leaves room for the new particles to be
  appended 
* 1 buffer that is the output of the two buffers combined.
* Compute pipeline is called x times where x = particles buffer length + spawned particles length. 

## Roadmap
- [ ] Check if I can only pass in index and do the vertex in the shader.
- [ ] Check if I can always rotate circle towards camera.
- [ ] See what work can be done using compute shaders.
