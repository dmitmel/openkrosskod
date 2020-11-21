#version 150
in vec2 position;
uniform float t;

vec2 rotate(vec2 v, float a) {
  float s = sin(a);
  float c = cos(a);
  mat2 m = mat2(c, -s, s, c);
  return m * v;
}

void main() {
  gl_Position = vec4(rotate(position, t), 0.0, 1.0);
  // gl_Position = vec4(position, 0.0, 1.0);
}
