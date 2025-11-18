#!/run/current-system/sw/bin/fish
# Deploy script: build a combined app (server + web assets) via Nix and upload to the server.
# Requires: Nix with flakes enabled and SSH access to t

# Build the aarch64 combined app defined in flake.nix
nix build .#app-aarch64

# Sync the web assets (dist/) and the server binary to the target
# -L / --copy-links ensures we copy the actual files, not symlinks from symlinkJoin/Nix store
rsync -avzL result/dist result/bin/peer_practice peer_practice@pi4:/data/peer_practice

# Restart the service on the server
ssh root@pi4 "systemctl restart peer-practice"

exit