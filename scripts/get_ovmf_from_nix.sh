#!/usr/bin/env bash

# helper script to take ovmf files from nix package manager

set -euxo pipefail

ovmfdir=$(nix build .#ovmf.fd --print-out-paths --no-link)
rm -f *.fd
cp $ovmfdir/FV/* .
chmod +rw *.fd
