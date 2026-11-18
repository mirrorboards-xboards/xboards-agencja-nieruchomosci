#!/usr/bin/env bash
# Writes the hooks of every module of the description into $XMETA_OUT (called by `xmeta codegen`,
# in this client's folder). xmeta compares them with src/ and writes them there.
set -euo pipefail

: "${XMETA_DESCRIPTION:?XMETA_DESCRIPTION is required}"
: "${XMETA_OUT:?XMETA_OUT is required}"

exec pnpm exec xmeta-react-codegen "$XMETA_DESCRIPTION" "$XMETA_OUT"
