#!/usr/bin/env bash
set -euo pipefail

usage() {
	echo "Usage: $0 <crate_name> <version>" >&2
	exit 1
}

crate_name="$1"; shift
if [[ -z "$crate_name" ]]; then usage; fi
version="$1"; shift
if [[ -z "$version" ]]; then usage; fi

# build the container image
docker buildx build --tag "$crate_name:$version" .
if [[ -n "$DEMOCRACY_REGISTRY" ]]; then
	docker image tag "$crate_name:$version" "$DEMOCRACY_REGISTRY/$crate_name:$version"
	docker image push "$DEMOCRACY_REGISTRY/$crate_name:$version"
fi

# generate the changelog
git cliff --output "CHANGELOG.md" --tag "$version"
git add "CHANGELOG.md"
git commit -m "chore(changelog): generate changelog for version $version"
