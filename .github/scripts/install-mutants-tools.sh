#!/usr/bin/env bash
# ミューテーションテストで使う cargo-mutants と cargo-nextest を、公式リリースのバイナリを
# sha256 で検証してから $CARGO_HOME/bin に置く（cargo install のビルドを避ける）。
# 版と sha256 は mutation.yml の環境変数で渡す。Linux（plan）と Windows（シャード、Git Bash）の両方で動く。
# 引数に --mutants-only を付けると cargo-mutants だけを入れる（plan は --list しか使わない）。
set -euo pipefail
: "${CARGO_MUTANTS_VERSION:?}"
bindir="${CARGO_HOME:-${HOME}/.cargo}/bin"

# install_tool <URL> <sha256> <アーカイブ内の実行ファイル名>
install_tool() {
  local url=$1 sha=$2 bin=$3 work archive
  work=$(mktemp -d)
  archive="${work}/$(basename "$url")"
  curl -fsSL -o "$archive" "$url"
  echo "${sha}  ${archive}" | sha256sum -c -
  case "$archive" in
    *.tar.gz) tar -xzf "$archive" --no-same-owner -C "$work" ;;
    *.zip) 7z x -y -o"$work" "$archive" > /dev/null ;;
  esac
  install -m 0755 "${work}/${bin}" "${bindir}/${bin}"
}

case "${RUNNER_OS:-Linux}" in
  Windows)
    mutants_asset="cargo-mutants-x86_64-pc-windows-msvc.zip"
    mutants_sha="${CARGO_MUTANTS_SHA256_WINDOWS:?}"
    nextest_target="x86_64-pc-windows-msvc"
    nextest_sha="${CARGO_NEXTEST_SHA256_WINDOWS:-}"
    exe=".exe"
    ;;
  *)
    mutants_asset="cargo-mutants-x86_64-unknown-linux-gnu.tar.gz"
    mutants_sha="${CARGO_MUTANTS_SHA256_LINUX:?}"
    nextest_target="x86_64-unknown-linux-gnu"
    nextest_sha="${CARGO_NEXTEST_SHA256_LINUX:-}"
    exe=""
    ;;
esac

install_tool \
  "https://github.com/sourcefrog/cargo-mutants/releases/download/v${CARGO_MUTANTS_VERSION}/${mutants_asset}" \
  "$mutants_sha" "cargo-mutants${exe}"
cargo mutants --version

if [[ "${1:-}" != "--mutants-only" ]]; then
  : "${CARGO_NEXTEST_VERSION:?}" "${nextest_sha:?}"
  install_tool \
    "https://github.com/nextest-rs/nextest/releases/download/cargo-nextest-${CARGO_NEXTEST_VERSION}/cargo-nextest-${CARGO_NEXTEST_VERSION}-${nextest_target}.tar.gz" \
    "$nextest_sha" "cargo-nextest${exe}"
  cargo nextest --version
fi
