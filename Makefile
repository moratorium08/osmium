BINS := misc bootloader kernel

build: $(BINS)
$(BINS):
	make build -C $@
.PHONY: build $(BINS)

setup:
	./scripts/setup.sh

run:
	./scripts/run.sh

