BUILD_DIR := target

MAKEFILE_PATH := $(realpath $(lastword $(MAKEFILE_LIST)))
MAKEFILE_DIR := $(dir $(MAKEFILE_PATH))

VALGRIND_LOG := $(MAKEFILE_DIR)/valgrind-log-%p.txt

VALGRIND_FLAGS :=
VALGRIND_FLAGS += --trace-children=yes
VALGRIND_FLAGS += --show-error-list=yes
# VALGRIND_FLAGS += --leak-check=full
# VALGRIND_FLAGS += --show-leak-kinds=all
VALGRIND_FLAGS += --log-file=$(VALGRIND_LOG)
# VALGRIND_FLAGS += --xml=yes
# VALGRIND_FLAGS += --xml-file=valgrind.xml

current: v test

configure:
	cmake -DCMAKE_BUILD_TYPE=Release -S . -B $(BUILD_DIR)

configure-debug:
	cmake -DCMAKE_BUILD_TYPE=Debug -S . -B $(BUILD_DIR)

build:
	cmake --build $(BUILD_DIR)

install: build
	cmake --install $(BUILD_DIR)

v: configure install
	-@rm -f valgrind-log*.txt
	-cd /home/khang/repos/dwm && \
	valgrind $(VALGRIND_FLAGS) -- git-checkout2 snoop

test: configure install
	cargo test -- --test-threads=1

fmt:
	git ls-files '*.c' '*.h' | xargs clang-format -i
