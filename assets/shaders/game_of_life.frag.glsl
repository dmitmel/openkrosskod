#version 100

#ifdef GL_ES
precision highp float;
#endif

varying vec4 v_color;

void main() {
  gl_FragColor = v_color;
}
