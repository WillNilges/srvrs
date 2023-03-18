#!/bin/bash
set -e
help_me () {
    echo "-d : run in background | -p : prompt (required) | -h : help"
}
SD_BACK="-it"
SD_PROMPT=""
while getopts ":h:dp:" option; do
    case $option in
        d)
            SD_BACK="-d";;
        p) # select file path 
            SD_PROMPT=$OPTARG;;
        h)
            help_me
            exit;;
    esac
done
if [[ -z "$SD_PROMPT" ]]; then help_me; exit 1; fi
podman run --rm $SD_BACK --hooks-dir=/usr/share/containers/oci/hooks.d/ --security-opt=label=disable -v $(dirname $SD_PROMPT):/workdir:Z -e SD2_PROMPT=$(basename $SD_PROMPT) --name="$(whoami)_$(basename $SD_PROMPT)" 'willnilges/srvrs-sd:v1.0'
