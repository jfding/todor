#!/bin/sh

[[ -z $1 ]] && { echo "Usage: $(basename $0) <version>"; exit 1 }
version="$1"

# Linux and Darwin builds.
for arch in aarch64 x86_64; do
  for target in apple-darwin unknown-linux-musl; do
    url="https://github.com/jfding/todor/releases/download/$version/todor-$arch-$target.zip"
    sha=$(curl -sfSL "$url" | sha256sum)
    echo "$version-$arch-$target $sha"
  done
done

# Source.
for ext in zip tar.gz; do
  url="https://github.com/jfding/todor/archive/refs/tags/$version.$ext"
  sha=$(curl -sfSL "$url" | sha256sum)
  echo "source.$ext $sha"
done
