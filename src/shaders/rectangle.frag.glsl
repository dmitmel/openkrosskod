#ifdef GL_ES
precision highp float;
#endif

uniform vec4 u_color;
uniform sampler2D u_tex;

varying vec2 v_texcoord;

void main(void) {
  gl_FragColor = u_color * texture2D(u_tex, v_texcoord);
  // if (gl_FragColor.a == 0.0) {
  //   discard;
  // }
}
