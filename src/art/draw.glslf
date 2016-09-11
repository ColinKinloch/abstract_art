#version 450

layout(location = 0) in vec2 screen_coord;
layout(location = 0) out vec4 colour;

layout(set = 0, binding = 0) uniform sampler2D frame;

void main() {
  colour = texture(frame, screen_coord);
}
