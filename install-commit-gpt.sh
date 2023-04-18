#!/bin/bash

# The user should always run as "source" and not execute directly
if [ "$0" = "$BASH_SOURCE" ]; then
    printf "Please run this script as 'source install-commit-gpt.sh'\n"
    exit 1
fi

# Check if happycommit is in the user's PATH
if ! command -v happycommit &> /dev/null; then
    printf "happycommit not found in your PATH. Please install it first.\n"
    exit 1
fi

current_shell=$(basename "$SHELL")

# Add the git commit-gpt command to the user's shell configuration file
config_file=""
case "$current_shell" in
    bash)
        config_file="$HOME/.bashrc"
        ;;
    zsh)
        config_file="$HOME/.zshrc"
        ;;
    *)
        printf "Unsupported shell. Please add manually.\n"
        exit 1
        ;;
esac

# Check if it's Darwin so sed works properly
unameOut="$(uname -s)"
case "${unameOut}" in
    Darwin*)    machine=Mac;;
    *)          machine="UNKNOWN:${unameOut}"
esac

printf 'Installing git command and adding to your "$PATH"\n'
# Create the directory ~/.happycommit/bin and copy the happycommit binary to it
mkdir -p ~/.happycommit/bin
cp git-commit-gpt ~/.happycommit/bin
chmod +x ~/.happycommit/bin/git-commit-gpt

# Add the directory to the user's PATH
# if path is already in the PATH, don't add it again
if ! echo "$PATH" | grep -q "$HOME/.happycommit/bin"; then
    printf "export PATH=\"$HOME/.happycommit/bin:\$PATH\"\n" >> "$config_file"
fi

# Reload the shell configuration file
source "$config_file"

# make sure git commit-gpt exists
if ! command -v git-commit-gpt &> /dev/null; then
    printf "git commit-gpt not found in your PATH. Please add it manually.\n"
    exit 1
fi

# Ask for user's OPENAI_API_KEY so it can be added to happycommit's config file
# Add the OPENAI_API_KEY to happycommit's config file
# if it isn't there already.
# If it is there, let the user know.

# First check if the config file exists
if [ ! -f "$HOME/.happycommit/config.toml" ]; then
    printf "Creating config file at $HOME/.happycommit/config.toml\n"
    mkdir -p "$HOME/.happycommit"
    touch "$HOME/.happycommit/config.toml"
fi

OPENAI_API_KEY=""
# Check if the OPENAI_API_KEY is already in the config file
if ! grep -qF "OPENAI_API_KEY" "$HOME/.happycommit/config.toml"; then
    # Read the OPENAI_API_KEY from the user
    printf "Please enter your OPENAI_API_KEY (you can get it at https://beta.openai.com/account/api-keys):\nNote: Your input will not be shown on the screen.\n> "
    read -rs OPENAI_API_KEY
    printf "\n"
    # Add the OPENAI_API_KEY to the config.toml file
    printf "OPENAI_API_KEY = \"%s\"\n" "$OPENAI_API_KEY" >> "$HOME/.happycommit/config.toml"
else
    printf "OPENAI_API_KEY already exists in $HOME/.happycommit/config.toml\n"
    printf "Would you like to update it? (y/n) "
    read -r update
    if [ "$update" = "y" ]; then
        # TODO: Remove code duplication
        printf "Please enter your OPENAI_API_KEY (you can get it at https://beta.openai.com/account/api-keys):\nNote: Your input will not be shown on the screen.\n> "
        read -rs OPENAI_API_KEY
        printf "\n"
        # Remove the previous OPENAI_API_KEY from the config file
        if [ "$machine" = "Mac" ]; then
            sed -i '' '/OPENAI_API_KEY/d' "$HOME/.happycommit/config.toml"
        else
            sed -i '/OPENAI_API_KEY/d' "$HOME/.happycommit/config.toml"
        fi
        # Add the OPENAI_API_KEY to the config file
        printf "OPENAI_API_KEY = \"%s\"\n" "$OPENAI_API_KEY" >> "$HOME/.happycommit/config.toml"
    fi
fi

# Done!
printf "git commit-gpt installed successfully. You can now use 'git commit-gpt' to run happycommit.\n"
