.PHONY: help build build-release build-full-release build-minimal-release \
        test clean fmt lint check bench doc ci

CARGO        ?= cargo
BUILD_DIR    := target
PROFILE      := debug
BINARY_NAME  := vk
MIN_BINARY   := vk-ci

ifeq ($(OS),Windows_NT)
    EXT := .exe
else
    EXT :=
endif

BIN_PATH        := $(BUILD_DIR)/$(PROFILE)/$(BINARY_NAME)$(EXT)
MIN_BIN_PATH    := $(BUILD_DIR)/release/$(MIN_BINARY)$(EXT)

help: ## Show this help message
	@echo "Usage: make <target>"
	@echo ""
	@echo "Available targets:"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-22s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## Build debug binary
	@echo "Building $(BINARY_NAME) (debug)..."
	@$(CARGO) build
	@echo "✓ Built at $(BIN_PATH)"

build-release: ## Build release binary
	@echo "Building $(BINARY_NAME) (release)..."
	@$(CARGO) build --release --bin $(BINARY_NAME)
	@strip $(BUILD_DIR)/release/$(BINARY_NAME)$(EXT) 2>/dev/null || true
	@echo "✓ Built at $(BUILD_DIR)/release/$(BINARY_NAME)$(EXT)"

build-full-release: ## Build release with full features
	@echo "Building $(BINARY_NAME) (release + full)..."
	@$(CARGO) build --release --bin $(BINARY_NAME) --features full
	@strip $(BUILD_DIR)/release/$(BINARY_NAME)$(EXT) 2>/dev/null || true
	@echo "✓ Built at $(BUILD_DIR)/release/$(BINARY_NAME)$(EXT)"

build-minimal-release: ## Build minimal CI binary
	@echo "Building $(MIN_BINARY) (minimal release)..."
	@$(CARGO) build --release --bin $(MIN_BINARY) --no-default-features --features minimal
	@strip $(MIN_BIN_PATH) 2>/dev/null || true
	@echo "✓ Built at $(MIN_BIN_PATH)"

test: ## Run tests
	@$(CARGO) test

fmt: ## Format code
	@$(CARGO) fmt --all

lint: ## Run clippy with warnings denied
	@$(CARGO) clippy --all-targets --all-features -- -D warnings

check: ## Run cargo check
	@$(CARGO) check

bench: ## Run benchmarks
	@$(CARGO) bench

doc: ## Generate documentation
	@$(CARGO) doc --no-deps --open

# ===== CI =====
ci: fmt lint test ## Run CI checks locally
	@echo "✓ CI checks passed"

# ===== Clean =====
clean: ## Clean build artifacts
	@$(CARGO) clean
	@echo "✓ Cleaned"
