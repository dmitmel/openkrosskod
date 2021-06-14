#version 100

#ifdef GL_ES
precision highp float;
#endif

uniform vec4 u_global_color;

varying vec2 v_pos;
varying vec2 v_pos_inside_square;
varying vec4 v_corner_values;

const float GAMMA_CORRECTION = 1.0 / 2.2; // the true value
// const float GAMMA_CORRECTION = 2.2;
// const float GAMMA_CORRECTION = 1.0;

void main() {
  float pair1 = mix(v_corner_values.x, v_corner_values.y, v_pos_inside_square.x);
  float pair2 = mix(v_corner_values.w, v_corner_values.z, v_pos_inside_square.x);
  float value = mix(pair1,             pair2,             v_pos_inside_square.y);
  value = pow(value, GAMMA_CORRECTION);
  gl_FragColor = vec4(vec3(value), 1.0) * u_global_color;
}
