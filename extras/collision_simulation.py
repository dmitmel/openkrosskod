#!/usr/bin/env python3

import math
import os
import sys

os.environ["PYGAME_HIDE_SUPPORT_PROMPT"] = "1"

import pygame.draw
from pygame.color import Color
from pygame.math import Vector2
from visualization_core import Visualization

COLLISION_PUSH_COEFF = 1e4


class Body:

  def __init__(
    self, pos: Vector2, accel: Vector2, mass: float, radius: float, color: Color
  ) -> None:
    self.pos = pos
    self.prev_pos = Vector2(pos)
    self.accel = accel
    self.mass = mass
    self.radius = radius
    self.color = color


class CollisionSimulation(Visualization):

  def __init__(self) -> None:
    super().__init__()
    self.fixed_delta_time = 1.0 / 10000.0

    self.b1 = Body(
      pos=Vector2(0, 20),
      accel=Vector2(0, 0),
      mass=10,
      radius=20,
      color=Color(0xff, 0, 0),
    )

    dt = self.fixed_delta_time
    self.b2 = Body(
      pos=Vector2(250, 0),
      accel=Vector2(-400 / dt, 0),
      mass=10,
      radius=20,
      color=Color(0xff, 0, 0),
    )

    self.bodies = [self.b1, self.b2]

    # self.bodies: List[Body] = []
    # for _ in range(10):
    #   self.bodies.append(
    #     Body(
    #       pos=Vector2(random.uniform(-400, 400), random.uniform(-400, 400)),
    #       vel=Vector2(random.uniform(-50, 50), random.uniform(-50, 50)),
    #       mass=random.uniform(1, 100),
    #       radius=random.uniform(10, 40),
    #       color=Color(0xff, 0, 0),
    #     )
    #   )

  def fixed_update(self) -> None:
    for body1_idx in range(len(self.bodies)):
      for body2_idx in range(0, body1_idx):
        if body1_idx == body2_idx:
          continue
        body1 = self.bodies[body1_idx]
        body2 = self.bodies[body2_idx]

        sqr_dist = body1.pos.distance_squared_to(body2.pos)
        contact_dist = body1.radius + body2.radius
        if sqr_dist <= contact_dist * contact_dist:
          surface_dist = math.sqrt(sqr_dist) - contact_dist
          push_force = body2.pos - body1.pos
          push_force.scale_to_length(COLLISION_PUSH_COEFF * surface_dist)

          body1.accel += push_force / body1.mass
          body2.accel -= push_force / body2.mass

          vel_angle = (body2.pos - body2.prev_pos).angle_to(body1.pos - body1.prev_pos)
          if vel_angle > 180.0:
            vel_angle = 360.0 - vel_angle
          elif vel_angle < -180:
            vel_angle = 360.0 + vel_angle
          print("{:.6f} deg".format(vel_angle))

    tmp_prev_pos = Vector2()
    for body in self.bodies:
      tmp_prev_pos.update(body.pos)
      # <https://en.wikipedia.org/wiki/Leapfrog_integration>
      body.pos += body.pos - body.prev_pos + body.accel * self.fixed_delta_time ** 2
      body.prev_pos.update(tmp_prev_pos)
      body.accel.update(0, 0)

  def render(self) -> None:
    surface = self.window_surface
    tmp_vel = Vector2()
    for body in self.bodies:
      body_x = self.window_w / 2 + body.pos.x
      body_y = self.window_h / 2 + body.pos.y
      tmp_vel.update(body.pos.x - body.prev_pos.x, body.pos.y - body.prev_pos.y)
      try:
        tmp_vel.scale_to_length(100.0)
      except ValueError:
        tmp_vel.update(0, 0)
      pygame.draw.circle(surface, body.color, (body_x, body_y), body.radius)
      pygame.draw.line(
        surface, Color(0x00ff00ff), (body_x, body_y), (body_x + tmp_vel.x, body_y + tmp_vel.y)
      )


if __name__ == "__main__":
  sys.exit(CollisionSimulation().main(sys.argv))
