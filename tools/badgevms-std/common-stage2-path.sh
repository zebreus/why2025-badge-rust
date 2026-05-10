#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/common.sh"
repo=$(rust_repo)
stage2_dir_for_repo "$repo"
