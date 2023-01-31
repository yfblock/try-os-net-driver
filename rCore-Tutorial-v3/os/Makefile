# Building
TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := target/$(TARGET)/$(MODE)/asm
FS_IMG := ../user/target/$(TARGET)/$(MODE)/fs.img
APPS := ../user/src/bin/*

# BOARD
BOARD := qemu
SBI ?= rustsbi
BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin

# Building mode argument
ifeq ($(MODE), release)
	MODE_ARG := --release
endif

# KERNEL ENTRY
KERNEL_ENTRY_PA := 0x80200000

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

# Disassembly
DISASM ?= -x

# Run usertests or usershell
TEST ?=

# build: env $(KERNEL_BIN) fs-img 

build: $(KERNEL_BIN)

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	cargo install cargo-binutils
	rustup component add rust-src
	rustup component add llvm-tools-preview

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@

fs-img: $(APPS)
	@cd ../user && make build TEST=$(TEST)
	@rm -f $(FS_IMG)
	@cd ../easy-fs-fuse && cargo run --release -- -s ../user/src/bin/ -t ../user/target/riscv64gc-unknown-none-elf/release/

nvme-img:
	@rm -f nvme.img
	@dd if=/dev/zero of=nvme.img count=40 bs=1M	# 40M
	@mkfs.vfat nvme.img -F 32

	@mkdir -p mount
	@sudo mount nvme.img ./mount

	@sudo cp -r ../user/target/riscv64gc-unknown-none-elf/release/*.bin ./mount
	@find ./mount -name "*.bin" | awk -F "./mount/" '{print $2}' | awk -F "." '{print $1}' | xargs -i -t sudo mv ./mount/{}.bin ./mount/{}
	@sudo umount ./mount

$(APPS):

kernel:
	@echo Platform: $(BOARD)
	@cp src/linker-$(BOARD).ld src/linker.ld
	@cargo build --release
	@rm src/linker.ld

clean:
	@cargo clean

disasm: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) | less

disasm-vim: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) > $(DISASM_TMP)
	@vim $(DISASM_TMP)
	@rm $(DISASM_TMP)

FWDPORT=6200

# run: run-inner

run: run-nvme

run-nvme: build
# make nvme-img
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) \
		-kernel $(KERNEL_BIN) \
		-drive file=nvme.img,if=none,id=nvm \
		-device nvme,serial=deadbeef,drive=nvm \
		-netdev user,id=net0,hostfwd=udp::$(FWDPORT)-:2000 -object filter-dump,id=net0,netdev=net0,file=packets.pcap \
		-device virtio-net-device,netdev=net0
# -netdev user,id=net0,hostfwd=udp::$(FWDPORT)-:2000 -object filter-dump,id=net0,netdev=net0,file=packets.pcap \
# -device e1000,netdev=net0,bus=pcie.0


# run-inner: build
# 	@qemu-system-riscv64 \
# 		-machine virt \
# 		-nographic \
# 		-bios $(BOOTLOADER) \
# 		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) \
# 		-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
#         -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

# QEMUOPTS += -netdev user,id=net0,hostfwd=udp::$(FWDPORT)-:2000 -object filter-dump,id=net0,netdev=net0,file=packets.pcap
# QEMUOPTS += -device e1000,netdev=net0,bus=pcie.0

run-inner: build
		qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) \
		-kernel $(KERNEL_BIN) \
		-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
        -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
		-netdev user,id=net0,hostfwd=udp::$(FWDPORT)-:2000 -object filter-dump,id=net0,netdev=net0,file=packets.pcap \
		-device virtio-net-device,netdev=net0

debug: build
	@tmux new-session -d \
		"qemu-system-riscv64 -machine virt -nographic -kernel $(KERNEL_BIN) -s -S" && \
		tmux split-window -h "riscv64-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'" && \
		tmux -2 attach-session -d


gdbserver: build
	@qemu-system-riscv64 -machine virt -nographic -bios $(BOOTLOADER) -device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA) -s -S

gdbclient:
	@riscv64-unknown-elf-gdb -ex 'file $(KERNEL_ELF)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234'

ping:
	python3 ping.py $(FWDPORT)

.PHONY: build env kernel clean disasm disasm-vim run-inner fs-img gdbserver gdbclient ping