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

layout(set = 0, binding = 2) uniform GridMetadataRead {
    uint xsize;
    uint ysize;
    uint zsize;
} gridMetadataRead;

layout(set = 0, binding = 2) uniform GridMetadataWrite {
    uint xsize;
    uint ysize;
    uint zsize;
} gridMetadataWrite;

layout(set = 0, binding = 3) buffer GridBufferRead { 
    GridCell gridCell[]; 
} gridDataRead;

layout(set = 0, binding = 3) buffer GridBufferWrite { 
    GridCell gridCell[]; 
} gridDataWrite;

layout(set = 0, binding = 0) uniform NodeMetadataRead {
    uint nodeDataCount;
    uint nodeDataCapacity;
} nodeMetadataRead;

layout(set = 0, binding = 0) uniform NodeMetadataWrite {
    uint nodeDataCount;
    uint nodeDataCapacity;
} nodeMetadataWrite;

layout(set = 0, binding = 1) buffer NodeBufferRead { 
    Node nodes[]; 
} nodeDataWrite;

layout(set = 0, binding = 1) buffer NodeBuffer { 
    Node nodes[]; 
} nodeData;


uint getGridCellId(uint x, uint y, uint z) {
    return (gridMetadata.xsize * gridMetadata.ysize * z +
            gridMetadata.xsize * y + x);
}

void main() {
    uint id = gl_GlobalInvocationID.x;
}
"
}
