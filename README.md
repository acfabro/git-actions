# Git-Actions

A Rust-based automation tool that listens for Git events (like pushes, pull requests) from platforms such as Bitbucket
and executes configurable actions based on customizable rules.

## Features

- Listens for webhook events from Git platforms (Bitbucket, GitHub, etc.)
- Executes configurable actions (shell commands, HTTP requests) based on event data
- Supports powerful rule matching with branch patterns, path patterns, and event types
- Uses templating for dynamic values in commands and HTTP requests
- Kubernetes-inspired configuration structure

## Configuration

For config documentation, see [Configuration](docs/schema/README.md).

## Running the app

Assuming you already have the requisite yaml config files, to compile and install:

```bash
cargo install --path .

git-actions -c /path/to/config.yaml
```

To just run the app without installing, you can use:

```bash
cargo run -- -c /path/to/config.yaml
```

With docker:

```bash
# Build the image
docker build -t git-actions .

# Run the container
docker run -p 8080:8080 \
  -v $(pwd)/server.yaml:/server.yaml \
  -v $(pwd)/rules.yaml:/rules.yaml \
  -v $(pwd)/webhooks.yaml:/webhooks.yaml \
  git-actions
```

## Command-line Options

- `-c, --config <FILE>`: Path to the server configuration file (default: `server.yaml`)
- `-h, --help`: Print help information
- `-V, --version`: Print version information

## Planned TODOs

1. Templating for dynamic values
2. Add more webhook types (GitHub, GitLab, etc.)
3. Implement more action types (shell, kubernetes, etc)
4. Action queueing and retry logic
5. Implement as a Kubernetes operator (?)
6. A lot of `TODO`s in the code

## License

This project is licensed under the MIT License - see the LICENSE file for details.
