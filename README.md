# Git-Actions

A Rust-based automation tool that listens for Git events (like pushes, pull requests) from platforms such as Bitbucket and executes configurable actions based on customizable rules.

## Features

- Listens for webhook events from Git platforms (Bitbucket, GitHub, etc.)
- Executes configurable actions (shell commands, HTTP requests) based on event data
- Supports powerful rule matching with branch patterns, path patterns, and event types
- Uses templating for dynamic values in commands and HTTP requests
- Kubernetes-inspired configuration structure

## Configuration

Git-Actions uses YAML configuration files:

1. **Server Configuration** (`server.yaml`): Configures the HTTP server that listens for webhook events
2. **Webhook Configurations** (`webhooks/*.yaml`): Defines webhook endpoints and authentication
3. **Rules Configuration** (`rules.yaml`): Defines rules that respond to Git events

For detailed schema documentation, see [Configuration Schema](docs/schema/README.md).

### Basic Configuration Example

```yaml
# server.yaml
apiVersion: git-actions/v1
kind: Server
metadata:
  name: "git-actions-server"
spec:
  port: 8080
  host: "0.0.0.0"
  logging:
    level: "INFO"
  configs:
    - "webhooks/*.yaml"
    - "rules.yaml"
```

## Running with Docker

### Using Pre-built Image

```bash
# Pull the image
docker pull your-registry/git-actions:latest

# Run with configuration mounted from host
docker run -p 8080:8080 \
  -v $(pwd)/server.yaml:/server.yaml \
  -v $(pwd)/rules.yaml:/rules.yaml \
  -v $(pwd)/webhooks:/webhooks \
  your-registry/git-actions:latest
```

### Building the Docker Image

```bash
# Build the image
docker build -t git-actions .

# Run the container
docker run -p 8080:8080 \
  -v $(pwd)/server.yaml:/server.yaml \
  -v $(pwd)/rules.yaml:/rules.yaml \
  -v $(pwd)/webhooks:/webhooks \
  git-actions
```

### Environment Variables

When using Docker, you can pass environment variables for webhook tokens and API authentication:

```bash
docker run -p 8080:8080 \
  -v $(pwd)/config:/config \
  -e BITBUCKET_API_TOKEN=your_token \
  -e BITBUCKET_MYREPO_TOKEN=webhook_token \
  your-registry/git-actions:latest
```

## Running Locally

```bash
# Run with default configuration (server.yaml in current directory)
git-actions

# Run with custom configuration file
git-actions --config /path/to/server.yaml
```

## Command-line Options

- `-c, --config <FILE>`: Path to the server configuration file (default: `server.yaml`)
- `-h, --help`: Print help information
- `-V, --version`: Print version information

## Planned TODOs

1. Add more webhook types (GitHub, GitLab, etc.)
2. Implement more action types (shell, kubernetes, etc)
3. Action queueing and retry logic
4. Implement as a Kubernetes operator (?)
5. A lot of `TODO`s in the code

## License

This project is licensed under the MIT License - see the LICENSE file for details.