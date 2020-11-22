# https://tech.davis-hansson.com/p/make/
SHELL := bash
.ONESHELL:
.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
MAKEFLAGS += --warn-undefined-variables
MAKEFLAGS += --no-builtin-rules

.PHONY: all clean __cargo_build__

all: __cargo_build__ openkrosskod.renderdoc.cap

__cargo_build__:
	cargo build --release --workspace

clean:
	cargo clean

openkrosskod.renderdoc.cap: scripts/generate_renderdoc_capture_settings.py
	$< > $@
