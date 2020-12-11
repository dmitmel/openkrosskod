#version 100

#ifdef GL_ES
precision highp float;
#endif

#if __VERSION__ >= 130
  #define varying out
  #define attribute in
  #define texture2D texture
#endif

varying vec2 v_position;
varying vec4 v_color;
varying vec2 v_texcoord;

#if defined(VERTEX)

  attribute vec2 a_position;
  attribute vec4 a_color;
  attribute vec2 a_texcoord;

  void main() {
    gl_Position = vec4(a_position, 0.0, 1.0);
    v_position = a_position;
    v_color = a_color;
    v_texcoord = a_texcoord;
  }

#elif defined(FRAGMENT)

  uniform sampler2D u_tex;

  const vec3 GAMMA_CORRECTION_RGB = vec3(1.0 / 2.2);

  vec3 gamma_correct(vec3 color) {
    return pow(color, GAMMA_CORRECTION_RGB);
  }

  vec4 gamma_correct(vec4 color) {
    return vec4(gamma_correct(color.rgb), color.a);
  }

  void main() {
    gl_FragColor = mix(texture2D(u_tex, v_texcoord), gamma_correct(v_color), v_color.a);
  }

#endif
