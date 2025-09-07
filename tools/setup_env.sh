#!/usr/bin/env bash
set -euo pipefail
# https://solana.com/zh/docs/intro/installation#solana-cli-basics
########################################
# Logging Functions
########################################
log_info() {
    printf "[INFO] %s\n" "$1"
}

log_error() {
    printf "[ERROR] %s\n" "$1" >&2
}


########################################
# OS Detection
########################################
detect_os() {
    local os
    os="$(uname)"
    if [[ "$os" == "Linux" ]]; then
        echo "Linux"
    elif [[ "$os" == "Darwin" ]]; then
        echo "Darwin"
    else
        echo "$os"
    fi
}

########################################
# Install OS-Specific Dependencies
########################################
install_dependencies() {
    local os="$1"
    if [[ "$os" == "Linux" ]]; then
        log_info "Detected Linux OS. Updating package list and installing dependencies..."
        SUDO=""
        if command -v sudo >/dev/null 2>&1; then
            SUDO="sudo"
        fi
        $SUDO apt-get update 
        $SUDO apt-get install -y \
                build-essential \
                pkg-config \
                libudev-dev \
                llvm \
                libclang-dev \
                protobuf-compiler \
                libssl-dev
    elif [[ "$os" == "Darwin" ]]; then
        log_info "Detected macOS."
    else
        log_info "Detected $os."
    fi

    echo ""
}

########################################
# Install Rust via rustup
########################################
install_rust() {
    if command -v rustc >/dev/null 2>&1; then
        log_info "Rust is already installed. Updating..."
        rustup update
    else
        log_info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        log_info "Rust installation complete."

        rustup toolchain install nightly-2025-04-01
        rustup default nightly-2025-04-01
    fi

    # Source the Rust environment
    if [[ -f "$HOME/.cargo/env" ]]; then
        . "$HOME/.cargo/env"
    elif [[ -f "$HOME/.cargo/env.fish" ]]; then
        log_info "Sourcing Rust environment for Fish shell..."
        source "$HOME/.cargo/env.fish"
    else
        log_error "Rust environment configuration file not found."
    fi

    if command -v rustc >/dev/null 2>&1; then
        rustc --version
    else
        log_error "Rust installation failed."
    fi

    echo ""
}

########################################
# Install Solana CLI
########################################
install_solana_cli() {
    local os="$1"

    if command -v solana >/dev/null 2>&1; then
        log_info "Solana CLI is already installed. Updating..."
        agave-install update
    else
        log_info "Installing Solana CLI..."
        sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
        log_info "Solana CLI installation complete."
    fi

    if command -v solana >/dev/null 2>&1; then
        solana --version
    else
        log_error "Solana CLI installation failed."
    fi

    if [[ "$os" == "Linux" ]]; then
        export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
    elif [[ "$os" == "Darwin" ]]; then
        echo 'export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"' >> ~/.zshrc
    fi

    echo ""
}

########################################
# Install Anchor CLI
########################################
install_anchor_cli() {
    if command -v anchor >/dev/null 2>&1; then
        log_info "Anchor CLI is already installed. Updating..."
        if ! command -v avm >/dev/null 2>&1; then
            log_info "AVM is not installed. Installing AVM..."
            cargo install --force --git https://github.com/coral-xyz/anchor avm
        fi
        avm install 0.30.1
        avm use 0.30.1
    else
        log_info "Installing Anchor CLI..."
        cargo install --git https://github.com/coral-xyz/anchor avm
        avm install 0.30.1
        avm use 0.30.1
        log_info "Anchor CLI installation complete."
    fi

    if command -v anchor >/dev/null 2>&1; then
        anchor --version
    else
        log_error "Anchor CLI installation failed."
    fi

    echo ""
}

########################################
# Install nvm and Node.js
########################################
install_nvm_and_node() {
    if [ -s "$HOME/.nvm/nvm.sh" ]; then
        log_info "NVM is already installed."
    else
        log_info "Installing NVM..."
        curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/master/install.sh | bash
    fi

    export NVM_DIR="$HOME/.nvm"
    # Immediately source nvm and bash_completion for the current session
    if [ -s "$NVM_DIR/nvm.sh" ]; then
        . "$NVM_DIR/nvm.sh"
    else
        log_error "nvm not found. Ensure it is installed correctly."
    fi

    if [ -s "$NVM_DIR/bash_completion" ]; then
        . "$NVM_DIR/bash_completion"
    fi

    if command -v node >/dev/null 2>&1; then
        local current_node
        current_node=$(node --version)
        local latest_node
        latest_node=$(nvm version-remote node)
        if [ "$current_node" = "$latest_node" ]; then
            log_info "Latest Node.js ($current_node) is already installed."
        else
            log_info "Updating Node.js: Installed ($current_node), Latest ($latest_node)."
            nvm install node
            nvm alias default node
            nvm use default
        fi
    else
        log_info "Installing Node.js via NVM..."
        nvm install node
        nvm alias default node
        nvm use default
    fi

    echo ""
}


########################################
# Install Yarn
########################################
install_yarn() {
    if command -v yarn >/dev/null 2>&1; then
        log_info "Yarn is already installed."
    else
        log_info "Installing Yarn..."
        npm install --global yarn
    fi

    if command -v yarn >/dev/null 2>&1; then
        yarn --version
    else
        log_error "Yarn installation failed."
    fi

    echo ""
}

########################################
# Print Installed Versions
########################################
print_versions() {
    echo ""
    echo "Installed Versions:"
    echo "Rust: $(rustc --version 2>/dev/null || echo 'Not installed')"
    echo "Solana CLI: $(solana --version 2>/dev/null || echo 'Not installed')"
    echo "Anchor CLI: $(anchor --version 2>/dev/null || echo 'Not installed')"
    echo "Node.js: $(node --version 2>/dev/null || echo 'Not installed')"
    echo "Yarn: $(yarn --version 2>/dev/null || echo 'Not installed')"
    echo ""
}

########################################
# Append nvm Initialization to the Correct Shell RC File
########################################
ensure_nvm_in_shell() {
    local shell_rc=""
    if [[ "$SHELL" == *"zsh"* ]]; then
        shell_rc="$HOME/.zshrc"
    elif [[ "$SHELL" == *"bash"* ]]; then
        shell_rc="$HOME/.bashrc"
    else
        shell_rc="$HOME/.profile"
    fi

    if [ -f "$shell_rc" ]; then
        if ! grep -q 'export NVM_DIR="$HOME/.nvm"' "$shell_rc"; then
            log_info "Appending nvm initialization to $shell_rc"
            {
                echo ''
                echo 'export NVM_DIR="$HOME/.nvm"'
                echo '[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"  # This loads nvm'
            } >> "$shell_rc"
        fi
    else
        log_info "$shell_rc does not exist, creating it with nvm initialization."
        echo 'export NVM_DIR="$HOME/.nvm"' > "$shell_rc"
        echo '[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"  # This loads nvm' >> "$shell_rc"
    fi
}

########################################
# Main Execution Flow
########################################
main() {
    local os
    os=$(detect_os)

    install_dependencies "$os"
    install_rust
    install_solana_cli "$os"
    install_anchor_cli
    install_nvm_and_node
    install_yarn

    ensure_nvm_in_shell

    print_versions

    echo 'export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"' >> $HOME/.bashrc
    source $HOME/.bashrc

    echo "Installation complete. Please restart your terminal to apply all changes."
}

main "$@"
