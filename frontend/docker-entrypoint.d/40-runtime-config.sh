#!/bin/sh
set -eu

fail() {
  printf '%s\n' "$1" >&2
  exit 1
}

api_upstream=${API_UPSTREAM:-}
api_base_url=${API_BASE_URL:-}

[ -n "$api_upstream" ] || fail 'API_UPSTREAM is required'
[ -n "$api_base_url" ] || fail 'API_BASE_URL is required'

host_pattern='([A-Za-z0-9]([A-Za-z0-9-]*[A-Za-z0-9])?)(\.([A-Za-z0-9]([A-Za-z0-9-]*[A-Za-z0-9])?))*'
upstream_pattern="^https?://${host_pattern}(:[0-9]{1,5})?$"
absolute_base_pattern="^https?://${host_pattern}(:[0-9]{1,5})?(/[A-Za-z0-9._-]+)*$"
relative_base_pattern='^/api(/[A-Za-z0-9._-]+)*$'

if ! printf '%s' "$api_upstream" | grep -Eq "$upstream_pattern"; then
  fail 'API_UPSTREAM must be an HTTP(S) origin without credentials, path, query, or fragment'
fi

if ! printf '%s' "$api_base_url" | grep -Eq "$relative_base_pattern|$absolute_base_pattern"; then
  fail 'API_BASE_URL must be /api or a safe HTTP(S) URL'
fi

validate_port() {
  url=$1
  authority=${url#*://}
  authority=${authority%%/*}

  case "$authority" in
    *:*)
      port=${authority##*:}
      [ "$port" -ge 1 ] 2>/dev/null && [ "$port" -le 65535 ] 2>/dev/null || return 1
      ;;
  esac
}

validate_port "$api_upstream" || fail 'API_UPSTREAM has an invalid port'

case "$api_base_url" in
  http://*|https://*)
    validate_port "$api_base_url" || fail 'API_BASE_URL has an invalid port'
    ;;
esac

case "$api_base_url" in
  */../*|*/..|*/./*|*/.) fail 'API_BASE_URL must not contain dot path segments' ;;
esac

output_path=${1:-/usr/share/nginx/html/runtime-config.js}
temporary_path="${output_path}.tmp"

umask 022
printf 'window.LUKAS_RUNTIME_CONFIG = {"apiBaseUrl":"%s"};\n' "$api_base_url" > "$temporary_path"
chmod 0644 "$temporary_path"
mv "$temporary_path" "$output_path"
