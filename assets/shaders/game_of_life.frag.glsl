#version 100

#ifdef GL_ES
precision highp float;
#endif

uniform vec4 u_global_color;

uniform sampler2D u_chunk_texture;

varying vec2 v_pos;

const float STATE_COLOR_STEPS = float(32);
const float HOT_HUE = 0.0;
const float COLD_HUE = 2.0 / 3.0;

// <https://github.com/hughsk/glsl-hsv2rgb/blob/1b1112c03408c19c0c64017f433d19f1f11049ba/index.glsl>
vec3 hsv2rgb(vec3 c) {
  vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
  return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

void main() {
  float state = texture2D(u_chunk_texture, v_pos).r * float(0xff);
  vec3 hsv = vec3(clamp((state - 1.0) / STATE_COLOR_STEPS, HOT_HUE, COLD_HUE), 1.0, 1.0);
  gl_FragColor = vec4(hsv2rgb(hsv) * float(state > 0.0), 1.0) * u_global_color;
}
