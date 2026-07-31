#!/usr/bin/env sh
set -eu

repository="binibinibin123/geullint"
api_url="https://api.github.com/repos/${repository}/releases?per_page=1"

case "$(uname -s)" in
  Darwin) operating_system="darwin" ;;
  Linux) operating_system="linux" ;;
  *)
    printf 'GeulLint: unsupported operating system: %s\n' "$(uname -s)" >&2
    exit 1
    ;;
esac

case "$(uname -m)" in
  x86_64 | amd64) architecture="x64" ;;
  arm64 | aarch64) architecture="arm64" ;;
  *)
    printf 'GeulLint: unsupported architecture: %s\n' "$(uname -m)" >&2
    exit 1
    ;;
esac

if [ -n "${GEULLINT_VERSION:-}" ]; then
  version="${GEULLINT_VERSION#v}"
else
  release_json="$(curl -fsSL "$api_url")"
  tag_name="$(printf '%s\n' "$release_json" | grep -m 1 '"tag_name"' | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p')"
  if [ -z "$tag_name" ]; then
    printf 'GeulLint: unable to determine the latest release.\n' >&2
    exit 1
  fi
  version="${tag_name#v}"
fi

target="${operating_system}-${architecture}"
archive_stem="geullint-v${version}-${target}"
archive_name="${archive_stem}.tar.gz"
download_base="https://github.com/${repository}/releases/download/v${version}"
install_directory="${GEULLINT_INSTALL_DIR:-${HOME}/.local/bin}"
temporary_directory="$(mktemp -d)"
trap 'rm -rf "$temporary_directory"' EXIT INT TERM

curl -fL --retry 3 -o "${temporary_directory}/${archive_name}" "${download_base}/${archive_name}"
curl -fL --retry 3 -o "${temporary_directory}/${archive_name}.sha256" "${download_base}/${archive_name}.sha256"

if command -v sha256sum >/dev/null 2>&1; then
  (cd "$temporary_directory" && sha256sum -c "${archive_name}.sha256")
elif command -v shasum >/dev/null 2>&1; then
  (cd "$temporary_directory" && shasum -a 256 -c "${archive_name}.sha256")
else
  printf 'GeulLint: sha256sum or shasum is required to verify the download.\n' >&2
  exit 1
fi

tar -xzf "${temporary_directory}/${archive_name}" -C "$temporary_directory"
mkdir -p "$install_directory"
install -m 755 "${temporary_directory}/${archive_stem}/geullint" "${install_directory}/geullint"

printf '\nGeulLint v%s installed at %s/geullint\n' "$version" "$install_directory"
case ":${PATH}:" in
  *":${install_directory}:"*) ;;
  *) printf 'Add %s to PATH, then run: geullint --version\n' "$install_directory" ;;
esac
