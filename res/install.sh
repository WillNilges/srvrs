#!/bin/bash

set -e

APP=srvrs

# Directories to create/modify
BASE=/var/srvrs
SERVICE="$BASE/whisper"
SCRIPTS="$BASE/scripts"
WORK="$BASE/work"
DIST="$BASE/distributor"
STATUS="$BASE/status"

cargo build --release

sudo useradd $APP | echo "User already added"

# Create the necessary directories and set permissions
sudo rm -rf $SERVICE $SCRIPTS $WORK $DIST $STATUS
sudo mkdir -p $SERVICE $SCRIPTS $WORK $DIST
sudo touch $STATUS

# Modifications for the daemon's directories.
sudo chown -R $APP:$APP $SCRIPTS $WORK $DIST $STATUS
sudo chmod 700 $SCRIPTS $WORK $DIST

# The directory that users will write to is special.
sudo chown -R $APP:member $SERVICE $STATUS
sudo chmod 730 $SERVICE

# Status file to let users know what's poppin'
sudo chmod 740 $STATUS


# Install script, systemd service, and binary
sudo cp res/run_whisper_container.sh /var/srvrs/scripts
sudo cp res/srvrs-whisper.service /etc/systemd/system/
sudo cp res/srvrs-distributor.service /etc/systemd/system/
sudo install target/release/$APP /usr/local/sbin/$APP 
sudo install target/release/$APP-distributor /usr/local/sbin/$APP-distributor

# Launch srvrs!
sudo systemctl enable srvrs-whisper
sudo systemctl enable srvrs-distributor

sudo systemctl restart srvrs-whisper
sudo systemctl restart srvrs-distributor

echo "Installed $APP."
