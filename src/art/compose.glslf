#version 450

layout(location = 0) in vec2 screen_coord;
layout(location = 0) out vec4 colour;

layout(set = 0, binding = 0) uniform sampler2D bg3;
layout(set = 0, binding = 1) uniform sampler2D bg4;

void main() {
  vec4 c3 = texture(bg3, screen_coord);
  vec4 c4 = texture(bg4, screen_coord);
  c4.a /= 2;
  colour.rgb = c4.rgb * c4.a + c3.rgb * (1.0 - c4.a);
  colour.a = 1;
}
