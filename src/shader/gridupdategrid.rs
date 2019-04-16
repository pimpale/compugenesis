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

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

layout(binding = 0) uniform GridMetadata {
  uint xsize;
  uint ysize;
  uint zsize;
} gridMetadata;

layout(binding = 1) buffer GridData{
  GridCell gridCell[];
} gridData;

void main() {
  uint id = gl_GlobalInvocationID.x;
  gridData.gridCell[id].temperature = 5;
}
"
}
