BINS := misc bootloader kernel
BUILD_CONTAINER = moratorium08/osmium:develop

build: $(BINS)
$(BINS):
	CARGO_HOME=`pwd`/.cargo make build -C $@
.PHONY: build $(BINS)

setup:
	./scripts/setup.sh

run:
	./scripts/run.sh

# build by using Docker
d_build:
	docker run -w $(PWD) -v $(PWD):$(PWD) -it ${BUILD_CONTAINER} make build
