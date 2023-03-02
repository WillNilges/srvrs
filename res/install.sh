#!/bin/bash

set -e

APP=srvrs

cargo build --release

sudo mkdir -p /var/srvrs/whisper /var/srvrs/scripts /var/srvrs/work
sudo cp res/run_whisper_container.sh /var/srvrs/scripts
sudo cp res/srvrs-whisper.service /etc/systemd/system/
sudo install target/release/$APP /usr/local/sbin/$APP 
sudo chmod u+s /usr/local/sbin/$APP && echo "Installed $APP."
sudo systemctl enable --now srvrs-whisper
