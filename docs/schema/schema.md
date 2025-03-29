# Git-Actions Configuration Schema

This document provides detailed information about the Git-Actions configuration schema.

## Overview

Git-Actions uses two types of configuration files:
1. `server.yaml` - Server and HTTP listener configuration
2. Rule configuration files - Webhook and rules configuration

All configuration files use YAML format with Kubernetes-like structure using `apiVersion` and `kind` fields.

## Server Configuration (`server.yaml`)

The server configuration controls the HTTP server that listens for webhook events.

```yaml
# Server configuration for Git-Actions
apiVersion: "git-actions/v1"
kind: "ServerConfig"

# Server specification
spec:
  port: 8080                # Port number for the HTTP server (integer, required)
  host: "0.0.0.0"           # Host binding address (string, required)
  
  tls:                      # TLS configuration (optional)
    enabled: false          # Whether TLS is enabled (boolean, required)
    cert_file: "/path/to/cert.pem"  # Path to certificate file (string, required if enabled)
    key_file: "/path/to/key.pem"    # Path to key file (string, required if enabled)
  
  logging:                  # Logging configuration (optional)
    level: "INFO"           # Log level: ERROR, WARN, INFO, DEBUG, or TRACE (string, optional)
    format: "json"          # Log format: text, json, or structured (string, optional)
    file: "/var/log/git-actions.log" # Log file path (string, optional)
    
  rule_configs:             # Array of additional rule configuration files to load (array, optional)
    - "/path/to/rules/project-a-config.yaml"
    - "/path/to/rules/project-b-config.yaml"
    - "./rules/common-config.yaml"
```

## Rules Configuration Files

These files define webhook endpoints and the rules that respond to Git events.

### Top-Level Structure

```yaml
# Git-Actions rules configuration
apiVersion: "git-actions/v1"
kind: "RulesConfig"

# Configuration specification
spec:
  webhooks:                  # Webhook endpoint definitions (object, required)
    # webhook configurations...

  rules:                     # Rule definitions (array, required)
    # rule configurations...
```

### Webhook Configuration

```yaml
spec:
  webhooks:
    webhook_id:              # Unique webhook identifier (string, required)
      path: "/webhook/path"  # URL path for the webhook (string, required)
      type: "github"         # Type: "github", "gitlab", or "bitbucket" (string, required)
      # Authentication (depends on type)
      secretFromEnv: "ENV_VAR_NAME"  # For GitHub/GitLab (string, required for GitHub/GitLab)
      # OR for Bitbucket
      usernameFromEnv: "ENV_VAR_NAME" # For Bitbucket (string, required for Bitbucket)
      passwordFromEnv: "ENV_VAR_NAME" # For Bitbucket (string, required for Bitbucket)
```

### Rule Configuration

```yaml
spec:
  rules:
    - name: "rule-name"      # Unique rule name (string, required)
      description: "..."     # Description of the rule (string, optional)
      
      event_types:           # Event types to match (array of strings, required)
        - "push"
        - "pull_request.merged"
      
      webhook: "webhook_id"  # Webhook this rule applies to (string, optional)
      
      branches:              # Branch filters (array of objects, optional)
        - exact: "main"      # Exact branch name
        - pattern: "feature/*" # Glob pattern
        - regex: '^release-\d+\.\d+$' # Regular expression
        - not: "temp/*"      # Negated pattern
      
      paths:                 # Path/file filters (array of objects, optional)
        - exact: "file.txt"  # Exact file path
        - pattern: "src/**/*.js" # Glob pattern
        - regex: '.*\.sql$'  # Regular expression
        - not: "**/*.md"     # Negated pattern
      
      conditions:            # Additional conditions (array of strings, optional)
        - "{{ event.author }} != 'dependabot'"
        - "{{ event.changed_files.length }} > 0"
      
      actions:               # Actions to execute (array of objects, required)
        # HTTP action
        - type: "http"       # Action type (string, required)
          url: "https://example.com/api" # Request URL (string, required)
          method: "POST"     # HTTP method: GET, POST, PUT, DELETE (string, required)
          headers:           # HTTP headers (object, optional)
            Content-Type: "application/json"
            Authorization: "Bearer {{ env.API_TOKEN }}"
          payload:           # Request body (object/string, optional)
            key: "value"
            data: "{{ event.data }}"
          timeout: 30        # Request timeout in seconds (integer, optional)
        
        # Shell command action
        - type: "shell"      # Action type (string, required)
          command: "npm test" # Command to execute (string, required)
          working_dir: "./app" # Working directory (string, optional)
          environment:       # Environment variables (object, optional)
            KEY: "value"
            TOKEN: "{{ env.SECRET_TOKEN }}"
          timeout: 300       # Command timeout in seconds (integer, optional)
```

## Variable Substitution

Git-Actions uses the Tera templating engine for variable substitution with double curly braces:

```
{{ variable }}
```

### Available Variables

- `event` - Event data from the Git webhook
  - Common properties: `event.author`, `event.repository`, `event.branch`, `event.commit_hash`, `event.changed_files`, etc.
  - The exact properties depend on the event type and source (GitHub, GitLab, or Bitbucket)
  
- `env` - Environment variables
  - Access with `{{ env.VAR_NAME }}`
  - Example: `{{ env.API_TOKEN }}`

## Event Types

The following event types are supported depending on the Git platform:

| Event Type             | Description                 |
|------------------------|-----------------------------|
| `push`                 | Code pushed to a branch     |
| `pull_request.opened`  | New PR created              |
| `pull_request.updated` | PR updated with new commits |
| `pull_request.merged`  | PR merged to target branch  |
| `pull_request.closed`  | PR closed without merging   |
| `tag`                  | New tag created             |

## Examples

### GitHub Webhook with Docker Build

```yaml
# Git-Actions rules configuration
apiVersion: "git-actions/v1"
kind: "RulesConfig"

spec:
  webhooks:
    github_repo:
      path: "/webhook/github"
      type: "github"
      secretFromEnv: "GITHUB_SECRET"

  rules:
    - name: "docker-build"
      description: "Build Docker image when Dockerfile changes"
      event_types:
        - "push"
      branches:
        - exact: "main"
      paths:
        - pattern: "Dockerfile"
        - pattern: "docker/**/*"
      actions:
        - type: "shell"
          command: "docker build -t myapp:{{ event.branch }} ."
```

### Monorepo Service-Specific Testing

```yaml
# Git-Actions rules configuration
apiVersion: "git-actions/v1"
kind: "RulesConfig"

spec:
  webhooks:
    github_repo:
      path: "/webhook/github"
      type: "github"
      secretFromEnv: "GITHUB_SECRET"
      
  rules:
    - name: "frontend-tests"
      description: "Run frontend tests when frontend code changes"
      event_types:
        - "pull_request.opened"
        - "pull_request.updated"
      paths:
        - pattern: "frontend/**/*"
      actions:
        - type: "shell"
          command: "cd frontend && npm test"
          working_dir: "/repo/path"
