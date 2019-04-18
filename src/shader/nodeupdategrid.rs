vulkano_shaders::shader! {
    ty: "compute",
    src: "
#version 450

/* BEGIN COMMON HEADER */

#define STATUS_GARBAGE 0
#define STATUS_DEAD 1
#define STATUS_ALIVE 2
#define STATUS_NEVER_ALIVE 3

/* max unsigned integer */
#define MAX_UINT 4294967295 
/* Acts like a null pointer */
#define INVALID_INDEX MAX_UINT 

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
  vec3 absolutePositionCache;
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

/* BEGIN COMMON HEADER */

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;


layout(binding = 0) uniform NodeMetadata {
    uint freePtr; // Index of free stack
    uint nodeDataCapacity; // Number of nodes that can fit within the buffer
} nodeMetadata;

layout(binding = 1) buffer NodeData {
  Node nodes[];
} nodeData;


layout(binding = 2) uniform GridMetadata {
  uint xsize;
  uint ysize;
  uint zsize;
} gridMetadata;

layout(binding = 3) buffer GridData{
  GridCell gridCell[];
} gridData;

void main() {
    uint id = gl_GlobalInvocationID.x;
    if(id < nodeData.nodes.length()) {
        nodeData.nodes[id].age++;
    }
}
"
}
