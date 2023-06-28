#!/usr/bin/env bash
set -euo pipefail

sea-orm-cli migrate up
sea-orm-cli generate entity -o ./src/entity
cargo build
