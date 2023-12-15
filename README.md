# humandiff

This project gives you a human readable git diff output.

It leverages OpenAI GPT-4 to generate a summary of the diff
and suggests a commit message.

## Prerequisites

Make sure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install)
- [Git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git)

And that you have the following environment variables set:

- `OPENAI_API_KEY`: Your OpenAI API key. You can get one [here](https://beta.openai.com/).

## Installation

```bash
  cargo build --release
  cp target/release/humandiff /usr/local/bin # or wherever you want to put it
```

## Usage

Inside a git repo, run:

```bash
  humandiff
```

Example output:

```bash
$ humandiff
Concise explanation: A new README file was added to the repository. This file
includes information about the 'humandiff' project, which is a tool that
generates a human-readable git diff output using GPT-4. It also includes
instructions for installing and using the tool.
Suggested commit message: Added README file
```
