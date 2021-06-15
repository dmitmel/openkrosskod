#!/usr/bin/env python3

import sys

# <https://github.com/python/cpython/blob/v3.9.5/Modules/main.c#L195>
# <https://github.com/python/cpython/blob/v3.9.5/Python/sysmodule.c#L2694-L2695>
# <https://github.com/python/cpython/blob/v3.9.5/Python/sysmodule.c#L2706-L2707>
print(f"Python {sys.version} on {sys.platform}")

import os
from distutils.util import get_platform

# WHY THE HELL IS THIS EVEN REQUIRED????
# <https://github.com/python/cpython/blob/9d8dd8f08aae4ad6e73a9322a4e9dee965afebbc/Lib/distutils/command/build.py#L84>
sys.path.append(
  os.path.join(
    os.path.dirname(__file__), "build",
    "lib.{}-{}.{}".format(get_platform(), *sys.version_info[:2])
  )
)

import csv
import multiprocessing
import random
from datetime import datetime
from typing import Callable, Generator, Iterable, List, Optional, Tuple, TypeVar

try:
  import matplotlib.pyplot as plt  # type: ignore
except ImportError:
  plt = None

try:
  from tqdm import tqdm  # type: ignore
except ImportError:
  tqdm = None

try:
  from mega_tournament_ultra_fast import run_tournament as run_tournament_fast  # type: ignore

  # import pyximport  # type: ignore
  # pyximport.install()  # type: ignore
  # from mega_tournament_fast import run_tournament as run_tournament_fast  # type: ignore
except ImportError:
  run_tournament_fast = None

CHAMPION_WIN_PROBABILITY = 0.75
REGULAR_PLAYER_WIN_PROBABILITY = 0.5
ITERATIONS_PER_PLAYER_NUMBER = 1000000
MIN_PLAYERS_NUMBER, MAX_PLAYERS_NUMBER = 2, 100
MP_THREADS = 8
MP_CHUNK_SIZE = 1000
USE_FAST_IMPLEMENTATION = True

_T = TypeVar("_T")


def tqdm_wrapper(iterable: Iterable[_T],
                 total: Optional[int] = None) -> Iterable[_T]:  # noqa: F811
  if tqdm is not None:
    return tqdm(iterable, total=total)
  else:
    return iterable


def main(argv: List[str]) -> int:
  champion_wins_per_player_number: List[int] = [0] * (
    MAX_PLAYERS_NUMBER - MIN_PLAYERS_NUMBER + 1  #
  )

  for total_players, _, best_player in simulate_or_read_results(
    argv[1] if len(argv) > 1 else None,
    len(champion_wins_per_player_number) * ITERATIONS_PER_PLAYER_NUMBER,
  ):
    if best_player == 0:
      champion_wins_per_player_number[total_players - MIN_PLAYERS_NUMBER] += 1

  champion_win_probabilities = [0.0] * len(champion_wins_per_player_number)
  for total_players, champion_wins in enumerate(champion_wins_per_player_number):
    total_players += MIN_PLAYERS_NUMBER
    champion_win_probabilities[total_players - MIN_PLAYERS_NUMBER] = (
      champion_wins / ITERATIONS_PER_PLAYER_NUMBER  #
    )

  if plt is not None:
    plt.plot(range(MIN_PLAYERS_NUMBER, MAX_PLAYERS_NUMBER + 1), champion_win_probabilities)
    plt.xlabel("Total number of players")
    plt.ylabel("Win probability of the champion")
    plt.show()
  else:
    for x, y in zip(range(MIN_PLAYERS_NUMBER, MAX_PLAYERS_NUMBER + 1), champion_win_probabilities):
      print(" {:4}  {}".format(x, y))

  return 0


def simulate_or_read_results(
  table_file_name: Optional[str],
  total: int,
) -> Generator[Tuple[int, int, int], None, None]:

  if table_file_name is None:
    table_file_name = "tournaments_{}.csv".format(datetime.now().strftime("%Y-%m-%d-%H-%M-%S"))

    with multiprocessing.Pool(MP_THREADS) as pool, open(table_file_name, "w") as table_file:
      table_writer = csv.writer(table_file)
      for task_result in tqdm_wrapper(
        pool.imap_unordered(mp_task_executor, mp_task_generator(), chunksize=MP_CHUNK_SIZE),
        total=total,
      ):
        table_writer.writerow(task_result)
        yield task_result

  else:
    with open(table_file_name, "r") as table_file:
      table_reader = csv.reader(table_file)
      for total_players, iteration, best_player in tqdm_wrapper(table_reader, total=total):
        yield int(total_players), int(iteration), int(best_player)


def mp_task_generator() -> Generator[Tuple[int, int], None, None]:
  for total_players in range(MIN_PLAYERS_NUMBER, MAX_PLAYERS_NUMBER + 1):
    for iteration in range(ITERATIONS_PER_PLAYER_NUMBER):
      yield total_players, iteration


def mp_task_executor(task: Tuple[int, int]) -> Tuple[int, int, int]:
  total_players, iteration = task
  impl: Callable[[int, float, float], int] = run_tournament_slow
  if USE_FAST_IMPLEMENTATION and run_tournament_fast is not None:
    impl = run_tournament_fast
  best_player = impl(total_players, CHAMPION_WIN_PROBABILITY, REGULAR_PLAYER_WIN_PROBABILITY)
  return total_players, iteration, best_player


def run_tournament_slow(
  total_players: int, champion_win_probability: float, regular_player_win_probability: float
) -> int:
  scores = [0] * total_players

  for player2 in range(total_players):
    for player1 in range(0, player2):
      is_champion = player1 == 0
      win_probability = champion_win_probability if is_champion else regular_player_win_probability
      scores[player1 if random.random() < win_probability else player2] += 1

  best_score = -1
  best_player = -1
  for player, score in enumerate(scores):
    if score > best_score:
      best_score = score
      best_player = player

  for player, score in enumerate(scores):
    if score == best_score and player != best_player:
      return -1  # there is more than one best player
  return best_player


if __name__ == "__main__":
  sys.exit(main(sys.argv))
