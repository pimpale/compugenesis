vulkano_shaders::shader! {
    ty: "compute",
    src:
"#version 450

/* BEGIN COMMON HEADER */

#define NODE_STATUS_GARBAGE 0
#define NODE_STATUS_DEAD 1
#define NODE_STATUS_ALIVE 2
#define NODE_STATUS_NEVER_ALIVE 3

#define GRIDCELL_TYPE_INVALID_MATERIAL 0
#define GRIDCELL_TYPE_AIR 1
#define GRIDCELL_TYPE_WATER 2
#define GRIDCELL_TYPE_STONE 3
#define GRIDCELL_TYPE_SOIL 4

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
  bool visible; // visibility during vertex generation
  float length; // Length in meters (used for displacement)
  float area;   // Area in square meters (used for photosynthesis + wind)
  float volume; // Volume in cubic meters (used for light calculations)
  vec3 absolutePositionCache; // Cache of absolute position
  mat4 transformation; //Transformation from parent node
};

struct GridCell {
  uint typeCode;
  uint temperature;
  uint moisture;
  uint sunlight;
  uint gravity;
  uint plantDensity;
};

/* END COMMON HEADER */

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

layout(binding = 3) buffer GridData {
  GridCell gridCell[];
} gridData;

uint getGridId(vec3 loc) {
    uint x =  int(loc.x);
    uint y =  int(loc.y);
    uint z =  int(loc.z);
    return (gridMetadata.ysize*gridMetadata.xsize*z +
            gridMetadata.xsize*y + 
            x);
}

void main() {
    uint nid = gl_GlobalInvocationID.x;
    // Basic checks
    if(nid < nodeData.nodes.length() 
        && nodeData.nodes[nid].status != NODE_STATUS_GARBAGE) {
        Node n = nodeData.nodes[nid];
        uint gid = getGridId(n.absolutePositionCache);
        uint vol = int(n.volume*pow(100, 3));
        atomicAdd(gridData.gridCell[gid].plantDensity, vol);
        // calculate and subtract moisture from soil moisture
        // subtract sunlight using plant density
    }
}
"
}
