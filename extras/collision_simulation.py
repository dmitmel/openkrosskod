#!/usr/bin/env python3

import math
import os
import random
import sys
from typing import List

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

  def add_force(self, force: Vector2) -> None:
    self.accel.update(force.x / self.mass, force.y / self.mass)


class CollisionSimulation(Visualization):

  def __init__(self) -> None:
    super().__init__()
    self.fixed_delta_time = 1.0 / 1000.0
    dt = self.fixed_delta_time

    # self.b1 = Body(
    #   pos=Vector2(0, 20),
    #   accel=Vector2(0, 0),
    #   mass=10,
    #   radius=20,
    #   color=Color(0xff, 0, 0),
    # )
    #
    # self.b2 = Body(
    #   pos=Vector2(300, 0),
    #   accel=Vector2(400 / dt),
    #   mass=10,
    #   radius=20,
    #   color=Color(0xff, 0, 0),
    # )
    #
    # self.bodies = [self.b1, self.b2]

    self.bodies: List[Body] = []
    for _ in range(20):
      self.bodies.append(
        Body(
          pos=Vector2(
            random.uniform(-300, 300),
            random.uniform(-300, 300),
          ),
          accel=Vector2(
            random.uniform(-100, 100) / dt,
            random.uniform(-100, 100) / dt,
          ),
          # mass=random.uniform(10, 100),
          mass=50,
          radius=random.uniform(20, 30),
          color=Color(0xff, 0, 0),
        )
      )

  def fixed_update(self) -> None:
    dt = self.fixed_delta_time

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
          body1.add_force(push_force)
          push_force.update(-push_force.x, -push_force.y)
          body2.add_force(push_force)

          # vel_angle = (body2.pos - body2.prev_pos).angle_to(body1.pos - body1.prev_pos)
          # if vel_angle > 180.0:
          #   vel_angle = 360.0 - vel_angle
          # elif vel_angle < -180:
          #   vel_angle = 360.0 + vel_angle
          # print("{:.6f} deg".format(vel_angle))

    box_w = self.window_w
    box_h = self.window_h

    kinetic_energy = 0

    tmp_prev_pos = Vector2()
    tmp_vel = Vector2()
    for body in self.bodies:
      tmp_prev_pos.update(body.pos)
      tmp_vel.update(body.pos.x - body.prev_pos.x, body.pos.y - body.prev_pos.y)

      if body.pos.x + body.radius >= box_w / 2:
        tmp_vel.x = math.copysign(tmp_vel.x, -1)
      if body.pos.x - body.radius <= -box_w / 2:
        tmp_vel.x = math.copysign(tmp_vel.x, 1)
      if body.pos.y + body.radius >= box_h / 2:
        tmp_vel.y = math.copysign(tmp_vel.y, -1)
      if body.pos.y - body.radius <= -box_h / 2:
        tmp_vel.y = math.copysign(tmp_vel.y, 1)

      kinetic_energy += body.mass + tmp_vel.length_squared()

      # <https://en.wikipedia.org/wiki/Leapfrog_integration>
      body.pos += tmp_vel + body.accel * dt * dt
      body.prev_pos.update(tmp_prev_pos)
      body.accel.update(0, 0)

    print(kinetic_energy)

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
