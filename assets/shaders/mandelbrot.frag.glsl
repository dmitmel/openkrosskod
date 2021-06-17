#version 100

#ifdef GL_ES
precision highp float;
#endif

// #define SHADER_IMPL

varying vec2 v_screen_pos;
varying vec2 v_world_pos;

#ifdef SHADER_IMPL
  uniform int u_max_iterations;
  uniform float u_escape_radius;
  uniform bool u_julia_mode;
  uniform vec2 u_starting_point;
  uniform vec2 u_unit_size;
#else
  uniform sampler2D u_tex;
#endif

#ifdef SHADER_IMPL
  vec2 iterate(vec2 z, vec2 c) {
    return vec2(
      z.x*z.x - z.y*z.y + c.x,
      2.0 * z.x * z.y   + c.y
    );
  }
#endif

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
  float result = 1.0;

#ifdef SHADER_IMPL

  vec2 start = u_starting_point / u_unit_size;
  vec2 z = u_julia_mode ? v_world_pos : vec2(0.0);
  vec2 c = u_julia_mode ? start : v_world_pos;

  int iter = 0;
  float esc_r = u_escape_radius;
  while (iter < u_max_iterations && dot(z, z) <= esc_r * esc_r) {
    z = iterate(z, c);
    iter++;
  }

  if (iter < u_max_iterations) {
    // <https://en.wikipedia.org/wiki/Plotting_algorithms_for_the_Mandelbrot_set#Continuous_(smooth)_coloring>
    float nu = log2(log2(dot(z, z)) / 2.0); // == log2(log2(length(z)))
    float smooth_iter = float(iter) + 1.0 - nu;

    result = smooth_iter / float(u_max_iterations);
  } else {
    result = 1.0;
  }

#else

  result = texture2D(u_tex, v_screen_pos / 2.0 + 0.5).r;

#endif

  vec3 rgb_color = vec3(0.0);

  if (result < 1.0) {
    vec3 hsv_color = vec3(
      map(result, 0.0, 1.0, 2.0/3.0, 0.0),
      1.0,
      map(result, 0.0, 1.0, 0.2, 2.0)
    );
    rgb_color = hsv2rgb(hsv_color);
  }

  rgb_color = clamp(rgb_color, vec3(0.0), vec3(1.0));
  gl_FragColor = vec4(rgb_color, 1.0);
}
