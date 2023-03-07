#!/bin/bash

set -e

APP=srvrs

# Directories to create/modify
BASE=/var/srvrs
SERVICE="$BASE/whisper"
SCRIPTS="$BASE/scripts"
WORK="$BASE/work"

cargo build --release

sudo useradd $APP | echo "User already added"

# Create the necessary directories and set permissions
sudo mkdir -p $SERVICE $SCRIPTS $WORK
sudo chown -R $APP:$APP $SCRIPTS $WORK 
sudo chmod 730 $SERVICE
sudo chown -R :member $SERVICE
sudo chmod 700 $SCRIPTS $WORK

# Install script, systemd service, and binary
sudo cp res/run_whisper_container.sh /var/srvrs/scripts
sudo cp res/srvrs-whisper.service /etc/systemd/system/
sudo install target/release/$APP /usr/local/sbin/$APP 

# Launch srvrs!
sudo systemctl enable --now srvrs-whisper

echo "Installed $APP."
