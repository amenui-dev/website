# Particles

When a morph has many cells it is simulated on the GPU (a compute shader, or ping-ponging float targets), otherwise on the CPU, where a mesh side's cells start on its surface. `Look::particles.gpu_above` decides where the GPU takes over.
