.PHONY: help all clean test build release lint fmt check-fmt markdownlint nixie \
	check-diagrams generate-state-graphs check-state-graphs spelling


TARGET ?= skyjoust
PATH := $(HOME)/.cargo/bin:$(HOME)/.bun/bin:$(PATH)

CARGO ?= cargo
BUILD_JOBS ?=
RUST_FLAGS ?=
RUST_FLAGS := -D warnings $(RUST_FLAGS)
# The build standard: every `rustflags` source in `.cargo/config.toml` carries
# the parallel frontend, and the Linux source adds mold. Assigning `RUSTFLAGS`
# replaces those sources outright, so the gate targets restate the flags here.
# The recipes add them to any inherited `RUSTFLAGS` (setup-rust exports one in
# CI) instead of replacing it. `make release` adds neither standard flag; coverage takes
# neither only when its caller exports `RUSTFLAGS`, as setup-rust does in CI.
STANDARD_THREADS_FLAG ?= -Zthreads=8
STANDARD_MOLD_FLAG ?= -Clink-arg=-fuse-ld=mold
BUILD_HOST_OS ?= $(shell uname -s)
# mold is added only when the machine doing the build is Linux (only Make can
# tell whether it has mold) and the compilation target is Linux too, which is
# the host unless `CARGO_BUILD_TARGET` names another triple. Android triples
# contain `-linux-` but report `target_os = "android"`, so they are not Linux.
STANDARD_TARGET_IS_LINUX = $(if $(CARGO_BUILD_TARGET),$(or $(filter host-tuple,$(CARGO_BUILD_TARGET)),$(and $(findstring -linux-,$(CARGO_BUILD_TARGET)),$(if $(findstring -android,$(CARGO_BUILD_TARGET)),,yes))),yes)
STANDARD_RUSTFLAGS = $(STANDARD_THREADS_FLAG)$(if $(filter Linux,$(BUILD_HOST_OS)),$(if $(STANDARD_TARGET_IS_LINUX), $(STANDARD_MOLD_FLAG)))
# Release builds add neither standard flag: assigning `RUSTFLAGS`, even to an
# empty inherited value, displaces every `rustflags` source in the
# configuration, and a caller's own value passes through untouched.
RELEASE_RUSTFLAGS = RUSTFLAGS="$${RUSTFLAGS-}"
# Debug builds keep a caller's exported flags and add the standard ones,
# since an inherited `RUSTFLAGS` would otherwise displace the configuration.
DEBUG_RUSTFLAGS = RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(STANDARD_RUSTFLAGS)"
# Gate targets also deny warnings, so they compose the caller's flags, the
# warning policy and the standard flags in one place.
GATE_RUSTFLAGS = RUSTFLAGS="$${RUSTFLAGS:+$$RUSTFLAGS }$(RUST_FLAGS) $(STANDARD_RUSTFLAGS)"
# Whitaker's Dylint driver runs on its own pinned toolchain, which need not
# carry the Cranelift component the development profile selects, so its
# check builds take LLVM. Dylint builds its driver in a crate outside this
# repository, which the `[unstable]` table does not reach, so the override
# also enables the unstable key there.
WHITAKER_CODEGEN_BACKEND ?= llvm
RUSTDOC_FLAGS ?=
RUSTDOC_FLAGS := -D warnings $(RUSTDOC_FLAGS)
CARGO_FLAGS ?= --workspace --all-targets --all-features
DOC_FLAGS ?= --workspace --all-features --no-deps
CLIPPY_FLAGS ?= $(CARGO_FLAGS) -- $(RUST_FLAGS)
TEST_FLAGS ?= $(CARGO_FLAGS)
TEST_CMD := $(if $(shell $(CARGO) nextest --version 2>/dev/null),nextest run,test)
# dev-fast is the standard development path: the build, test, lint, and
# typecheck targets below pass this to every cargo invocation. Defined here
# so those targets (which sit above the appended Wave 1 block) can see it;
# the identical assignment in that block is a harmless no-op. Coverage,
# release, verification, and audit targets must never receive this flag.
DEV_FAST_CONFIG ?= tools/dev-fast/config.toml
MDLINT ?= markdownlint-cli2
# `make fmt` and `make check-fmt` call mdtablefix directly. `--git` selects the
# Markdown files Git tracks and `--include-untracked` adds the untracked files
# Git does not ignore, so a new document is formatted before it is staged.
# Both modes need mdtablefix 0.6.0 or later; CI pins the version at the
# install-mdtablefix step.
MDTABLEFIX ?= mdtablefix
MDTABLEFIX_SELECT = --git --include-untracked
MDTABLEFIX_RULES = --wrap --renumber --breaks --ellipsis --fences
NIXIE ?= nixie
WHITAKER ?= whitaker
DIAGRAM_DIFF_BASE ?= origin/main
DIAGRAM_PATHS := docs/*.dot docs/*.svg docs/*.md
UV ?= uv
UV_ENV = UV_CACHE_DIR=.uv-cache UV_TOOL_DIR=.uv-tools
TYPOS_CONFIG_BUILDER_VERSION ?= v0.1.3
TYPOS_CONFIG_BUILDER = $(UV_ENV) $(UV) tool run --python 3.14 --from \
	"git+https://github.com/leynos/typos-config-builder.git@$(TYPOS_CONFIG_BUILDER_VERSION)" \
	typos-config-builder

build: target/debug/$(TARGET) ## Build debug binary
release: target/release/$(TARGET) ## Build release binary

all: check-fmt check-state-graphs markdownlint check-diagrams lint test spelling ## Perform a comprehensive check of code

clean: ## Remove build artefacts
	$(CARGO) clean

test: ## Run tests with warnings treated as errors
	$(GATE_RUSTFLAGS) $(CARGO) --config "$(DEV_FAST_CONFIG)" $(TEST_CMD) $(TEST_FLAGS) $(BUILD_JOBS)
	@# Doctests run whichever runner is selected: `--all-targets` excludes them,
	@# so the plain `cargo test` branch needs the explicit run too.
	@doc_test_log="$$(mktemp)"; \
	if $(GATE_RUSTFLAGS) $(CARGO) --config "$(DEV_FAST_CONFIG)" test --doc --workspace --all-features 2> "$$doc_test_log"; then \
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

target/%/$(TARGET): ## Build binary in debug or release mode
	$(if $(findstring release,$(@)),$(RELEASE_RUSTFLAGS),$(DEBUG_RUSTFLAGS)) $(CARGO) build $(BUILD_JOBS) $(if $(findstring release,$(@)),--release,--config "$(DEV_FAST_CONFIG)") --bin $(TARGET)

lint: ## Run Clippy and the Whitaker Dylint suite with warnings denied
	$(GATE_RUSTFLAGS) RUSTDOCFLAGS="$(RUSTDOC_FLAGS)" $(CARGO) --config "$(DEV_FAST_CONFIG)" doc $(DOC_FLAGS)
	$(GATE_RUSTFLAGS) $(CARGO) --config "$(DEV_FAST_CONFIG)" clippy $(CLIPPY_FLAGS)
	@# `if` rather than `&& ... ||`, so a failing Whitaker run fails the target
	@# instead of falling through to the not-installed message.
	@if command -v $(WHITAKER) >/dev/null 2>&1; then \
		CARGO_UNSTABLE_CODEGEN_BACKEND=true CARGO_PROFILE_DEV_CODEGEN_BACKEND=$(WHITAKER_CODEGEN_BACKEND) $(GATE_RUSTFLAGS) $(WHITAKER) --all -- $(CARGO_FLAGS); \
	else \
		echo "whitaker not found on PATH; skipping whitaker lint. Install whitaker to run this check."; \
	fi

typecheck: ## Type-check without building
	$(GATE_RUSTFLAGS) $(CARGO) --config "$(DEV_FAST_CONFIG)" check $(CARGO_FLAGS)

fmt: ## Format Rust and Markdown sources
	$(CARGO) +nightly fmt --all
	$(MDTABLEFIX) --in-place $(MDTABLEFIX_SELECT) $(MDTABLEFIX_RULES)
	@unset FORCE_COLOR; $(MDLINT) --fix "**/*.md"

check-fmt: ## Verify formatting
	$(CARGO) fmt --all -- --check
	$(MDTABLEFIX) --check $(MDTABLEFIX_SELECT) $(MDTABLEFIX_RULES)

markdownlint: spelling ## Lint Markdown files and enforce spelling
	$(MDLINT) '**/*.md'

spelling: ## Enforce en-GB-oxendict spelling
	$(TYPOS_CONFIG_BUILDER) gate --repository .

nixie: ## Validate Mermaid diagrams
	$(NIXIE) --no-sandbox

check-diagrams: ## Validate diagrams when documentation diagrams changed
	@if git rev-parse --verify "$(DIAGRAM_DIFF_BASE)" >/dev/null 2>&1 && \
		test -z "$$(git diff --name-only "$(DIAGRAM_DIFF_BASE)" -- $(DIAGRAM_PATHS))"; then \
		echo "No documentation diagram changes detected; skipping nixie."; \
	else \
		$(MAKE) nixie; \
	fi

generate-state-graphs: ## Generate JSON state graph bundle from YAML
	python3 scripts/generate-state-graphs-json.py docs/skyjoust-state-graphs.yaml docs/skyjoust-state-graphs.json

check-state-graphs: check-diagrams ## Verify JSON state graph bundle matches YAML
	@tmp="$$(mktemp)"; \
	python3 scripts/generate-state-graphs-json.py docs/skyjoust-state-graphs.yaml "$$tmp"; \
	if cmp -s "$$tmp" docs/skyjoust-state-graphs.json; then \
		rm -f "$$tmp"; \
	else \
		echo "docs/skyjoust-state-graphs.json is stale. Run make generate-state-graphs."; \
		diff -u docs/skyjoust-state-graphs.json "$$tmp"; \
		rm -f "$$tmp"; \
		exit 1; \
	fi

help: ## Show available targets
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) | \
	awk 'BEGIN {FS=":"; printf "Available targets:\n"} {printf "  %-20s %s\n", $$1, $$2}'

# Opt-in accelerated debug builds (Cranelift + mold); requires a nightly
# toolchain. See AGENTS.md and tools/dev-fast/config.toml.
DEV_FAST_CONFIG ?= tools/dev-fast/config.toml

.PHONY: dev-build dev-test
dev-build: ## Build debug binaries with Cranelift and mold
	$(CARGO) --config "$(DEV_FAST_CONFIG)" build

dev-test: ## Run tests with Cranelift and mold
	$(CARGO) --config "$(DEV_FAST_CONFIG)" test
