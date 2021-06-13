from distutils.core import setup, Extension

setup(
  name="openkrosskod_extras",
  version="0.0.0",
  ext_modules=[
    Extension("mega_tournament_ultra_fast", sources=["mega_tournament_ultra_fast.cpp"]),
  ]
)
