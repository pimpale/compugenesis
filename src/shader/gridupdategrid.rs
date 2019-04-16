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

layout(binding = 0) uniform Constants {
  uint xsize;
  uint ysize;
  uint zsize;
} consts;

layout(binding = 1) buffer GridBuffer {
  GridCell gridCell[];
} buf;

void main() {
  uint id = gl_GlobalInvocationID.x;

  uint xsize = consts.xsize;
  uint ysize = consts.ysize;
  uint zsize = consts.zsize;

  buf.gridCell[id].temperature = 5;
}
"
}
