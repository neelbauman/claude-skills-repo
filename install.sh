#!/usr/bin/env bash
set -euo pipefail

# ── Configuration ────────────────────────────────────────────────
REPO="neelbauman/claude-skills-repo"                       # TODO: Replace with actual GitHub owner/repo
BIN_NAME="claude-registry"
INSTALL_DIR="${HOME}/.local/bin"
CONTENT_DIR="${HOME}/.local/share/claude-registry"

# ── OS / Architecture detection ─────────────────────────────────
detect_platform() {
  local os arch

  case "$(uname -s)" in
    Linux*)  os="linux" ;;
    Darwin*) os="darwin" ;;
    *)
      echo "Error: Unsupported OS: $(uname -s)" >&2
      exit 1
      ;;
  esac

  case "$(uname -m)" in
    x86_64|amd64)  arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *)
      echo "Error: Unsupported architecture: $(uname -m)" >&2
      exit 1
      ;;
  esac

  echo "${os} ${arch}"
}

# ── Download helpers ─────────────────────────────────────────────
download() {
  local url="$1" dest="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL -o "$dest" "$url"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$dest" "$url"
  else
    echo "Error: curl or wget is required" >&2
    return 1
  fi
}

# ── Install from GitHub Releases ─────────────────────────────────
install_from_release() {
  local os="$1" arch="$2"
  local url="https://github.com/${REPO}/releases/latest/download/${BIN_NAME}-${os}-${arch}.tar.gz"
  local tmpdir

  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' RETURN

  echo "Downloading ${BIN_NAME} from GitHub Releases..."
  echo "  URL: ${url}"

  if ! download "$url" "${tmpdir}/${BIN_NAME}.tar.gz"; then
    echo "Download failed." >&2
    return 1
  fi

  tar -xzf "${tmpdir}/${BIN_NAME}.tar.gz" -C "$tmpdir"

  mkdir -p "$INSTALL_DIR"
  mv "${tmpdir}/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"
  chmod +x "${INSTALL_DIR}/${BIN_NAME}"

  echo "Installed ${BIN_NAME} to ${INSTALL_DIR}/${BIN_NAME}"

  # Install completions and content from the extracted archive
  if [[ -d "${tmpdir}/completions" ]]; then
    install_completions "$tmpdir"
  fi
  install_content "$tmpdir"
}

# ── Fallback: build from source ──────────────────────────────────
install_from_source() {
  if ! command -v cargo >/dev/null 2>&1; then
    echo "Error: cargo is not installed. Install Rust first: https://rustup.rs" >&2
    exit 1
  fi
  if ! command -v git >/dev/null 2>&1; then
    echo "Error: git is not installed." >&2
    exit 1
  fi

  local tmpdir
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' RETURN

  echo "Building ${BIN_NAME} from source..."

  git clone "https://github.com/${REPO}.git" "${tmpdir}/repo"
  (cd "${tmpdir}/repo/cli" && cargo build --release)

  mkdir -p "$INSTALL_DIR"
  cp "${tmpdir}/repo/cli/target/release/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"
  chmod +x "${INSTALL_DIR}/${BIN_NAME}"

  echo "Installed ${BIN_NAME} to ${INSTALL_DIR}/${BIN_NAME}"

  # Install completions and content from the cloned repo
  install_completions "${tmpdir}/repo"
  install_content "${tmpdir}/repo"
}

# ── Install registry content ─────────────────────────────────────
# Copies claude/ and profiles/ from the source directory to CONTENT_DIR.
install_content() {
  local search_dir="${1:-$(cd "$(dirname "$0")" && pwd)}"

  mkdir -p "${CONTENT_DIR}"

  if [[ -d "${search_dir}/claude" ]]; then
    cp -r "${search_dir}/claude" "${CONTENT_DIR}/"
    echo "Installed registry content to ${CONTENT_DIR}/claude"
  fi

  if [[ -d "${search_dir}/profiles" ]]; then
    cp -r "${search_dir}/profiles" "${CONTENT_DIR}/"
    echo "Installed profiles to ${CONTENT_DIR}/profiles"
  fi
}

# ── Install shell completions ────────────────────────────────────
# Accepts an optional argument: directory containing completions/
install_completions() {
  local search_dir="${1:-$(cd "$(dirname "$0")" && pwd)}"

  # bash completion
  local bash_comp_dir="${HOME}/.local/share/bash-completion/completions"
  local bash_src="${search_dir}/completions/claude-registry.bash"
  if [[ -f "$bash_src" ]]; then
    mkdir -p "$bash_comp_dir"
    cp "$bash_src" "${bash_comp_dir}/${BIN_NAME}"
    echo "Installed bash completion to ${bash_comp_dir}/${BIN_NAME}"
  fi

  # zsh completion
  local zsh_comp_dir="${HOME}/.local/share/zsh/site-functions"
  local zsh_src="${search_dir}/completions/_claude-registry"
  if [[ -f "$zsh_src" ]]; then
    mkdir -p "$zsh_comp_dir"
    cp "$zsh_src" "${zsh_comp_dir}/_${BIN_NAME}"
    echo "Installed zsh completion to ${zsh_comp_dir}/_${BIN_NAME}"
  fi
}

# ── Main ─────────────────────────────────────────────────────────
main() {
  echo "Installing ${BIN_NAME}..."
  echo

  read -r os arch <<< "$(detect_platform)"
  echo "Detected platform: ${os} ${arch}"

  if ! install_from_release "$os" "$arch"; then
    echo
    echo "Pre-built binary not available. Falling back to source build..."
    install_from_source
  fi

  echo
  # PATH check
  case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
      echo "NOTE: ${INSTALL_DIR} is not in your PATH."
      echo "Add the following to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
      echo
      echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
      echo
      ;;
  esac

  echo "Done! Run '${BIN_NAME} --help' to get started."
}

main
