#define PY_SSIZE_T_CLEAN
#include <Python.h>
#include <cstddef>
#include <cstdint>
#include <random>
#include <vector>

namespace mega_tournament_ultra_fast {

PyObject *run_tournament(PyObject *self, PyObject *args) {
  size_t total_players = 0;
  double champion_win_probability = 0;
  double regular_player_win_probability = 0;

  if (!PyArg_ParseTuple(args, "ldd", &total_players, &champion_win_probability,
                        &regular_player_win_probability)) {
    return NULL;
  }

  static thread_local bool rng_initialized = false;
  static thread_local std::default_random_engine rng_engine;
  if (!rng_initialized) {
    rng_initialized = true;
    std::random_device device;
    rng_engine.seed(device());
  }

  std::uniform_real_distribution<double> distribution(0.0, 1.0);

  std::vector<size_t> scores(total_players, 0);

  for (size_t player2 = 0; player2 < total_players; player2++) {
    for (size_t player1 = 0; player1 < player2; player1++) {
      bool is_champion = player1 == 0;
      double win_probability = is_champion ? champion_win_probability
                                           : regular_player_win_probability;
      size_t winner =
          distribution(rng_engine) < win_probability ? player1 : player2;
      scores.at(winner)++;
    }
  }

  size_t best_score = 0;
  size_t best_player = 0;
  for (size_t player = 0; player < total_players; player++) {
    size_t score = scores.at(player);
    if (score > best_score) {
      best_score = score;
      best_player = player;
    }
  }

  for (size_t player = 0; player < total_players; player++) {
    size_t score = scores.at(player);
    if (score == best_score && player != best_player) {
      return PyLong_FromLong(-1);
    }
  }

  return PyLong_FromSsize_t(best_player);
}

static PyMethodDef py_methods[] = {
    {"run_tournament", run_tournament, METH_VARARGS, nullptr},
    {nullptr, nullptr, 0, nullptr},
};

static PyModuleDef py_module = {
    PyModuleDef_HEAD_INIT, "mega_tournament_ultra_fast", nullptr, 0, py_methods,
};

} // namespace mega_tournament_ultra_fast

PyMODINIT_FUNC PyInit_mega_tournament_ultra_fast() {
  return PyModule_Create(&mega_tournament_ultra_fast::py_module);
}
