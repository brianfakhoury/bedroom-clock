CROSS := .toolchain/riscv32-esp-elf/bin/riscv32-esp-elf-
SOURCES := $(wildcard src/*/*.c src/boot/*.S)
INCLUDES := -Isrc/boot -Isrc/hal -Isrc/drivers -Isrc/app -Isrc/board
WARNINGS := -Wall -Wextra -Werror

CFLAGS := -march=rv32imac_zicsr_zifencei -mabi=ilp32 -mno-relax \
          -Os -g -flto \
          -ffreestanding -fno-builtin -fno-common -fstrict-volatile-bitfields \
          -ffunction-sections -fdata-sections \
          $(WARNINGS) $(INCLUDES)

all: build/clock.bin

setup:
	@uv run setup

build/clock.elf: $(SOURCES) $(wildcard src/*/*.h) src/boot/link.ld Makefile
	@mkdir -p build
	@echo "Compiling firmware"
	@$(CROSS)gcc $(CFLAGS) -nostdlib -Tsrc/boot/link.ld \
		-Wl,--gc-sections,--no-warn-rwx-segments,-Map=build/clock.map \
		$(SOURCES) -lgcc -o $@
	@$(CROSS)size $@

build/clock.bin: build/clock.elf
	@uv run esptool --chip esp32c6 elf2image \
		--flash_mode dio --flash_freq 40m --flash_size 16MB \
		-o $@ $< > build/image.log 2>&1 || { cat build/image.log; exit 1; }
	@echo "Created firmware image: $@"

flash: build/clock.bin
	@uv run clock flash --file $< $(if $(PORT),--port $(PORT))

set-time:
	@uv run clock set-time $(if $(PORT),--port $(PORT))

test:
	@echo "Running C tests"
	@mkdir -p build
	@for source in tests/*/*.c; do \
		cc -std=c11 $(WARNINGS) $(INCLUDES) "$$source" -o build/test && \
		build/test || exit 1; \
	done
	@echo "All C tests passed"
	@echo "Running Python tests"
	@uv run pytest tests/tools --quiet > build/pytest.log 2>&1 || \
		{ cat build/pytest.log; exit 1; }
	@echo "All Python tests passed"

clean:
	@rm -rf build
	@echo "Build files cleaned"

.PHONY: all setup test clean flash set-time
