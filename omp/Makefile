#
# File: Makefile
# Author: Jack Yu (yyu57)
# Simple generic makefile for my C projects
# Run 'make build' to compile binary
#

CC ?= gcc
CFLAGS := -std=gnu17 -Werror -Wall -O3
CFLAGS_SEQ := $(CFLAGS)
CFLAGS_OMP := $(CFLAGS) -fopenmp -DPAR
CFLAGS_OMP_SAFE := $(CFLAGS_OMP) -DCRIT

BIN := matmul
BIN_SEQ := seqc_$(BIN)
BIN_OMP := omp_$(BIN)
BIN_OMP_SAFE := ompsafe_$(BIN)
SRCDIR := src
BLDDIR := build
SRCS := $(shell find $(SRCDIR) -name '*.c')

# === Recipes ===
.DEFAULT_GOAL := all

all: $(BIN_SEQ) $(BIN_OMP) $(BIN_OMP_SAFE)

$(BIN_SEQ): $(BLDDIR)/$(BIN_SEQ)
	@cp $(BLDDIR)/$(BIN_SEQ) $(BIN_SEQ)

$(BIN_OMP): $(BLDDIR)/$(BIN_OMP)
	@cp $(BLDDIR)/$(BIN_OMP) $(BIN_OMP)

$(BIN_OMP_SAFE): $(BLDDIR)/$(BIN_OMP_SAFE)
	@cp $(BLDDIR)/$(BIN_OMP_SAFE) $(BIN_OMP_SAFE)

$(BLDDIR)/$(BIN_SEQ): $(BLDDIR) $(SRCS)
	$(CC) $(CFLAGS_SEQ) $(SRCS) -o $@

$(BLDDIR)/$(BIN_OMP): $(BLDDIR) $(SRCS)
	$(CC) $(CFLAGS_OMP) $(SRCS) -o $@

$(BLDDIR)/$(BIN_OMP_SAFE): $(BLDDIR) $(SRCS)
	$(CC) $(CFLAGS_OMP_SAFE) $(SRCS) -o $@

build: all

$(BLDDIR):
	@mkdir -p $(BLDDIR)

clean:
	@rm -rf $(BLDDIR)
	@rm -f $(BIN_SEQ) $(BIN_OMP) $(BIN_OMP_SAFE)

.PHONY: build run clean all

