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

layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;

layout(binding = 0) uniform GridMetadata {
    uint xsize;
    uint ysize;
    uint zsize;
}
gridMetadata;

layout(binding = 1) uniform NodeMetadata {
    uint nodeDataCount;
    uint nodeDataCapacity;
} nodeMetadata;

layout(binding = 2) buffer NodeBuffer { 
    Node nodes[]; 
}
nodeData;

layout(binding = 3) buffer GridBuffer { 
    GridCell gridCell[]; 
}
gridData;

uint getGridCellId(uint x, uint y, uint z) {
    return (gridMetadata.xsize * gridMetadata.ysize * z +
            gridMetadata.xsize * y + x);
}

void main() {
    uint id = gl_GlobalInvocationID.x;
    nodeData.nodes[id].age++;
}
"
}
