#!/usr/bin/env python3

import math
import os
import random
import sys
from typing import List, Tuple

os.environ["PYGAME_HIDE_SUPPORT_PROMPT"] = "1"

import pygame
import pygame.display
import pygame.draw
import pygame.event
import pygame.time
from pygame.color import Color
from pygame.math import Vector2, Vector3

WINDOW_SIZE: Tuple[int, int] = (800, 800)
WINDOW_FLAGS: int = pygame.RESIZABLE
TARGET_FPS: int = 60

WORLD_UNIT_SIZE: float = 10.0
PARTICLE_COUNT: int = 1000
PARTICLE_SPAWN_RANGE: float = 10.0
PARTICLE_STEP: float = 0.01
PARTICLE_PATH_LEN: int = 10
CAMERA_ROTATION_SPEED: float = 0.01

COLOR_BACKGROUND = Color(0x101010ff)
COLOR_POINT = Color(0xff0000ff)
COLOR_AXIS_X = Color(0xff0000ff)
COLOR_AXIS_Y = Color(0x00ff00ff)
COLOR_AXIS_Z = Color(0x0000ffff)


# <https://en.wikipedia.org/wiki/Lorenz_system>
def attractor_function(p: Vector3, dt: float, out: Vector3) -> None:
  rho = 28.0
  sigma = 10.0
  beta = 8.0 / 3.0

  x, y, z = p.x, p.y, p.z
  dx = sigma * (y - x)
  dy = x * (rho - z)
  dz = x * y - beta * z

  out.x = p.x + dx * dt
  out.y = p.y + dy * dt
  out.z = p.z + dz * dt


def main(argv: List[str]) -> int:
  try:
    pygame.init()

    screen = pygame.display.set_mode(WINDOW_SIZE, WINDOW_FLAGS, vsync=1)
    pygame.display.set_caption(os.path.basename(__file__))

    running = True
    fps_clock = pygame.time.Clock()
    time = 0.0
    fixed_time = 0.0

    screen_w, screen_h = screen.get_size()

    is_mouse_down = False
    prev_mouse_x = 0
    prev_mouse_y = 0
    mouse_x = 0
    mouse_y = 0
    delta_mouse_x = 0
    delta_mouse_y = 0

    particle_paths: List[Vector3] = []
    particle_colors: List[Color] = []
    particle_paths_usage = 1
    particle_paths_offset = 0

    for _ in range(PARTICLE_COUNT):
      color = Color(0)
      color.hsva = (random.random() * 360.0, 100.0, 100.0, 10.0)
      particle_colors.append(color)

      particle_paths.append(
        Vector3((
          random.uniform(-PARTICLE_SPAWN_RANGE, PARTICLE_SPAWN_RANGE),
          random.uniform(-PARTICLE_SPAWN_RANGE, PARTICLE_SPAWN_RANGE),
          random.uniform(-PARTICLE_SPAWN_RANGE, PARTICLE_SPAWN_RANGE),
        ))
      )
      for _ in range(PARTICLE_PATH_LEN - 1):
        particle_paths.append(Vector3())

    camera_rotation_x = 0
    camera_rotation_y = 0
    viewport_matrix = Matrix4.identity()

    def update_viewport_matrix() -> None:
      # <http://math.hws.edu/graphicsbook/source/webgl/simple-rotator.js>
      cos_x = math.cos(camera_rotation_x)
      sin_x = math.sin(camera_rotation_x)
      cos_y = math.cos(camera_rotation_y)
      sin_y = math.sin(camera_rotation_y)
      viewport_matrix.update(
        cos_y,  sin_x * sin_y, -cos_x * sin_y, 0,
        0    ,  cos_x        ,  sin_x        , 0,
        sin_y, -sin_x * cos_y,  cos_x * cos_y, 0,
        0    ,  0            ,  0            , 1,
      )  # yapf: disable

    def project_point(p: Vector3, out: Vector2) -> None:
      viewport_matrix.transform_vec3(p, tmp_project_point_vec3)
      out.x = tmp_project_point_vec3.x * WORLD_UNIT_SIZE + screen_w / 2
      out.y = -tmp_project_point_vec3.y * WORLD_UNIT_SIZE + screen_h / 2

    update_viewport_matrix()

    tmp_projected_path: List[Vector2] = []
    for _ in range(PARTICLE_PATH_LEN):
      tmp_projected_path.append(Vector2())

    axis_tmp1 = Vector2()
    axis_tmp2 = Vector2()
    axis_origin = Vector3((0, 0, 0))
    axis_x = Vector3((1, 0, 0))
    axis_y = Vector3((0, 1, 0))
    axis_z = Vector3((0, 0, 1))

    while running:
      for event in pygame.event.get():
        if event.type == pygame.QUIT or (event.type == pygame.KEYDOWN and event.key == pygame.K_q):
          running = False
        elif event.type == pygame.VIDEORESIZE:
          screen_w, screen_h = screen.get_size()
        elif event.type == pygame.MOUSEBUTTONDOWN:
          is_mouse_down = True
          mouse_x, mouse_y = event.pos
        elif event.type == pygame.MOUSEBUTTONUP:
          is_mouse_down = False
          mouse_x, mouse_y = event.pos
        elif event.type == pygame.MOUSEMOTION:
          mouse_x, mouse_y = event.pos
        elif event.type == pygame.WINDOWLEAVE:
          is_mouse_down = False

      delta_time = fps_clock.tick(TARGET_FPS) / 1000.0
      fixed_delta_time = 1.0 / TARGET_FPS
      time += delta_time
      fixed_time += fixed_delta_time

      delta_mouse_x = mouse_x - prev_mouse_x
      delta_mouse_y = mouse_y - prev_mouse_y

      screen.fill(COLOR_BACKGROUND)  # type: ignore

      if is_mouse_down:
        camera_rotation_x -= delta_mouse_y * CAMERA_ROTATION_SPEED
        camera_rotation_y -= delta_mouse_x * CAMERA_ROTATION_SPEED
        update_viewport_matrix()

      tmp_project_point_vec3 = Vector3()

      project_point(axis_origin, axis_tmp1)
      project_point(axis_x, axis_tmp2)
      pygame.draw.line(screen, COLOR_AXIS_X, axis_tmp1, axis_tmp2)
      project_point(axis_y, axis_tmp2)
      pygame.draw.line(screen, COLOR_AXIS_Y, axis_tmp1, axis_tmp2)
      project_point(axis_z, axis_tmp2)
      pygame.draw.line(screen, COLOR_AXIS_Z, axis_tmp1, axis_tmp2)

      if particle_paths_usage < PARTICLE_PATH_LEN:
        particle_paths_usage += 1

      prev_particle_paths_offset = particle_paths_offset
      particle_paths_offset += 1
      if particle_paths_offset >= particle_paths_usage:
        particle_paths_offset = 0

      for part_idx in range(PARTICLE_COUNT):
        part_off = part_idx * PARTICLE_PATH_LEN
        prev_last_point = particle_paths[part_off + prev_particle_paths_offset]
        last_point = particle_paths[part_off + particle_paths_offset]
        attractor_function(prev_last_point, PARTICLE_STEP, last_point)

        color = particle_colors[part_idx]

        proj_idx = 0
        for i in range(particle_paths_offset + 1, particle_paths_usage):
          project_point(particle_paths[part_off + i], tmp_projected_path[proj_idx])
          proj_idx += 1
        for i in range(0, particle_paths_offset + 1):
          project_point(particle_paths[part_off + i], tmp_projected_path[proj_idx])
          proj_idx += 1

        pygame.draw.lines(screen, color, False, tmp_projected_path[:particle_paths_usage])

      pygame.display.flip()

      prev_mouse_x = mouse_x
      prev_mouse_y = mouse_y

  finally:
    pygame.quit()

  return 0


class Matrix4:
  """
  <https://github.com/dmitmel/openkrosskod/blob/f9b329afd47e4da9185cac7a779a51d73635a1a5/crates/cardboard_math/src/matrices.rs>
  """

  xx: float
  xy: float
  xz: float
  xw: float

  yx: float
  yy: float
  yz: float
  yw: float

  zx: float
  zy: float
  zz: float
  zw: float

  wx: float
  wy: float
  wz: float
  ww: float

  def __init__(
    self,
    xx: float, xy: float, xz: float, xw: float,
    yx: float, yy: float, yz: float, yw: float,
    zx: float, zy: float, zz: float, zw: float,
    wx: float, wy: float, wz: float, ww: float,
  ) -> None:  # yapf: disable
    self.update(xx, xy, xz, xw, yx, yy, yz, yw, zx, zy, zz, zw, wx, wy, wz, ww)

  def update(
    self,
    xx: float, xy: float, xz: float, xw: float,
    yx: float, yy: float, yz: float, yw: float,
    zx: float, zy: float, zz: float, zw: float,
    wx: float, wy: float, wz: float, ww: float,
  ) -> None:  # yapf: disable

    self.xx = xx
    self.xy = xy
    self.xz = xz
    self.xw = xw

    self.yx = yx
    self.yy = yy
    self.yz = yz
    self.yw = yw

    self.zx = zx
    self.zy = zy
    self.zz = zz
    self.zw = zw

    self.wx = wx
    self.wy = wy
    self.wz = wz
    self.ww = ww

  @staticmethod
  def identity() -> "Matrix4":
    return Matrix4(
      1, 0, 0, 0,
      0, 1, 0, 0,
      0, 0, 1, 0,
      0, 0, 0, 1,
    )  # yapf: disable

  def transform_vec3(self, rhs: Vector3, out: Vector3) -> None:
    out.x = self.xx * rhs.x + self.yx * rhs.y + self.zx * rhs.z + self.wx
    out.y = self.xy * rhs.x + self.yy * rhs.y + self.zy * rhs.z + self.wy
    out.z = self.xz * rhs.x + self.yz * rhs.y + self.zz * rhs.z + self.wz

  def mul_mat4(self, rhs: "Matrix4", out: "Matrix4") -> None:
    out.xx = self.xx * rhs.xx + self.yx * rhs.xy + self.zx * rhs.xz + self.wx * rhs.xw
    out.xy = self.xy * rhs.xx + self.yy * rhs.xy + self.zy * rhs.xz + self.wy * rhs.xw
    out.xz = self.xz * rhs.xx + self.yz * rhs.xy + self.zz * rhs.xz + self.wz * rhs.xw
    out.xw = self.xw * rhs.xx + self.yw * rhs.xy + self.zw * rhs.xz + self.ww * rhs.xw

    out.yx = self.xx * rhs.yx + self.yx * rhs.yy + self.zx * rhs.yz + self.wx * rhs.yw
    out.yy = self.xy * rhs.yx + self.yy * rhs.yy + self.zy * rhs.yz + self.wy * rhs.yw
    out.yz = self.xz * rhs.yx + self.yz * rhs.yy + self.zz * rhs.yz + self.wz * rhs.yw
    out.yw = self.xw * rhs.yx + self.yw * rhs.yy + self.zw * rhs.yz + self.ww * rhs.yw

    out.zx = self.xx * rhs.zx + self.yx * rhs.zy + self.zx * rhs.zz + self.wx * rhs.zw
    out.zy = self.xy * rhs.zx + self.yy * rhs.zy + self.zy * rhs.zz + self.wy * rhs.zw
    out.zz = self.xz * rhs.zx + self.yz * rhs.zy + self.zz * rhs.zz + self.wz * rhs.zw
    out.zw = self.xw * rhs.zx + self.yw * rhs.zy + self.zw * rhs.zz + self.ww * rhs.zw

    out.wx = self.xx * rhs.wx + self.yx * rhs.wy + self.zx * rhs.wz + self.wx * rhs.ww
    out.wy = self.xy * rhs.wx + self.yy * rhs.wy + self.zy * rhs.wz + self.wy * rhs.ww
    out.wz = self.xz * rhs.wx + self.yz * rhs.wy + self.zz * rhs.wz + self.wz * rhs.ww
    out.ww = self.xw * rhs.wx + self.yw * rhs.wy + self.zw * rhs.wz + self.ww * rhs.ww


if __name__ == "__main__":
  sys.exit(main(sys.argv))
