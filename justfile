build:
    #!/usr/bin/env bash
    echo "Compiling Sailor"
    cargo build

build-release:
    cargo build --release

install: uninstall build
    #!/usr/bin/env bash
    echo "Installing Sailor"
    sudo groupadd sailors 2>/dev/null
    sudo usermod -aG sailors $USER
    sudo cp /home/jens/dev/sailor/target/debug/sailor /usr/local/bin/sailor
    sudo cp /home/jens/dev/sailor/target/debug/sailord /usr/local/bin/sailord
    sudo cp /home/jens/dev/sailor/install/systemd.service /usr/lib/systemd/system/sailor.service
    sudo cp /home/jens/dev/sailor/install/systemd.socket /usr/lib/systemd/system/sailor.socket
    sudo systemctl daemon-reload
    # VERY important, it won't be enough to have the service be triggered by the socket,
    # because that doesn't cover the network requests.
    sudo systemctl enable --now sailor

uninstall: stop
    #!/usr/bin/env bash
    echo "Uninstalling Sailor"
    sudo rm -f /usr/local/bin/sailor
    sudo rm -f /usr/local/bin/sailord
    sudo rm -f /usr/lib/systemd/system/sailor.service
    sudo rm -f /usr/lib/systemd/system/sailor.socket
    sudo systemctl daemon-reload
    sudo systemctl reset-failed

start: stop
    sudo systemctl start sailor

stop:
    #!/usr/bin/env bash
    sudo systemctl stop sailor.socket
    sudo systemctl stop sailor.service
    exit 0

status type:
    sudo systemctl status sailor.{{type}}

