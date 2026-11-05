#!/bin/sh
set -eu

script_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
generator="$script_dir/docker-entrypoint.d/40-runtime-config.sh"
temporary_dir=$(mktemp -d)
output_path="$temporary_dir/runtime-config.js"
trap 'rm -rf "$temporary_dir"' EXIT HUP INT TERM

expect_failure() {
  if "$@" >/dev/null 2>&1; then
    printf 'Expected command to fail: %s\n' "$*" >&2
    exit 1
  fi
}

env API_UPSTREAM='http://api:8080' API_BASE_URL='/api' \
  DATABASE_URL='postgres://private-user:private-password@db/private' \
  sh "$generator" "$output_path"

grep -Fq '"apiBaseUrl":"/api"' "$output_path"
if grep -Fq 'private-password' "$output_path"; then
  printf '%s\n' 'Runtime config unexpectedly contains a secret' >&2
  exit 1
fi

env API_UPSTREAM='https://api.example.com:443' \
  API_BASE_URL='https://api.example.com/api' \
  sh "$generator" "$output_path"
grep -Fq '"apiBaseUrl":"https://api.example.com/api"' "$output_path"

grep -Fq 'proxy_ssl_server_name on;' "$script_dir/nginx/api-proxy.conf"
grep -Fq 'proxy_ssl_name $proxy_host;' "$script_dir/nginx/api-proxy.conf"
grep -Fq 'proxy_ssl_verify on;' "$script_dir/nginx/api-proxy.conf"
grep -Fq 'proxy_ssl_trusted_certificate /etc/ssl/certs/ca-certificates.crt;' \
  "$script_dir/nginx/api-proxy.conf"
grep -Fq 'proxy_set_header Host $proxy_host;' "$script_dir/nginx/api-proxy.conf"
grep -Fq 'apk add --no-cache ca-certificates' "$script_dir/Dockerfile"

expect_failure env API_UPSTREAM='http://api:8080' API_BASE_URL='";alert(1);//' \
  sh "$generator" "$output_path"
expect_failure env API_UPSTREAM='http://api:99999' API_BASE_URL='/api' \
  sh "$generator" "$output_path"
expect_failure env API_UPSTREAM='http://user:pass@api:8080' API_BASE_URL='/api' \
  sh "$generator" "$output_path"

printf '%s\n' 'runtime-config checks passed'
