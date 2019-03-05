BINS := misc bootloader kernel

all: $(BINS)
$(BINS):
	make -C $@

.PHONY: all $(BINS)
