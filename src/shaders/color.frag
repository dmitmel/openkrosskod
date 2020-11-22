#ifdef GL_ES
precision highp float;
#endif

varying vec2 frag_position;
varying vec3 frag_color;
varying vec2 frag_texcoord;
varying float frag_color_intensity;
uniform sampler2D tex;
uniform vec2 window_size;

void main(void) {
  gl_FragColor = mix(vec4(frag_color, 1.0), texture2D(tex, frag_texcoord), frag_color_intensity);
}
