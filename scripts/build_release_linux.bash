#!/usr/bin/env bash

set -euo pipefail

project_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
thread_core="${1:-posix}"
version="${2:-$(sed -n 's/^version = "\([^"]*\)"/\1/p' "${project_dir}/Cargo.toml" | head -n 1)}"
rust_toolchain="${RUST_TOOLCHAIN:-1.98.1}"

case "${thread_core}" in
    posix|preempt_rt)
        ;;
    *)
        echo "unsupported release thread core: ${thread_core}" >&2
        exit 2
        ;;
esac

if ! command -v podman >/dev/null 2>&1; then
    echo "podman is required to build a portable Linux release" >&2
    exit 1
fi

image="rtsyn-release-manylinux228:${thread_core}"
archive_root="rtsyn-v${version}-linux-x86_64-${thread_core}"
dist_dir="${project_dir}/dist"
staging_dir="${dist_dir}/${archive_root}"

mkdir -p "${staging_dir}"

podman build \
    --file "${project_dir}/Containerfile.release" \
    --build-arg "THREAD_CORE=${thread_core}" \
    --build-arg "RUST_TOOLCHAIN=${rust_toolchain}" \
    --tag "${image}" \
    "${project_dir}"

container_id="$(podman create "${image}")"
trap 'podman rm --force "${container_id}" >/dev/null 2>&1 || true' EXIT

podman cp "${container_id}:/release/rtsyn" "${staging_dir}/rtsyn"
tar --create --gzip --directory "${dist_dir}" \
    --file "${dist_dir}/${archive_root}.tar.gz" "${archive_root}"

echo "created ${dist_dir}/${archive_root}.tar.gz"

