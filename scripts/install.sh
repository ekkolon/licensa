#!/bin/bash

# Logging helper
function write_log {
    level=$1
    message=$2

    case $level in
    "info") foreground_color="\e[34m" ;;
    "success") foreground_color="\e[32m" ;;
    "warn") foreground_color="\e[33m" ;;
    "error") foreground_color="\e[31m" ;;
    *) foreground_color="\e[97m" ;; # Default to white
    esac

    timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    formatted_level="[$(echo "$level" | tr '[:lower:]' '[:upper:]')]"

    formatted_message="$timestamp $formatted_level $message"
    echo -e "$foreground_color$formatted_message\e[0m"
}

# GitHub repository details
version="0.1.3"
bin_name="licensa"
release_tag="v$version"

architecture=$(uname -m)

# Adjust asset_name_unpacked based on architecture
case "$architecture" in
"x86_64")
    asset_name_unpacked="${bin_name}-${release_tag}-x86_64-linux"
    ;;
"aarch64")
    asset_name_unpacked="${bin_name}-${release_tag}-aarch64-linux"
    ;;
*)
    write_log "error" "Unsupported architecture: $architecture"
    exit 1
    ;;
esac

asset_name_tar="${asset_name_unpacked}.tar.xz"

release_download_url="https://github.com/ekkolon/licensa/releases/download/$release_tag/$asset_name_tar"

# Target download directory
downloads_folder="/usr/local/src"
download_path="$downloads_folder/$asset_name_tar"

# Prompt the user for confirmation
read -p "This script is set to download and install Licensa CLI $release_tag.
Are you sure you want to proceed? (Type 'Y' for Yes, 'N' for No): " confirmation

if [[ "$confirmation" != 'Y' && "$confirmation" != 'y' ]]; then
    write_log "info" "The installation process has been canceled"
    exit
fi

write_log "info" "Detected architecture: $architecture"

# Download binary from GitHub release
write_log "info" "Downloading assets from GitHub ..."
wget -q "$release_download_url" -O "$download_path"
write_log "success" "Download succeeded!"

# Unpack the binary
write_log "info" "Unpacking..."
tar -xf "$download_path" -C "$downloads_folder"
write_log "success" "Successfully unpacked $asset_name_tar"

write_log "info" "Installing..."
destination_path="/usr/local/bin"

# Move unpacked source to destination
# TODO: Ask if should override, if path exists
source_folder="$downloads_folder/$asset_name_unpacked"
sudo cp -r "$source_folder"/* "$destination_path"

# Remove downloaded tar.gz
rm -rf "$download_path"
rm -rf "$source_folder"

write_log "success" "Licensa CLI has been installed successfully"

exit
