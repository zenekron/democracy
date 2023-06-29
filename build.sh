#!/usr/bin/env bash
set -euo pipefail

sea-orm-cli migrate fresh
sea-orm-cli generate entity -o ./democracy/src/entity
cargo build
