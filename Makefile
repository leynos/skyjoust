.PHONY: help all clean test build release lint fmt check-fmt markdownlint nixie


TARGET ?= skyjoust

CARGO ?= cargo
BUILD_JOBS ?=
RUST_FLAGS ?=
RUST_FLAGS := -D warnings $(RUST_FLAGS)
RUSTDOC_FLAGS ?=
RUSTDOC_FLAGS := -D warnings $(RUSTDOC_FLAGS)
CARGO_FLAGS ?= --workspace --all-targets --all-features
DOC_FLAGS ?= --workspace --all-features --no-deps
CLIPPY_FLAGS ?= $(CARGO_FLAGS) -- $(RUST_FLAGS)
TEST_FLAGS ?= $(CARGO_FLAGS)
TEST_CMD := $(if $(shell $(CARGO) nextest --version 2>/dev/null),nextest run,test)
MDLINT ?= markdownlint-cli2
NIXIE ?= nixie

build: target/debug/$(TARGET) ## Build debug binary
release: target/release/$(TARGET) ## Build release binary

all: check-fmt lint test ## Perform a comprehensive check of code

clean: ## Remove build artifacts
	$(CARGO) clean

test: ## Run tests with warnings treated as errors
	RUSTFLAGS="$(RUST_FLAGS)" $(CARGO) $(TEST_CMD) $(TEST_FLAGS) $(BUILD_JOBS)
ifneq ($(TEST_CMD),test)
	@doc_test_log="$$(mktemp)"; \
	if RUSTFLAGS="$(RUST_FLAGS)" $(CARGO) test --doc --workspace --all-features 2> "$$doc_test_log"; then \
		rm -f "$$doc_test_log"; \
	elif grep -q "no library targets found" "$$doc_test_log"; then \
		cat "$$doc_test_log"; \
		rm -f "$$doc_test_log"; \
		echo "No library targets found; skipping doc tests."; \
	else \
		cat "$$doc_test_log"; \
		rm -f "$$doc_test_log"; \
		exit 1; \
	fi
endif

target/%/$(TARGET): ## Build binary in debug or release mode
	$(CARGO) build $(BUILD_JOBS) $(if $(findstring release,$(@)),--release) --bin $(TARGET)

lint: ## Run Clippy with warnings denied
	RUSTDOCFLAGS="$(RUSTDOC_FLAGS)" $(CARGO) doc $(DOC_FLAGS)
	$(CARGO) clippy $(CLIPPY_FLAGS)
	@command -v whitaker >/dev/null 2>&1 && \
		RUSTFLAGS="$(RUST_FLAGS)" whitaker --all -- $(CARGO_FLAGS) || \
		{ echo "whitaker not found on PATH; skipping whitaker lint. Install whitaker to run this check."; }

typecheck: ## Type-check without building
	RUSTFLAGS="$(RUST_FLAGS)" $(CARGO) check $(CARGO_FLAGS)

fmt: ## Format Rust and Markdown sources
	$(CARGO) +nightly fmt --all
	mdformat-all

check-fmt: ## Verify formatting
	$(CARGO) fmt --all -- --check

markdownlint: ## Lint Markdown files
	$(MDLINT) '**/*.md'

nixie: ## Validate Mermaid diagrams
	$(NIXIE) --no-sandbox

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) | \
	awk 'BEGIN {FS=":"; printf "Available targets:\n"} {printf "  %-20s %s\n", $$1, $$2}'
