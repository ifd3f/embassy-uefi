#!/usr/bin/env bash

# Helper script to build and run the ping example on a QEMU VM.
#
# Open up Wireshark, run this script, and watch the packets go by.
# Modifications may be required for your system :).
# 
# Prerequisites:
# - You should copy OVMF_CODE.fd and OVMF_VARS.fd into the working directory
#   before running this. In most systems they're under /usr/share/OVMF, but on
#   NixOS you can use get_ovmf_from_nix.sh
# - This assumes you already have a bridge set up named virbr0.

set -euxo pipefail

example=ping

# set up an esp as a VVFAT directory
cargo build --example "$example" --target x86_64-unknown-uefi --release
rm -rf esp
mkdir -p esp/efi/boot
cp -r "target/x86_64-unknown-uefi/release/examples/$example.efi" esp/efi/boot/bootx64.efi

# launch!
qemu-system-x86_64 \
    -D ./qemu.log \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_CODE.fd \
    -drive if=pflash,format=raw,readonly=on,file=OVMF_VARS.fd \
    -drive format=raw,file=fat:rw:esp \
    -cpu max

    # if it doesn't appear to output packets, you can use an ipxe image instead (https://ipxe.org/)
    # to test if networking works sans smoltcp-rs
    # -drive if=virtio,file=ipxe.iso 