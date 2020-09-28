vulkano_shaders::shader! {
ty: "fragment",
    src: "
#version 450

layout(location = 0) in vec4 fragColor;
layout(location = 0) out vec4 outColor;

void main() {
    outColor = fragColor;
}"
}
