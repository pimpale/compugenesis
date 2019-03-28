vulkano_shaders::shader! {
ty: "compute",
    src: "
#version 450

#define GRIDCELL_INVALID_MATERIAL (0)
#define GRIDCELL_WATER (1)
#define GRIDCELL_STONE (2)
#define GRIDCELL_SOIL (3)


struct GridCell {
  uint typeCode;
  float temperature;
  float moisture;
  float sunlight;
  float gravity;
  float plantDensity;
};


layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer GridBuffer {
    GridCell gridCells[];
} buf;

void main() {
    uint idx = gl_GlobalInvocationID.x;
}
"
}
