help:
    just --list

build:
    cargo build

build-release:
    cargo build --release

install: uninstall build
    #!/usr/bin/env bash
    echo "Installing Sail"
    sudo groupadd sail 2>/dev/null
    sudo usermod -aG sail $USER
    sudo cp /home/jens/dev/sail/target/debug/sail /usr/local/bin/sail
    sudo cp /home/jens/dev/sail/target/debug/saild /usr/local/bin/saild
    sudo cp /home/jens/dev/sail/install/systemd.service /usr/lib/systemd/system/sail.service
    sudo cp /home/jens/dev/sail/install/systemd.socket /usr/lib/systemd/system/sail.socket
    sudo systemctl daemon-reload
    # VERY important, it won't be enough to have the service be triggered by the socket,
    # because that doesn't cover the network requests.
    sudo systemctl enable --now sail

uninstall: stop
    #!/usr/bin/env bash
    echo "Uninstalling Sail"
    sudo rm -f /usr/local/bin/sail
    sudo rm -f /usr/local/bin/saild
    sudo rm -f /usr/lib/systemd/system/sail.service
    sudo rm -f /usr/lib/systemd/system/sail.socket
    sudo systemctl daemon-reload
    sudo systemctl reset-failed

start: stop
    sudo systemctl start sail

stop:
    #!/usr/bin/env bash
    sudo systemctl stop sail.socket
    sudo systemctl stop sail.service
    exit 0

status type:
    sudo systemctl status sail.{{type}}

