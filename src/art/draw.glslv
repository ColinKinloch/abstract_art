#version 450

layout(location = 0) in vec2 position;
layout(location = 0) out vec2 screen_coord;

void main() {
  screen_coord = position * 0.5;
  gl_Position = vec4(position - 1.0, 0.0, 1.0);
}
