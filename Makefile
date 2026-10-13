.DEFAULT_GOAL := help
SHELL := /bin/bash

DATA_DIR ?= wow-data
WOW_DATA ?= $(abspath $(DATA_DIR)/Data)
GAME := target/release/warcraft-survivors

.PHONY: help build start native fetch-data clean

help: ## Show this help
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-14s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

fetch-data: ## Download game data, or use an existing WOW_DATA folder
	WOW_DATA_DIR="$(DATA_DIR)" ./scripts/fetch-data.sh

build: fetch-data ## Fetch missing data and build the native desktop game
	./scripts/native-build.sh

start: ## Launch the native desktop game (run make build first)
	@test -x "$(GAME)" || { echo "Not built yet; run: make build"; exit 1; }
	WOW_DATA="$(WOW_DATA)" "./$(GAME)"

native: build ## Build and launch the native desktop game
	$(MAKE) start

clean: ## Remove Rust build output; keep game data and local settings
	cargo clean
