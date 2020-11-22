#ifdef GL_ES
precision highp float;
#endif

varying vec2 frag_position;
varying vec4 frag_color;
varying vec2 frag_texcoord;
uniform sampler2D tex;

const vec3 GAMMA_CORRECTION_RGB = vec3(1.0 / 2.2);

vec3 gamma_correct(vec3 color) {
  return pow(color, GAMMA_CORRECTION_RGB);
}

vec4 gamma_correct(vec4 color) {
  return vec4(gamma_correct(color.rgb), color.a);
}

void main(void) {
  gl_FragColor = mix(texture2D(tex, frag_texcoord), gamma_correct(frag_color), frag_color.a);
}
