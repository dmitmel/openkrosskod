#ifdef GL_ES
precision highp float;
#endif

attribute vec2 position;
varying vec2 frag_position;
attribute vec3 color;
varying vec3 frag_color;
attribute vec2 texcoord;
varying vec2 frag_texcoord;
attribute float color_intensity;
varying float frag_color_intensity;

uniform float time;

vec2 rotate(vec2 v, float a) {
  float s = sin(a);
  float c = cos(a);
  mat2 m = mat2(c, -s, s, c);
  return m * v;
}

void main() {
  // gl_Position = vec4(rotate(position, time), 0.0, 1.0);
  gl_Position = vec4(position, 0.0, 1.0);
  frag_position = position;
  frag_color = color;
  frag_texcoord = texcoord;
  frag_color_intensity = color_intensity;
}
