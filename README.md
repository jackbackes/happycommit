_Note: HappyCommit works but is still early. Check any generated commit message thoroughly before saving it! And we're always open for a PR. Thanks for using and contributing!_

# Welcome to HappyCommit! 😊 

HappyCommit is a delightful tool that harnesses the power of OpenAI's GPT-3.5 Turbo language model to generate meaningful and descriptive Git commit messages for you. With HappyCommit, you can now focus on writing great code while we take care of crafting the perfect commit messages.

## Demo 1: Introducing HappyCommit

https://user-images.githubusercontent.com/13596692/232681973-0738f7dd-4755-4466-bbef-797cdf8440d4.mov

## Demo 2: Long diffs

https://user-images.githubusercontent.com/13596692/232682150-ce2dfdb7-5f3c-489c-9cbf-8b53a17a0ea9.mov

## Prerequisites

- Make sure you have [Homebrew](https://brew.sh/) installed on your macOS system.

## Getting Started (macOS)

1. To begin your journey with HappyCommit, install it by running the following commands:

   ```bash
   brew tap jackbackes/git-commit-gpt
   brew install jackbackes/homebrew-git-commit-gpt/git-commit-gpt.rb
   ```

2. After the installation, let's add your OpenAI API key to the configuration file. Simply run the following command and enter your API key when prompted:

   ```bash
   printf "Enter your OPENAI_API_KEY: " && read -rs api_key && mkdir ~/.happycommit >> /dev/null && echo "OPENAI_API_KEY = \\\"$api_key\\\"" >> ~/.happycommit/config.toml
   ```

   Don't have an OpenAI API key yet? No worries! Grab one from your [OpenAI account](https://beta.openai.com/account/api-keys).

3. That's it! HappyCommit is now ready to make your Git experience more enjoyable. Just use the `git commit-gpt` command to generate and add a commit message based on your code changes.

### Getting Started (Linux)

...Coming soon!

### Getting Started (Windows)

...Coming soon!

### Getting Started (Build from source)

1. Install Rust and Cargo by following the instructions [here](https://www.rust-lang.org/tools/install).
2. Install the "Just" task runner by following the instructions [here](https://github.com/casey/just) or just run `cargo install just`.

```bash
git clone git@github.com:jackbackes/happycommit.git
cd happycommit
just install
./add-openai-api-key.sh
```

## Usage

Instead of running the standard `git commit` command, use this friendly alternative:

```bash
git add .
git commit-gpt
```

HappyCommit will analyze your staged changes and generate a meaningful commit message that brings a smile to your face.

## Contributing

We'd love for you to join us in making HappyCommit even better! If you have suggestions, feature requests, or bug reports, please feel free to open an issue or submit a pull request on our GitHub repository. We're excited to see your contributions!

## License

HappyCommit is warmly licensed under the Apache License 2.0.
