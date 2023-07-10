#!/usr/bin/env bash
set -euo pipefail

# https://github.com/crate-ci/cargo-release/blob/master/docs/reference.md#hook-environment-variables
# - PREV_VERSION: The version before cargo-release was executed (before any version bump).
# - PREV_METADATA: The version's metadata field before cargo-release was executed (before any version bump).
# - NEW_VERSION: The current (bumped) crate version.
# - NEW_METADATA: The current (bumped) crate version's metadata field.
# - DRY_RUN: Whether the release is actually happening (true / false)
# - CRATE_NAME: The name of the crate.
# - WORKSPACE_ROOT: The path to the workspace.
# - CRATE_ROOT: The path to the crate.



#
# util
#

run() {
	if [[ "$DRY_RUN" = "true" ]]; then
		echo "[skipped] $*" >&2
	else
		"$@"
	fi
}



#
# steps
#

build_image() {
	local tag="$CRATE_NAME:$NEW_VERSION"
	run docker builx build --tag "$tag" "$WORKSPACE_ROOT"

	if [[ -n "$DEMOCRACY_REGISTRY" ]]; then
		run docker image tag "$tag" "$DEMOCRACY_REGISTRY/$tag"
		run docker image push "$DEMOCRACY_REGISTRY/$tag"
	fi
}

generate_changelog() {
	run git cliff --output "CHANGELOG.md" --tag "$NEW_VERSION"
}



#
# main
#

build_image
generate_changelog
