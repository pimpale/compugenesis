vulkano_shaders::shader! {
    ty: "compute",
    src: "
#version 450

struct Node {
  uint leftChildIndex;
  uint rightChildIndex;
  uint parentIndex;
  uint age;
  uint archetypeId;
  uint status;
  bool visible;
  float area;
  float length;
  vec4 absolutePositionCache;
  mat4 transformation;
};

struct GridCell {
  uint typeCode;
  float temperature;
  float moisture;
  float sunlight;
  float gravity;
  float plantDensity;
};

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer NodeBuffer {
    Node nodes[];
    GridCell gridCells[];
} buf;

void main() {
    uint idx = gl_GlobalInvocationID.x;
    //TODO figure out how to lock a grid cell.
}
"
}
