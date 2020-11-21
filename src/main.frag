#version 150
layout(origin_upper_left) in vec4 gl_FragCoord;
uniform vec2 window_size;
out vec4 out_color;
uniform float t;

vec3 hsv2rgb(vec3 c) {
  vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
  return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

// float f(float x, float y) {
//   // return y - sin(x);
//   // return y - -x*x*x;
//   return cos(y) - sin(x);
// }

void main() {
  float p = distance(gl_FragCoord.xy / window_size, vec2(t/10 - floor(t/10))) * 4;
  out_color = vec4(hsv2rgb(vec3(p, 1, 1)), 1.0);

  // vec2 p = (gl_FragCoord.xy - window_size / 2) / 50;
  // float v = f(p.x, p.y);
  // out_color = abs(v) < 0.1 ? vec4(vec3(v/0.01), 1.0) : vec4(0.0, 0.0, 0.0, 1.0);
  // out_color = vec4(vec3(1.0 - abs(v) / 0.1), 1);
}
