vulkano_shaders::shader! {
ty: "compute",
    src: "
#version 450

#define GRIDCELL_TYPE_INVALID_MATERIAL (0)
#define GRIDCELL_TYPE_AIR (1)
#define GRIDCELL_TYPE_WATER (2)
#define GRIDCELL_TYPE_STONE (3)
#define GRIDCELL_TYPE_SOIL (4)


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
