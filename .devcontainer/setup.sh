DFXVM_INIT_YES=true sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)";
source "$HOME/.local/share/dfx/env";

# Add dfx to shell profile for future sessions
echo 'source "$HOME/.local/share/dfx/env"' >> ~/.bashrc;
echo 'source "$HOME/.local/share/dfx/env"' >> ~/.zshrc;
