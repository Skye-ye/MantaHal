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
