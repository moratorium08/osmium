BINS := misc bootloader kernel

all: $(BINS)
$(BINS):
	make -C $@
