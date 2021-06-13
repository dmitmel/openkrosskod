#cython: language_level=3

# from cpython.mem cimport PyMem_Malloc, PyMem_Realloc, PyMem_Free
from cython cimport view
import random


def run_tournament(Py_ssize_t total_players, double CHAMPION_WIN_PROBABILITY, double REGULAR_PLAYER_WIN_PROBABILITY):
  # try:
  #   scores2 = <long*> PyMem_Malloc(total_players * sizeof(long))
  #   if not scores2:
  #     raise MemoryError()

  cdef view.array scores_arr
  cdef long[:] scores
  cdef Py_ssize_t player1, player2, player, best_player
  cdef long score, best_score
  cdef double win_probability, random_value
  cdef bint is_champion

  scores_arr = view.array(shape=(total_players,), itemsize=sizeof(long), format="l")
  scores = scores_arr
  scores[:] = 0

  for player2 in range(total_players):
    for player1 in range(0, player2):
      is_champion = player1 == 0
      win_probability = CHAMPION_WIN_PROBABILITY if is_champion else REGULAR_PLAYER_WIN_PROBABILITY
      random_value = random.random()
      scores[player1 if random_value < win_probability else player2] += 1

  best_score = -1
  best_player = -1
  for player in range(total_players):
    score = scores[player]
    if score > best_score:
      best_score = score
      best_player = player

  for player in range(total_players):
    score = scores[player]
    if score == best_score and player != best_player:
      return -1
  return best_player

  # finally:
  #  PyMem_Free(scores2)
