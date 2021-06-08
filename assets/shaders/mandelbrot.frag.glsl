#version 100

#ifdef GL_ES
precision highp float;
#endif

varying vec2 v_pos;

uniform int u_max_iterations;
uniform float u_escape_radius;

vec2 iterate(vec2 z, vec2 c) {
  return vec2(
    z.x*z.x - z.y*z.y + c.x,
    2.0 * z.x * z.y   + c.y
  );
}

// <https://github.com/hughsk/glsl-hsv2rgb/blob/1b1112c03408c19c0c64017f433d19f1f11049ba/index.glsl>
vec3 hsv2rgb(vec3 c) {
  vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
  vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
  return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}

float map(float value, float src_min, float src_max, float dest_min, float dest_max) {
  return (value - src_min) / (src_max - src_min) * (dest_max - dest_min) + dest_min;
}

void main() {
  vec2 z = vec2(0.0);
  vec2 c = v_pos;

  int i = 0;
  float er = u_escape_radius;
  while (i < u_max_iterations && dot(z, z) <= er * er) {
    z = iterate(z, c);
    i++;
  }

  vec3 rgb_color = vec3(0.0);

  if (i < u_max_iterations) {
    // <https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#Continuous_(smooth)_coloring>
    float nu = log2(log2(dot(z, z)) / 2.0); // == log2(log2(length(z)))
    float smooth_i = float(i) + 1.0 - nu;

    float x = smooth_i / float(u_max_iterations);
    vec3 hsv_color = vec3(
      map(x, 0.0, 1.0, 2.0/3.0, 0.0),
      1.0,
      map(x, 0.0, 1.0, 0.2, 2.0)
    );
    rgb_color = hsv2rgb(hsv_color);
  }

  rgb_color = clamp(rgb_color, vec3(0.0), vec3(1.0));
  gl_FragColor = vec4(rgb_color, 1.0);
}
