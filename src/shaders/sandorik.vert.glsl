#ifdef GL_ES
precision highp float;
#endif

attribute vec2 position;
varying vec2 frag_position;
attribute vec4 color;
varying vec4 frag_color;
attribute vec2 texcoord;
varying vec2 frag_texcoord;

void main(void) {
  gl_Position = vec4(position, 0.0, 1.0);
  frag_position = position;
  frag_color = color;
  frag_texcoord = texcoord;
}
