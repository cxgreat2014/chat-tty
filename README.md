The content of this readme was assisted by chat-tty

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/Rust%20Version-Latest-brightgreen.svg)](https://www.rust-lang.org/)
![Crates.io](https://img.shields.io/crates/d/chat-gpt-streamer)


# chat-tty

chat-tty is a terminal-based Rust application that allows you to stream completions from OpenAI's ChatGPT model.

Interact with ChatGPT directly in your terminal and experience lightning-fast completions for any use case, including brainstorming ideas
, generating content or casual conversations with the AI. Harness ChatGPT's capabilities quickly and easily in this lightweight console-b
ased interface.

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
- [License](#license)

## Installation

To build, run and install this application, you need to have the latest version of Rust installed on your computer. You can install Rust 
by following the instructions at [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).


Now, you are ready to install the application!

```bash
cargo install chat-tty
```

## Usage

Before using the chat-tty, you need to provide a valid OpenAI API key. Set the `OPENAI_KEY` environment variable to your Open
AI API key:

```bash
export OPENAI_KEY=your_api_key
```

Now, you can simply run the application by executing:

```bash
chat-tty
```

## License

This project is licensed under the MIT License. See the [`LICENSE`](LICENSE) file for more details.

