#define PY_SSIZE_T_CLEAN
#include <Python.h>
#include <cstddef>
#include <cstdint>
#include <iostream>
#include <random>
#include <vector>

namespace mega_tournament_ultra_fast {

#ifdef METH_FASTCALL
// FASTCALL doesn't make the function faster in my instance, so it is disabled
// #define mega_tournament_use_fastcall
#endif

PyObject *run_tournament(PyObject *self,
#ifdef mega_tournament_use_fastcall
                         PyObject *const *args, Py_ssize_t nargs
#else
                         PyObject *args
#endif
) {
  size_t total_players = 0;
  double champion_win_probability = 0;
  double regular_player_win_probability = 0;

#ifdef mega_tournament_use_fastcall

  const Py_ssize_t required_nargs = 3;
  if (nargs != required_nargs) {
    // <https://github.com/python/cpython/blob/v3.9.5/Python/getargs.c#L374-L382>
    PyErr_Format(PyExc_TypeError,
                 "function takes exactly %d argument%s (%zd given)",
                 required_nargs, required_nargs == 1 ? "" : "s", nargs);
    return nullptr;
  }

  total_players = PyLong_AsSize_t(args[0]);
  if (PyErr_Occurred()) {
    return nullptr;
  }
  champion_win_probability = PyFloat_AsDouble(args[1]);
  if (PyErr_Occurred()) {
    return nullptr;
  }
  regular_player_win_probability = PyFloat_AsDouble(args[2]);
  if (PyErr_Occurred()) {
    return nullptr;
  }

#else
  if (!PyArg_ParseTuple(args, "ldd", &total_players, &champion_win_probability,
                        &regular_player_win_probability)) {
    return nullptr;
  }
#endif

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

#ifdef mega_tournament_use_fastcall
_PyCFunctionFast _run_tournament_type_assert = run_tournament;
#endif

static PyMethodDef py_methods[] = {
    {"run_tournament", (PyCFunction)run_tournament,
#ifdef mega_tournament_use_fastcall
     METH_FASTCALL,
#else
     METH_VARARGS,
#endif
     nullptr},
    {nullptr, nullptr, 0, nullptr},
};

static PyModuleDef py_module = {
    PyModuleDef_HEAD_INIT, "mega_tournament_ultra_fast", nullptr, 0, py_methods,
};

} // namespace mega_tournament_ultra_fast

PyMODINIT_FUNC PyInit_mega_tournament_ultra_fast() {
  return PyModule_Create(&mega_tournament_ultra_fast::py_module);
}
