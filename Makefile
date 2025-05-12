DOCKER_NAME := manta_hal

DOCKER_RUN_ARGS := run
DOCKER_RUN_ARGS += --rm
DOCKER_RUN_ARGS += -it
DOCKER_RUN_ARGS += -v $(PWD):/mnt
DOCKER_RUN_ARGS += -w /mnt
DOCKER_RUN_ARGS += --network=host
DOCKER_RUN_ARGS += $(DOCKER_NAME)
DOCKER_RUN_ARGS += bash

PHONY += build_docker
build_docker:
	docker build -t ${DOCKER_NAME} .

PHONY += docker
docker:
	docker $(DOCKER_RUN_ARGS)


PHONY += build
build:
	cargo build --all-features --target riscv64gc-unknown-none-elf
	cargo build --all-features --target loongarch64-unknown-none

PHONY += build-release
build-release:
	cargo build --release --all-features --target riscv64gc-unknown-none-elf
	cargo build --release --all-features --target loongarch64-unknown-none

PHONY += clippy
clippy:
	cargo clippy --all-features --target riscv64gc-unknown-none-elf
	cargo clippy --all-features --target loongarch64-unknown-none

PHONY += fmt
fmt:
	cargo fmt

PHONY += clean
clean:
	cargo clean