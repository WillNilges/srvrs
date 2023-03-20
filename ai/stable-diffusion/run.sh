#!/bin/bash
set -e
if [[ -z "$1" ]]; then echo "Need file argument"; exit 1; fi
podman run --rm $SD_BACK --hooks-dir=/usr/share/containers/oci/hooks.d/ --security-opt=label=disable -v $(dirname $SD_PROMPT):/workdir:Z -e SD2_PROMPT=$(basename $SD_PROMPT) --name="$(whoami)_$(basename $SD_PROMPT)" 'willnilges/srvrs-sd:v1.0'
