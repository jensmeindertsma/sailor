# Setting up a development environment

Becaus Sail consists of a systemd service and socket, it is recommended to develop this project inside a virtual machine, so that any system wide issues are contained and Sail can be free to bind to its ports. Sail currently employs Nginx as a middleware which handles TLS for all requests, then forwards plain HTTP to the Sail daemon, which replies with a plain HTTP response, which is then encrypted by Nginx. Eventually we would like to include this functionality in Sail itself, but for now this allows us to focus on the important features.

## An Ubuntu dev box

Follow these steps to set up your development environment inside a Ubuntu VM. We will leave the provisioning of the virtual machine up to you. Make sure you can access it over SSH, preferably using an alias defined in `~/.ssh/config` on your host machine.

1. Update all packages:
  ```
  sudo apt update
  sudo apt upgrade -y
  ```

2. Install Docker:
   ```
   # Add Docker's official GPG key:
   sudo apt-get update
   sudo apt-get install ca-certificates curl
   sudo install -m 0755 -d /etc/apt/keyrings
   sudo curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
   sudo chmod a+r /etc/apt/keyrings/docker.asc
  
   # Add the repository to Apt sources:
   echo \
    "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu \
    $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
   sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
   sudo apt-get update
   ```
   ```
   sudo apt-get install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
   ```

3. Install Nix:
   ```
   sh <(curl -L https://nixos.org/nix/install) --daemon
   ```
   Then, in a new shell:
   ```
   echo "experimental-features = nix-command flakes" | sudo tee -a /etc/nix/nix.conf
   nix profile install nixpkgs#direnv
   echo 'eval "$(direnv hook bash)"' >> ~/.bashrc
   ```

4. Install the Github CLI:
   ```
   sudo apt install gh
   gh auth login
   ```

5. Then, from the host
   ```
   scp -r .config/git <VM_NAME>:/home/<USER_NAME>/.config/git
   scp -r .config/direnv <VM_NAME>:/home/<USER_NAME>/.config/direnv
   ```
   If you don't have a `~/.config/direnv/direnv.toml` file, here's what's in mine:
   ```toml
   hide_env_diff = true
   warn_timeout = 0
   ```

6. Finally, install the source of Sail:
   ```sh
   mkdir dev
   cd dev
   gh repo clone jensmeindertsma/sail # Clone your fork here instead
   cd sail
   direnv allow
   direnv reload
   just install
   ```
