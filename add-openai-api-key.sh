#!/bin/sh

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

printf "API Key added successfully. Happy committing!\n"