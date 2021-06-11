#!/usr/bin/env python3

import math
import os
import random
import sys
from typing import List

os.environ["PYGAME_HIDE_SUPPORT_PROMPT"] = "1"

import pygame.draw
from pygame.color import Color
from pygame.math import Vector2, Vector3
from visualization_core import Matrix4, Visualization

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


class LorenzAttractor(Visualization):

  def __init__(self) -> None:
    super().__init__()
    self.window_opts.caption = os.path.basename(__file__)

    self.particle_paths: List[Vector3] = []
    self.particle_colors: List[Color] = []
    self.particle_paths_usage = 1
    self.particle_paths_offset = 0

    self.camera_rotation_x = 0
    self.camera_rotation_y = 0
    self.viewport_matrix = Matrix4.identity()

  def prestart(self) -> None:
    for _ in range(PARTICLE_COUNT):
      color = Color(0)
      color.hsva = (random.random() * 360.0, 100.0, 100.0, 10.0)
      self.particle_colors.append(color)

      self.particle_paths.append(
        Vector3((
          random.uniform(-PARTICLE_SPAWN_RANGE, PARTICLE_SPAWN_RANGE),
          random.uniform(-PARTICLE_SPAWN_RANGE, PARTICLE_SPAWN_RANGE),
          random.uniform(-PARTICLE_SPAWN_RANGE, PARTICLE_SPAWN_RANGE),
        ))
      )
      for _ in range(PARTICLE_PATH_LEN - 1):
        self.particle_paths.append(Vector3())

    self.update_viewport_matrix()

  def update_viewport_matrix(self) -> None:
    # <http://math.hws.edu/graphicsbook/source/webgl/simple-rotator.js>
    cos_x = math.cos(self.camera_rotation_x)
    sin_x = math.sin(self.camera_rotation_x)
    cos_y = math.cos(self.camera_rotation_y)
    sin_y = math.sin(self.camera_rotation_y)
    self.viewport_matrix.update(
      cos_y,  sin_x * sin_y, -cos_x * sin_y, 0,
      0    ,  cos_x        ,  sin_x        , 0,
      sin_y, -sin_x * cos_y,  cos_x * cos_y, 0,
      0    ,  0            ,  0            , 1,
    )  # yapf: disable

  _tmp_project_point_vec3 = Vector3()

  def project_point(self, p: Vector3, out: Vector2) -> None:
    self.viewport_matrix.transform_vec3(p, self._tmp_project_point_vec3)
    out.x = self._tmp_project_point_vec3.x * WORLD_UNIT_SIZE + self.window_w / 2
    out.y = -self._tmp_project_point_vec3.y * WORLD_UNIT_SIZE + self.window_h / 2

  _axis_tmp1 = Vector2()
  _axis_tmp2 = Vector2()
  _axis_origin = Vector3((0, 0, 0))
  _axis_x = Vector3((1, 0, 0))
  _axis_y = Vector3((0, 1, 0))
  _axis_z = Vector3((0, 0, 1))

  _tmp_projected_path: List[Vector2] = []
  for _ in range(PARTICLE_PATH_LEN):
    _tmp_projected_path.append(Vector2())

  def update(self) -> None:
    if self.is_mouse_down:
      self.camera_rotation_x -= self.delta_mouse_y * CAMERA_ROTATION_SPEED
      self.camera_rotation_y -= self.delta_mouse_x * CAMERA_ROTATION_SPEED
      self.update_viewport_matrix()

  def render(self) -> None:
    surface = self.window_surface

    surface.fill(COLOR_BACKGROUND)  # type: ignore

    self.project_point(self._axis_origin, self._axis_tmp1)
    self.project_point(self._axis_x, self._axis_tmp2)
    pygame.draw.line(surface, COLOR_AXIS_X, self._axis_tmp1, self._axis_tmp2)
    self.project_point(self._axis_y, self._axis_tmp2)
    pygame.draw.line(surface, COLOR_AXIS_Y, self._axis_tmp1, self._axis_tmp2)
    self.project_point(self._axis_z, self._axis_tmp2)
    pygame.draw.line(surface, COLOR_AXIS_Z, self._axis_tmp1, self._axis_tmp2)

    if self.particle_paths_usage < PARTICLE_PATH_LEN:
      self.particle_paths_usage += 1

    self.prev_particle_paths_offset = self.particle_paths_offset
    self.particle_paths_offset += 1
    if self.particle_paths_offset >= self.particle_paths_usage:
      self.particle_paths_offset = 0

    for part_idx in range(PARTICLE_COUNT):
      part_off = part_idx * PARTICLE_PATH_LEN
      prev_last_point = self.particle_paths[part_off + self.prev_particle_paths_offset]
      last_point = self.particle_paths[part_off + self.particle_paths_offset]
      attractor_function(prev_last_point, PARTICLE_STEP, last_point)

      color = self.particle_colors[part_idx]

      proj_idx = 0
      for i in range(self.particle_paths_offset + 1, self.particle_paths_usage):
        self.project_point(self.particle_paths[part_off + i], self._tmp_projected_path[proj_idx])
        proj_idx += 1
      for i in range(0, self.particle_paths_offset + 1):
        self.project_point(self.particle_paths[part_off + i], self._tmp_projected_path[proj_idx])
        proj_idx += 1

      pygame.draw.lines(
        surface, color, False, self._tmp_projected_path[:self.particle_paths_usage]
      )


if __name__ == "__main__":
  sys.exit(LorenzAttractor().main(sys.argv))
