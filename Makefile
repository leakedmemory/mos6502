CC := gcc
CFLAGS := -std=c17 -Wall -Wextra -Wfloat-equal -Wpointer-arith \
	  -Wstrict-prototypes -Wwrite-strings -Wcast-qual -Wswitch-enum \
	  -Wconversion -fsanitize=undefined -g -O2

SRC_DIR := src
OBJ_DIR := obj

SRC := $(wildcard $(SRC_DIR)/*.c)
OBJ := $(patsubst $(SRC_DIR)/%.c, $(OBJ_DIR)/%.o, $(SRC))
LIB := libmos6502.a

CDEPS := $(patsubst %.c, %.d, $(SRC))

# general targets
.PHONY: all run clean clean_all

all: make_dirs $(LIB)

clean:
	@rm -rf $(OBJ_DIR) $(LIB)

clean_all:
	@rm -rf $(OBJ_DIR) $(LIB) $(TESTS_OBJ_DIR) $(TESTS_BIN)

# bin compilation
.PHONY: make_dirs
make_dirs: $(OBJ_DIR)

$(OBJ_DIR):
	@mkdir -p $@

$(LIB): $(OBJ)
	@echo "  (LD) Linking library '"$@"'"
	@ar rcs $@ $<
	@echo "Compilation finished"

$(OBJ_DIR)/%.o: $(SRC_DIR)/%.c Makefile
	@echo "  (CC) Compiling '"$<"'"
	@$(CC) -c $< $(CFLAGS) -MMD -MP -o $@

# tests compilation
CXX := g++
CXXFLAGS := -std=c++23 -Wall -Wextra -Wfloat-equal -Wpointer-arith \
	    -Wwrite-strings -Wswitch-enum -Wconversion -fsanitize=undefined -O2
CXXLDFLAGS := -L. -lmos6502 -lgtest

TESTS_DIR := tests
TESTS_OBJ_DIR := $(TESTS_DIR)/obj
TESTS_INCLUDE := -I$(SRC_DIR)

TESTS_MAIN := $(TESTS_DIR)/main.cc
TESTS := $(wildcard $(TESTS_DIR)/*.cc)
TESTS := $(filter-out $(TESTS_MAIN), $(TESTS))
TESTS_OBJ := $(patsubst $(TESTS_DIR)/%.cc, $(TESTS_OBJ_DIR)/%.o, $(TESTS))
TESTS_BIN := $(TESTS_DIR)/main

CXXDEPS := $(patsubst %.cc, %.d, $(TESTS))

.PHONY: tests
tests: make_dirs $(OBJ) $(LIB) make_tests_dirs $(TESTS_BIN)
	@echo
	@./$(TESTS_BIN)

.PHONY: make_tests_dirs
make_tests_dirs: $(TESTS_OBJ_DIR)

$(TESTS_OBJ_DIR):
	@mkdir $@

$(TESTS_BIN): $(TESTS_MAIN) $(OBJ) $(TESTS_OBJ)
	@echo "  (LD) Compiling and linking tests executable '"$@"'"
	@$(CXX) $^ $(CXXLDFLAGS) $(CXXFLAGS) -o $@
	@echo "Compilation finished"

$(TESTS_OBJ_DIR)/%.o: $(TESTS_DIR)/%.cc Makefile
	@echo "  (CC) Compiling test '"$<"'"
	@$(CXX) -c $< $(TESTS_INCLUDE) $(CXXFLAGS) -MMD -MP -o $@

-include $(CDEPS)
-include $(CXXDEPS)
