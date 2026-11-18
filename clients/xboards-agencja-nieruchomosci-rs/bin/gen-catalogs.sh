#!/usr/bin/env bash
# Writes every file of src/ into $XMETA_OUT (called by `xmeta codegen`, in this client's folder).
# xmeta compares them with src/ and writes them there.
set -euo pipefail

: "${XMETA_DESCRIPTION:?XMETA_DESCRIPTION is required}"
: "${XMETA_OUT:?XMETA_OUT is required}"

exec cargo run --quiet --locked \
  --manifest-path ../../../../mirrorboards-core/mirrorboards-xmeta/xmeta-rust/Cargo.toml \
  --bin xmeta-rust-codegen -- "$XMETA_DESCRIPTION" "$XMETA_OUT"
