import os
from abc import ABCMeta
from typing import List

import pygame.constants
import pygame.display
import pygame.draw
import pygame.event
import pygame.surface
import pygame.time
from pygame import init as pygame_init
from pygame import quit as pygame_quit
from pygame.color import Color
from pygame.math import Vector3

__all__ = ["Visualization", "WindowOptions", "Matrix4"]


class Visualization(metaclass=ABCMeta):

  def __init__(self) -> None:
    self.window_opts = WindowOptions()
    self.window_surface: pygame.surface.Surface
    self.window_w = 0
    self.window_h = 0
    self.is_running = True

    self.primary_clock: pygame.time.Clock
    self.target_fps = 60
    self.time = 0
    self.delta_time = 0
    self.fixed_time = 0
    self.fixed_delta_time = 1.0 / 120.0
    self.fixed_update_time_accumulator = 0.0

    self.is_mouse_down = False
    self.mouse_x = 0
    self.mouse_y = 0
    self.prev_mouse_x = 0
    self.prev_mouse_y = 0
    self.delta_mouse_x = 0
    self.delta_mouse_y = 0

    self.background_color = Color(0x101010ff)

  def main(self, argv: List[str]) -> int:
    try:
      pygame_init()

      self.window_surface = pygame.display.set_mode(
        (self.window_opts.w, self.window_opts.h),
        self.window_opts.flags,
        vsync=int(self.window_opts.vsync),
      )
      pygame.display.set_caption(self.window_opts.caption)
      self.window_w, self.window_h = self.window_surface.get_size()

      self.primary_clock = pygame.time.Clock()

      self.prestart()

      while self.is_running:
        self.delta_time = self.primary_clock.tick(self.target_fps) / 1000.0
        self.time += self.delta_time

        self.fixed_update_time_accumulator += self.delta_time
        while self.fixed_update_time_accumulator >= self.fixed_delta_time:
          self.fixed_time += self.fixed_delta_time
          self.fixed_update()
          self.fixed_update_time_accumulator -= self.fixed_delta_time

        self._process_events()
        self.update()

        self.window_surface.fill((0, 0, 0))  # type: ignore
        self.render()

        pygame.display.flip()

    finally:
      self.shutdown()

      pygame_quit()

    return 0

  def _process_events(self) -> None:
    self.prev_mouse_x = self.mouse_x
    self.prev_mouse_y = self.mouse_y

    for event in pygame.event.get():
      if event.type == pygame.constants.QUIT or (
        event.type == pygame.constants.KEYDOWN and event.key == pygame.constants.K_q
      ):
        self.is_running = False
      elif event.type == pygame.constants.VIDEORESIZE:
        self.window_w, self.window_h = self.window_surface.get_size()
      elif event.type == pygame.constants.MOUSEBUTTONDOWN:
        self.is_mouse_down = True
        self.mouse_x, self.mouse_y = event.pos
      elif event.type == pygame.constants.MOUSEBUTTONUP:
        self.is_mouse_down = False
        self.mouse_x, self.mouse_y = event.pos
      elif event.type == pygame.constants.MOUSEMOTION:
        self.mouse_x, self.mouse_y = event.pos
      elif event.type == pygame.constants.WINDOWLEAVE:
        self.is_mouse_down = False

    self.delta_mouse_x = self.mouse_x - self.prev_mouse_x
    self.delta_mouse_y = self.mouse_y - self.prev_mouse_y

  def prestart(self) -> None:
    pass

  def fixed_update(self) -> None:
    pass

  def update(self) -> None:
    pass

  def render(self) -> None:
    pass

  def shutdown(self) -> None:
    pass


class WindowOptions:

  def __init__(
    self,
    w: int = 800,
    h: int = 800,
    flags: int = pygame.constants.RESIZABLE,
    vsync: bool = True,
    caption: str = os.path.basename(__file__),
  ) -> None:
    self.w = w
    self.h = h
    self.flags = flags
    self.vsync = vsync
    self.caption = caption


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
