#!/bin/sh
# Install or update one explicitly selected local Mirket release for Codex.
# The trusted release record supplies both hashes. Large third-party assets are
# acquired into private staging; Mirket still owns release verification and activation.
set -eu
umask 077

fail() { printf 'mirket installer: %s\n' "$1" >&2; exit 1; }
usage() {
  printf '%s\n' 'usage: install-macos.sh --source ABSOLUTE_RELEASE_DIR --manifest-sha256 HEX --binary-sha256 HEX [--video-chrome-archive ABSOLUTE_ZIP] [--semantic-runtime-archive ABSOLUTE_TGZ --semantic-model-root ABSOLUTE_DIR] [--raster-archive-root ABSOLUTE_DIR] [--user-home ABSOLUTE_DIR] [--project-root ABSOLUTE_DIR] [--update | --reinstall | --reinstall-reconcile | --uninstall-reconcile]' >&2
  exit 2
}
valid_hash() {
  [ "${#1}" -eq 64 ] || return 1
  case "$1" in *[!0123456789abcdef]*) return 1 ;; esac
}
actual_hash() {
  result=$(/usr/bin/shasum -a 256 "$1") || fail "cannot hash $1"
  printf '%s\n' "${result%% *}"
}
require_hash() {
  actual=$(actual_hash "$1")
  [ "$actual" = "$2" ] || fail "SHA-256 mismatch for $1"
}

source_dir= manifest_hash= binary_hash= project_root= video_chrome_archive_source= semantic_runtime_archive_source= semantic_model_root= raster_archive_root= action=setup
selected_user_home=${HOME:-}
while [ "$#" -gt 0 ]; do
  case "$1" in
    --source|--manifest-sha256|--binary-sha256|--video-chrome-archive|--semantic-runtime-archive|--semantic-model-root|--raster-archive-root|--user-home|--project-root)
      [ "$#" -ge 2 ] || usage
      case "$1" in
        --source) source_dir=$2 ;;
        --manifest-sha256) manifest_hash=$2 ;;
        --binary-sha256) binary_hash=$2 ;;
        --video-chrome-archive) video_chrome_archive_source=$2 ;;
        --semantic-runtime-archive) semantic_runtime_archive_source=$2 ;;
        --semantic-model-root) semantic_model_root=$2 ;;
        --raster-archive-root) raster_archive_root=$2 ;;
        --user-home) selected_user_home=$2 ;;
        --project-root) project_root=$2 ;;
      esac
      shift 2 ;;
    --update) [ "$action" = setup ] || usage; action=update; shift ;;
    --reinstall) [ "$action" = setup ] || usage; action=reinstall; shift ;;
    --reinstall-reconcile) [ "$action" = setup ] || usage; action=reinstall-reconcile; shift ;;
    --uninstall-reconcile) [ "$action" = setup ] || usage; action=uninstall-reconcile; shift ;;
    *) usage ;;
  esac
done

[ "$(/usr/bin/uname -s)" = Darwin ] || fail 'this script requires macOS'
[ "$(/usr/bin/uname -m)" = arm64 ] || fail 'this release entry point requires macOS ARM64'
host_version=$(/usr/bin/sw_vers -productVersion 2>/dev/null) || fail 'cannot determine macOS version'
[ "$host_version" = 26.5.2 ] || fail 'this first release is qualified only for macOS 26.5.2 ARM64'
[ -n "$source_dir" ] && [ -n "$manifest_hash" ] && [ -n "$binary_hash" ] || usage
valid_hash "$manifest_hash" && valid_hash "$binary_hash" || fail 'expected hashes must be 64 lowercase hexadecimal characters'
case "$source_dir" in /*) ;; *) fail 'source must be an absolute directory' ;; esac
case "$selected_user_home" in /*) ;; *) fail 'user home must be an absolute directory' ;; esac
[ -d "$source_dir" ] && [ ! -L "$source_dir" ] || fail 'source must be a real directory'
[ -d "$selected_user_home" ] && [ ! -L "$selected_user_home" ] || fail 'user home must be a real directory'
source_canonical=$(cd "$source_dir" && pwd -P) || fail 'cannot resolve source'
user_canonical=$(cd "$selected_user_home" && pwd -P) || fail 'cannot resolve user home'
[ "$source_dir" = "$source_canonical" ] || fail 'source path must be canonical'
[ "$selected_user_home" = "$user_canonical" ] || fail 'user home path must be canonical'
if [ -n "$project_root" ]; then
  case "$project_root" in /*) ;; *) fail 'project root must be absolute' ;; esac
  case "$action" in
    reinstall|reinstall-reconcile|uninstall-reconcile) fail 'project root is not used by lifecycle reconciliation or reinstall' ;;
  esac
fi

manifest=$source_dir/manifest.json
binary=$source_dir/bin/mirket
[ -f "$manifest" ] && [ ! -L "$manifest" ] || fail 'release manifest is missing or linked'
[ -f "$binary" ] && [ ! -L "$binary" ] || fail 'release executable is missing or linked'
require_hash "$manifest" "$manifest_hash"

# Execute an exact private copy, so a source-path replacement between hashing
# and process startup cannot change the installer binary. The Rust setup/update
# owner independently verifies the complete selected release and destinations.
scratch=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/mirket-install.XXXXXXXX") || fail 'cannot create private scratch'
cleanup() {
  /bin/rm -rf "$scratch"
}
trap cleanup EXIT
/bin/cp "$binary" "$scratch/mirket" || fail 'cannot copy release executable'
/bin/chmod 700 "$scratch/mirket" || fail 'cannot protect release executable'
require_hash "$scratch/mirket" "$binary_hash"

install_source=$source_canonical
video_selected=0 semantic_selected=0 raster_selected=0
if [ "$action" != uninstall-reconcile ]; then
  components=$(/usr/bin/plutil -extract components json -o - "$manifest") || fail 'release components are invalid'
  case "$components" in
    *'"video"'*) video_selected=1 ;;
  esac
  case "$components" in
    *'"semantic"'*) semantic_selected=1 ;;
  esac
  case "$components" in
    *'"raster"'*) raster_selected=1 ;;
  esac
  if [ "$video_selected" -eq 1 ] || [ "$semantic_selected" -eq 1 ] || [ "$raster_selected" -eq 1 ]; then
    [ -d "$source_canonical/runtime" ] && [ ! -L "$source_canonical/runtime" ] || fail 'release runtime directory is invalid'
    if [ "$video_selected" -eq 1 ]; then
      [ ! -e "$source_canonical/runtime/browser" ] && [ ! -L "$source_canonical/runtime/browser" ] || fail 'video Chrome source must be acquired by setup'
    fi
    if [ "$raster_selected" -eq 1 ]; then
      [ -d "$source_canonical/runtime/design" ] && [ ! -L "$source_canonical/runtime/design" ] || fail 'raster requires a real design source directory'
      [ ! -e "$source_canonical/runtime/design/raster" ] && [ ! -L "$source_canonical/runtime/design/raster" ] || fail 'raster source must be acquired by setup'
    fi
    install_source=$scratch/release
    /usr/bin/ditto "$source_canonical" "$install_source" || fail 'cannot stage release'
    [ -d "$install_source/runtime" ] && [ ! -L "$install_source/runtime" ] || fail 'staged runtime directory is invalid'
  fi
  if [ "$video_selected" -eq 1 ]; then
      video_chrome_archive=$scratch/chrome-mac-arm64.zip
      if [ -n "$video_chrome_archive_source" ]; then
        case "$video_chrome_archive_source" in /*) ;; *) fail 'video Chrome archive path must be absolute' ;; esac
        [ -f "$video_chrome_archive_source" ] && [ ! -L "$video_chrome_archive_source" ] || fail 'video Chrome archive must be a regular file'
        /bin/cp "$video_chrome_archive_source" "$video_chrome_archive" || fail 'cannot stage supplied video Chrome archive'
      else
        /usr/bin/curl --fail --silent --show-error --location --proto '=https' --proto-redir '=https' --tlsv1.2 \
          --connect-timeout 20 --max-time 600 \
          --output "$video_chrome_archive" \
          'https://storage.googleapis.com/chrome-for-testing-public/156.0.8078.73/mac-arm64/chrome-mac-arm64.zip' \
          || fail 'cannot acquire the pinned Chrome for Testing archive'
      fi
      [ "$(/usr/bin/stat -f %z "$video_chrome_archive")" -eq 194963618 ] || fail 'video Chrome archive size mismatch'
      require_hash "$video_chrome_archive" a46be48ee2e33b3edc1a98a39668b9f0abb8f23f031bc4df195e2388e6de28d1
      /bin/mkdir -m 700 "$install_source/runtime/browser" || fail 'cannot create private video Chrome staging'
      /usr/bin/ditto -x -k "$video_chrome_archive" "$install_source/runtime/browser" || fail 'cannot extract pinned video Chrome archive'
      # Mirket verifies the complete extracted tree, adapter and runtime
      # against its compiled inventory before any managed destination changes.
  fi
  if [ "$semantic_selected" -eq 1 ]; then
    # A prepared release source contains only the small, manifest-bound receipt.
    # The model and native runtime are acquired before the setup transaction.
    semantic_source=$source_canonical/runtime/semantic
    [ -d "$semantic_source" ] && [ ! -L "$semantic_source" ] || fail 'semantic source must contain only its receipt'
    [ -f "$semantic_source/package-receipt.json" ] && [ ! -L "$semantic_source/package-receipt.json" ] || fail 'semantic receipt is missing or linked'
    [ "$(/usr/bin/find "$semantic_source" -type f -print | /usr/bin/wc -l | /usr/bin/tr -d ' ')" -eq 1 ] || fail 'semantic source contains unexpected files'
    [ -z "$(/usr/bin/find "$semantic_source" -type l -print)" ] || fail 'semantic source contains links'
    [ "$(/usr/bin/stat -f %z "$semantic_source/package-receipt.json")" -eq 2149 ] || fail 'semantic receipt size mismatch'
    require_hash "$semantic_source/package-receipt.json" 8ee58d0c9ad7cc708c6df150f37799111ad415e063382b2fcd1cdde369edcbe4
    semantic_stage=$install_source/runtime/semantic
    if [ -n "$semantic_runtime_archive_source" ]; then
      case "$semantic_runtime_archive_source" in /*) ;; *) fail 'semantic runtime archive path must be absolute' ;; esac
      [ -f "$semantic_runtime_archive_source" ] && [ ! -L "$semantic_runtime_archive_source" ] || fail 'semantic runtime archive must be a regular file'
    fi
    if [ -n "$semantic_model_root" ]; then
      case "$semantic_model_root" in /*) ;; *) fail 'semantic model root must be absolute' ;; esac
      [ -d "$semantic_model_root" ] && [ ! -L "$semantic_model_root" ] || fail 'semantic model root must be a real directory'
    fi
    semantic_archive=$scratch/onnxruntime-osx-arm64-1.31.0.tgz
    if [ -n "$semantic_runtime_archive_source" ]; then
      /bin/cp "$semantic_runtime_archive_source" "$semantic_archive" || fail 'cannot stage supplied semantic runtime archive'
    else
      /usr/bin/curl --fail --silent --show-error --location --proto '=https' --proto-redir '=https' --tlsv1.2 \
        --connect-timeout 20 --max-time 900 --output "$semantic_archive" \
        'https://github.com/microsoft/onnxruntime/releases/download/v1.31.0/onnxruntime-osx-arm64-1.31.0.tgz' \
        || fail 'cannot acquire the pinned ONNX Runtime archive'
    fi
    [ "$(/usr/bin/stat -f %z "$semantic_archive")" -eq 36549009 ] || fail 'semantic runtime archive size mismatch'
    require_hash "$semantic_archive" 1031b36dd3bdaa8d976a6130d9cf2a05e9be320b93ae804f8427b91dc3432afc
    /bin/mkdir -p "$semantic_stage/onnxruntime" "$semantic_stage/model/onnx" "$semantic_stage/model/1_Pooling" || fail 'cannot create private semantic staging'
    extract_semantic_file() {
      member=$1 destination=$2 expected_size=$3 expected_hash=$4
      /usr/bin/tar -xzOf "$semantic_archive" "$member" > "$semantic_stage/$destination" || fail "cannot extract semantic runtime member $member"
      [ "$(/usr/bin/stat -f %z "$semantic_stage/$destination")" -eq "$expected_size" ] || fail "semantic runtime member size mismatch: $destination"
      require_hash "$semantic_stage/$destination" "$expected_hash"
    }
    extract_semantic_file './onnxruntime-osx-arm64-1.31.0/ThirdPartyNotices.txt' 'onnxruntime/ThirdPartyNotices.txt' 338088 143764b952fdb1a7c69ce653bfba74a7744d6a8a573bfb73e235fba356c83de3
    extract_semantic_file './onnxruntime-osx-arm64-1.31.0/LICENSE' 'onnxruntime/LICENSE' 1073 2f07c72751aed99790b8a4869cf2311df85a860b22ded05fa22803587a48922c
    extract_semantic_file './onnxruntime-osx-arm64-1.31.0/lib/libonnxruntime.1.31.0.dylib' 'onnxruntime/libonnxruntime.1.31.0.dylib' 44830288 2a50a63e7f41f8d4d551d3ee47248df3864f6540109fce8ad520ca3b51af4b70
    acquire_model_file() {
      relative=$1 remote_path=$2 expected_size=$3 expected_hash=$4
      destination=$semantic_stage/model/$relative
      if [ -n "$semantic_model_root" ]; then
        case "$relative" in
          */*) source_parent=$semantic_model_root/${relative%/*}
               [ -d "$source_parent" ] && [ ! -L "$source_parent" ] || fail "semantic model source directory is missing or linked: $relative" ;;
        esac
        source_file=$semantic_model_root/$relative
        [ -f "$source_file" ] && [ ! -L "$source_file" ] || fail "semantic model source is missing or linked: $relative"
        /bin/cp "$source_file" "$destination" || fail "cannot stage semantic model file: $relative"
      else
        /usr/bin/curl --fail --silent --show-error --location --proto '=https' --proto-redir '=https' --tlsv1.2 \
          --connect-timeout 20 --max-time 900 --max-filesize "$((expected_size + 4096))" --output "$destination" \
          "https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/e8f8c211226b894fcb81acc59f3b34ba3efd5f42/$remote_path" \
          || fail "cannot acquire pinned semantic model file: $relative"
      fi
      [ "$(/usr/bin/stat -f %z "$destination")" -eq "$expected_size" ] || fail "semantic model size mismatch: $relative"
      require_hash "$destination" "$expected_hash"
    }
    acquire_model_file 'onnx/tokenizer.json' 'tokenizer.json' 9081518 2c3387be76557bd40970cec13153b3bbf80407865484b209e655e5e4729076b8
    acquire_model_file 'onnx/tokenizer_config.json' 'tokenizer_config.json' 526 5036ea374ffedd706e3bef33e2e0d6953cb868ef8a490e76e32ba0faa37a6b9b
    acquire_model_file 'onnx/special_tokens_map.json' 'special_tokens_map.json' 239 378eb3bf733eb16e65792d7e3fda5b8a4631387ca04d2015199c4d4f22ae554d
    acquire_model_file 'onnx/config.json' 'config.json' 645 6300193cb75e01cf80c96decef7187dfb33094d97cc1490b7ead6ff134476e4e
    acquire_model_file '1_Pooling/config.json' '1_Pooling/config.json' 190 4be450dde3b0273bb9787637cfbd28fe04a7ba6ab9d36ac48e92b11e350ffc23
    acquire_model_file 'sentence_bert_config.json' 'sentence_bert_config.json' 53 70f4448f31320443fe3557cacea5abf2dcc4915dda8c80646bec9f3bb0aa5a1f
    acquire_model_file 'README.md' 'README.md' 3888 1e98ea05b0de579fcaad3d625b62ea55647142ed674d5f5ebf1440e4bbbb6f23
    acquire_model_file 'onnx/model_qint8_arm64.onnx' 'onnx/model_qint8_arm64.onnx' 118412398 783fea82d71a58179b830a4dbd2d58447e640609e98eedf9ffa12622d375a672
  fi
  if [ "$raster_selected" -eq 1 ]; then
    if [ -n "$raster_archive_root" ]; then
      case "$raster_archive_root" in /*) ;; *) fail 'raster archive root must be absolute' ;; esac
      [ -d "$raster_archive_root" ] && [ ! -L "$raster_archive_root" ] || fail 'raster archive root must be a real directory'
      raster_archive_canonical=$(cd "$raster_archive_root" && pwd -P) || fail 'cannot resolve raster archive root'
      [ "$raster_archive_root" = "$raster_archive_canonical" ] || fail 'raster archive root must be canonical'
    fi
    raster_stage=$install_source/runtime/design/raster/node_modules
    /bin/mkdir -p "$raster_stage/@img" || fail 'cannot create private raster staging'
    acquire_raster_package() {
      package_name=$1 archive_name=$2 archive_size=$3 archive_hash=$4 archive_url=$5
      destination=$raster_stage/$package_name
      archive=$scratch/$archive_name
      if [ -n "$raster_archive_root" ]; then
        source_archive=$raster_archive_root/$archive_name
        [ -f "$source_archive" ] && [ ! -L "$source_archive" ] || fail "raster archive is missing or linked: $archive_name"
        /bin/cp "$source_archive" "$archive" || fail "cannot stage raster archive: $archive_name"
      else
        /usr/bin/curl --fail --silent --show-error --location --proto '=https' --proto-redir '=https' --tlsv1.2 \
          --connect-timeout 20 --max-time 600 --max-filesize "$((archive_size + 4096))" \
          --output "$archive" "$archive_url" || fail "cannot acquire pinned raster archive: $archive_name"
      fi
      [ "$(/usr/bin/stat -f %z "$archive")" -eq "$archive_size" ] || fail "raster archive size mismatch: $archive_name"
      require_hash "$archive" "$archive_hash"
      /bin/mkdir -m 700 "$destination" || fail "cannot create raster package destination: $package_name"
      /usr/bin/tar -xzf "$archive" --strip-components 1 -C "$destination" || fail "cannot extract pinned raster archive: $archive_name"
    }
    # These six immutable archives were inspected for safe package/ members.
    # The CLI independently checks every extracted byte and mode before setup.
    acquire_raster_package sharp sharp-0.35.5.tgz 214749 3b6bbbe6b308f5c383938ca4a31926cd5d79c4651ed8c5fda16a97da2014a9ac 'https://registry.npmjs.org/sharp/-/sharp-0.35.5.tgz'
    acquire_raster_package @img/colour colour-1.1.0.tgz 17410 6c2df3ac33d4b8647191ad8942a579e6004be00c25846677eab85f04702d85b1 'https://registry.npmjs.org/@img/colour/-/colour-1.1.0.tgz'
    acquire_raster_package detect-libc detect-libc-2.1.2.tgz 7776 270dec0fc06cff86481da8af2dd8f18dee6b602790b14ef0e1c2c18d7da39427 'https://registry.npmjs.org/detect-libc/-/detect-libc-2.1.2.tgz'
    acquire_raster_package semver semver-7.8.5.tgz 29399 d85045d4300d7d57c891336b95df532e73f34c22ffcd222452b6d08b9d127d5d 'https://registry.npmjs.org/semver/-/semver-7.8.5.tgz'
    acquire_raster_package @img/sharp-darwin-arm64 sharp-darwin-arm64-0.35.5.tgz 110737 52168af984c322e5548bebfb437a9976244824338ea5b8a5b1e0cdaaf06480cb 'https://registry.npmjs.org/@img/sharp-darwin-arm64/-/sharp-darwin-arm64-0.35.5.tgz'
    acquire_raster_package @img/sharp-libvips-darwin-arm64 sharp-libvips-darwin-arm64-1.3.4.tgz 8341921 e8350e62344d027995ef46cdd97a5ea1c88bc54ab82ab8c3fcb0c076d839675f 'https://registry.npmjs.org/@img/sharp-libvips-darwin-arm64/-/sharp-libvips-darwin-arm64-1.3.4.tgz'
    [ -z "$(/usr/bin/find "$install_source/runtime/design/raster" -type l -print)" ] || fail 'raster archive introduced a link'
    /usr/bin/find "$install_source/runtime/design/raster" -type f -exec /bin/chmod 600 {} + || fail 'cannot protect raster files'
    /usr/bin/find "$install_source/runtime/design/raster" -type d -exec /bin/chmod 700 {} + || fail 'cannot protect raster directories'
  fi
fi
[ -z "$video_chrome_archive_source" ] || [ "$video_selected" -eq 1 ] || fail 'video Chrome archive was supplied for a release without video'
[ -z "$semantic_runtime_archive_source" ] || [ "$semantic_selected" -eq 1 ] || fail 'semantic runtime archive was supplied for a release without semantic memory'
[ -z "$semantic_model_root" ] || [ "$semantic_selected" -eq 1 ] || fail 'semantic model root was supplied for a release without semantic memory'
[ -z "$raster_archive_root" ] || [ "$raster_selected" -eq 1 ] || fail 'raster archive root was supplied for a release without raster'

if [ "$action" = uninstall-reconcile ]; then
  "$scratch/mirket" uninstall-reconcile --home "$user_canonical/.mirket" --user-home "$user_canonical"
elif [ -n "$project_root" ]; then
  "$scratch/mirket" "$action" --home "$user_canonical/.mirket" --source "$install_source" --user-home "$user_canonical" --project-root "$project_root"
else
  "$scratch/mirket" "$action" --home "$user_canonical/.mirket" --source "$install_source" --user-home "$user_canonical"
fi
