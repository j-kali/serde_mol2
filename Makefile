.PHONY: all
all:
	@echo "Run my targets individually!"

.PHONY: env
.ONESHELL:
env:
	test -d env || python3 -m venv env
	. env/bin/activate
	pip install maturin

.PHONY: develop
.ONESHELL:
develop: env
	. env/bin/activate
	maturin develop

.PHONY: build
.ONESHELL:
build: env
	. env/bin/activate
	maturin build

.PHONY: dist
.ONESHELL:
dist: env
	. env/bin/activate
	docker run --rm -v $(shell pwd):/io konstin2/maturin:v0.12.6 build --release --strip -b bin
	docker run --rm -v $(shell pwd):/io konstin2/maturin:v0.12.6 build --release --strip
	./join-whl.sh

target/release/serde-mol2:
	cargo build --release

.PHONY: test
.ONESHELL:
test: target/release/serde-mol2 develop
	. env/bin/activate
	./test.sh
