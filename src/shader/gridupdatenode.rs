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
  bool visible; // visibility during vertex generation
  float length; // Length in meters (used for displacement)
  float radius;   // Radius in square meters (used for photosynthesis + wind)
  float volume; // Volume in cubic meters (used for light calculations)
  vec3 absolutePositionCache; // Cache of absolute position
  mat4 transformation; //Transformation from parent node
};


struct GridCell {
    uint typeCode;
    float temperature;
    float moisture;
    float sunlight;
    float gravity;
    float plantDensity;
};

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;


layout(binding = 0) uniform NodeMetadata {
    uint nodeDataCount;
    uint nodeDataCapacity;
} nodeMetadata;

layout(binding = 1) buffer NodeBuffer { 
    Node nodes[]; 
} nodeData;

layout(binding = 2) uniform GridMetadata {
    uint xsize;
    uint ysize;
    uint zsize;
} gridMetadata;

layout(binding = 3) buffer GridBuffer { 
    GridCell gridCell[]; 
} gridData;

uint getGridCellId(uint x, uint y, uint z) {
    return (gridMetadata.xsize * gridMetadata.ysize * z +
            gridMetadata.xsize * y + x);
}

void main() {
    uint id = gl_GlobalInvocationID.x;
}
"
}
