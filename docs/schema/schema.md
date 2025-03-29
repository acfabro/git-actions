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
# nonk8s
# Server configuration for Git-Actions
apiVersion: "git-actions/v1"
kind: "ServerConfig"

# Server specification
spec:
  port: 8080                # Port number for the HTTP server (integer, required)
  host: "0.0.0.0"           # Host binding address (string, required)

  tls: # TLS configuration (optional)
    enabled: false          # Whether TLS is enabled (boolean, required)
    cert_file: "/path/to/cert.pem"  # Path to certificate file (string, required if enabled)
    key_file: "/path/to/key.pem"    # Path to key file (string, required if enabled)

  logging: # Logging configuration (optional)
    level: "INFO"           # Log level: ERROR, WARN, INFO, DEBUG, or TRACE (string, optional)
    format: "json"          # Log format: text, json, or structured (string, optional)
    file: "/var/log/git-actions.log" # Log file path (string, optional)

  rule_configs: # Array of additional rule configuration files to load (array, optional)
    - "/path/to/rules/project-a-config.yaml"
    - "/path/to/rules/project-b-config.yaml"
    - "./rules/common-config.yaml"
```

## Rules Configuration Files

These files define webhook endpoints and the rules that respond to Git events.

### Top-Level Structure

```yaml
# nonk8s
# Git-Actions rules configuration
apiVersion: "git-actions/v1"
kind: "RulesConfig"

# Configuration specification
spec:
  webhooks:                  # Webhook endpoint definitions (object, required)
  # webhook configurations...

  rules:                     # Rule definitions (object, required)
  # rule configurations...
```

### Webhook Configuration

```yaml
spec:
  webhooks:
    # Unique identifier for the webhook configuration (string, required key)
    webhook_id:
      path: "/webhook/path"  # URL path for the webhook (string, required)
      # Type: "bitbucket" (required). Note: Only "bitbucket" (Server/DC) is supported in V1.
      # Support for "github", "gitlab", and Bitbucket Cloud is planned for future versions.
      type: "bitbucket"
      # Authentication: Use secretFromEnv for HMAC signature validation (recommended for Bitbucket Server/DC)
      secretFromEnv: "BITBUCKET_SECRET_ENV_VAR"  # Env var containing the webhook secret (string, required)
      # usernameFromEnv/passwordFromEnv might be needed for specific Bitbucket API interactions in future actions,
      # but secretFromEnv is primary for webhook validation.
```

### Rule Configuration

```yaml
spec:
  rules:
    "rule-name":            # Unique rule name as the key (string, required)
      description: "..."     # Description of the rule (string, optional)
      
      # Webhooks this rule applies to (array of strings, required).
      # Each string must match a key in the top-level 'spec.webhooks' object.
      webhooks: ["webhook_id_1", "webhook_id_2"]
      
      # Event types to match (array of strings, required). Use hyphens.
      event_types:
        - "push"
        - "pull_request_merged"
      
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
      
      # Additional conditions using Tera expressions (array of strings, optional).
      # NOTE: This field is parsed but IGNORED by the rule matching logic in V1.
      # Condition evaluation is planned for a future version.
      conditions:
        - "{{ event.author != 'dependabot' }}"
        - "{{ event.changed_files | length > 0 }}"
      
      actions:               # Actions to execute (array of objects, required)
        # HTTP action
        - http:              # Action type is the key
            url: "https://example.com/api" # Request URL (string, required)
            method: "POST"     # HTTP method: GET, POST, PUT, DELETE (string, required)
            headers:           # HTTP headers (object, optional)
              Content-Type: "application/json"
              Authorization: "Bearer {{ env.API_TOKEN }}"
            body:              # Request body (object/string, optional)
              key: "value"
              data: "{{ event.data }}"
            # timeout: (Removed - Universal default timeout applies in V1)
        
        # Shell command action
        - shell:             # Action type is the key
            command: "npm test" # Command to execute (string, required)
            working_dir: "./app" # Working directory (string, optional)
            environment:       # Environment variables (object, optional)
              KEY: "value"
              TOKEN: "{{ env.SECRET_TOKEN }}"
            # timeout: (Removed - Universal default timeout applies in V1)
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
| `pull_request_opened`  | New PR created              |
| `pull_request_updated` | PR updated with new commits |
| `pull_request_merged`  | PR merged to target branch  |
| `pull_request_closed`  | PR closed without merging   |
| `tag`                  | New tag created             |
# Note: Event type names depend on the source (Bitbucket, GitHub, GitLab).
# These are examples; refer to provider documentation and normalized event structure.

## Examples

### GitHub Webhook with Docker Build

```yaml
# nonk8s
# Git-Actions rules configuration
apiVersion: "git-actions/v1"
kind: "RulesConfig"

spec:
  webhooks:
    # Define the webhook source
    github_repo_example: # Unique ID for this webhook
      path: "/webhook/github-example"
      type: "github" # Non-functional in V1, illustrative purposes only
      secretFromEnv: "GITHUB_SECRET"

  rules:
    "docker-build-on-main-push":
      description: "Build Docker image when Dockerfile changes on main branch"
      webhooks: [ "github_repo_example" ] # Link rule to the webhook ID
      event_types:
        - "push" # Use hyphenated if applicable (e.g., pull_request_opened)
      branches:
        - exact: "main"
      paths:
        - pattern: "Dockerfile"
        - pattern: "docker/**/*"
      # No 'conditions' field used in V1 logic
      actions:
        - shell: # Use 'shell:' as the key
            command: "docker build -t myapp:{{ event.branch }} ."
            # No 'timeout' field needed
```

### Monorepo Service-Specific Testing (Example)

```yaml
# Git-Actions rules configuration
apiVersion: "git-actions/v1"
kind: "RulesConfig"

spec:
  webhooks:
    # Define the webhook source
    monorepo_webhook: # Unique ID
      path: "/webhook/monorepo"
      type: "bitbucket" # Assuming Bitbucket for V1 compatibility
      secretFromEnv: "MONOREPO_SECRET"
      
  rules:
    "frontend-tests-on-pr":
      description: "Run frontend tests when frontend code changes in a PR"
      webhooks: ["monorepo_webhook"] # Link rule to the webhook ID
      event_types:
        - "pull_request_opened" # Use hyphenated event types
        - "pull_request_updated"
      paths:
        - pattern: "frontend/**/*"
      # No 'conditions' field used in V1 logic
      actions:
        - shell: # Use 'shell:' as the key
            command: "cd frontend && npm test"
            working_dir: "/repo/path" # Example working directory
            # No 'timeout' field needed
