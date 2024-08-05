help:
    just --list

build:
    cargo build

build-release:
    cargo build --release

install: build
    #!/usr/bin/env bash
    echo "Installing Sail"
    sudo groupadd sail 2>/dev/null
    sudo usermod -aG sail $USER
    sudo systemctl stop sail.socket
    sudo systemctl stop sail.service
    sudo cp /home/jens/dev/sail/target/debug/sail /usr/local/bin/sail
    sudo cp /home/jens/dev/sail/target/debug/saild /usr/local/bin/saild
    sudo cp /home/jens/dev/sail/install/systemd.service /usr/lib/systemd/system/sail.service
    sudo cp /home/jens/dev/sail/install/systemd.socket /usr/lib/systemd/system/sail.socket
    sudo systemctl daemon-reload
    sudo systemctl reset-failed
    
    sudo systemctl enable --now sail

status type:
    sudo systemctl status sail.{{type}}

