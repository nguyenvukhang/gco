BUILD_DIR := target

current: test

configure:
	cmake -S . -B $(BUILD_DIR)

build:
	cmake --build $(BUILD_DIR)

install: build
	cmake --install $(BUILD_DIR)

run: install
	git -C /home/khang/repos/dwm nu checkout2 k

v: install
	cd /home/khang/repos/dwm && \
	valgrind --trace-children=yes --show-error-list=yes -- \
	git-checkout2 meme

test: install
	cargo test -- --test-threads=1

fmt:
	git ls-files '*.c' '*.h' | xargs clang-format -i
