TEST_ROOT = $(PWD)/test

BUILD_PROFILE ?= debug
BUILD_ROOT     = $(PWD)/target/$(BUILD_PROFILE)
PRODUCT        = $(BUILD_ROOT)/nexus

.PHONY: build
build:
	cargo build

.PHONY: test-unit
test-unit:
	cargo test

.PHONY: test-integrate
test-integrate: TEST_PKGS=tickets notes
test-integrate: export NEXUS=$(PRODUCT)
test-integrate: build
	@for test in $(TEST_PKGS); do \
	  "$(TEST_ROOT)/bin/$${test}.sh"; \
	done

.PHONY: test
test:
	cargo test
