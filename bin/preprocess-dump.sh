#!/bin/sh

set -euxo pipefail

cat $1 | zstd -d | jq --compact-output '.created_utc = (.created_utc | tonumber)' > "$2"