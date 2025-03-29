# Git-Actions Configuration Schema

This document provides detailed information about the Git-Actions configuration schema.

## Overview

Git-Actions uses three main types of configuration resources, typically defined in separate YAML files:
1. `ServerConfig` - Defines server settings, logging, and paths to other configurations. Usually in `server.yaml`.
2. `WebhookConfig` - Defines individual webhook endpoints, their type, authentication, and API details. Often placed in a `webhooks/` directory (e.g., `webhooks/bitbucket-repo-a.yaml`).
3. `RulesConfig` - Defines rules that trigger actions based on events from specific webhooks. Usually in `rules.yaml` or similar.

All configuration files use YAML format with a Kubernetes-inspired structure using `apiVersion`, `kind`, and optional `metadata` fields.

## 1. Server Configuration (`ServerConfig`)

The `ServerConfig` resource controls the main application server, including the HTTP listener, TLS, logging, and references to other configuration files.

```yaml
# Example: server.yaml
apiVersion: git-actions/v1
kind: Server
metadata:  # Optional metadata for the server configuration
  name: "git-actions-server"
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

  config_files: # Globs pointing to WebhookConfig and RulesConfig files (array, optional)
    - "webhooks/*.yaml"
    - "rules.yaml"
```

## 2. Webhook Configuration (`WebhookConfig`)

The `WebhookConfig` resource defines a specific endpoint that receives webhooks from a source like Bitbucket or GitHub. Each webhook needs its own configuration.

```yaml
# Example: webhooks/bitbucket-repo-a.yaml
apiVersion: git-actions/v1
kind: Webhook
metadata:
  name: "bitbucket-repo-a"  # Unique name for this webhook configuration (string, required)
spec:
  path: "/webhook/bitbucket/repo-a" # URL path for this webhook (string, required)
  
  # Type-specific configuration section (only one of these should be present)
  bitbucket: # Bitbucket-specific configuration (object, required for Bitbucket webhooks)
    tokenFromEnv: "BITBUCKET_MYREPO_TOKEN" # Environment variable containing the webhook token (string, required)
    
    api: # Optional: Configuration for making API calls back to Bitbucket
      baseUrl: "https://bitbucket.example.com/rest/api/1.0" # Base URL of the Bitbucket API (string, required if api is present)
      project: "PROJ" # Project key/slug (string, optional, context-dependent)
      repo: "repo-a"  # Repository name/slug (string, optional, context-dependent)
      auth: # Authentication details for the API (object, required if api is present)
        type: "token" # Authentication type (string, required, e.g., "token", "basic")
        tokenFromEnv: "BITBUCKET_API_TOKEN_A" # Environment variable containing the API token (string, required for token auth)
        # usernameFromEnv: "BITBUCKET_USERNAME" # For basic auth
        # passwordFromEnv: "BITBUCKET_PASSWORD" # For basic auth
  
  # github: # GitHub-specific configuration (object, required for GitHub webhooks)
  #   secretFromEnv: "GITHUB_WEBHOOK_SECRET" # Environment variable containing the webhook secret (string, required)
  #
  #   api: # Optional: Configuration for making API calls back to GitHub
  #     # Similar structure to Bitbucket API configuration
```

## 3. Rules Configuration (`RulesConfig`)

The `RulesConfig` resource defines a set of rules that determine which actions to take when specific events occur on configured webhooks.

```yaml
# Example: rules.yaml
apiVersion: git-actions/v1
kind: Rules
metadata: # Optional metadata for the ruleset
  name: "default-rules"
spec:
  rules: # Map of rule definitions (object, required)
  # rule configurations...
```

### Rule Definition

```yaml
spec:
  rules:
    "run-tests":            # Unique rule name as the key (string, required)
      description: "..."     # Description of the rule (string, optional)
      
      # Webhooks this rule applies to (array of strings, required).
      # Each string must match the 'metadata.name' of a WebhookConfig resource.
      webhooks:
        - "bitbucket-repo-a"
        - "bitbucket-repo-b" # Example if another webhook config exists
      
      # Event types to match (array of strings, required). Use hyphens.
      event_types:
        - "push"
        - "pull_request_opened"
        - "pull_request_updated"
        # - "pull_request_merged" # Example event type
      
      branches:              # Branch filters (array of objects, optional)
        - exact: "main"      # Exact branch name
        - pattern: "feature/*" # Glob pattern
        - regex: '^release-\d+\.\d+$' # Regular expression
        # - not: "temp/*"      # Negated pattern (Note: 'not' filters might not be implemented initially)
      
      paths:                 # Path/file filters (array of objects, optional)
        - exact: "file.txt"  # Exact file path
        - pattern: "src/**/*.js" # Glob pattern
        - regex: '.*\.sql$'  # Regular expression
        # - not: "**/*.md"     # Negated pattern (Note: 'not' filters might not be implemented initially)
      
      # The 'conditions' field is removed as rule matching logic is handled by event_types, branches, paths.
      
      actions:               # Actions to execute (array of objects, required)
        # HTTP action
        - http:              # Action type is the key
            url: "http://jenkins.example.com/job/run-tests/build" # Request URL (string, required)
            method: "POST"     # HTTP method: GET, POST, PUT, DELETE, etc. (string, required)
            headers:           # HTTP headers (object, optional)
              Content-Type: "application/json"
              Authorization: "Bearer {{ env.API_TOKEN }}"
            body: |            # Request body (string, often YAML or JSON, optional)
              {
                "parameter": [
                  {"name": "BRANCH", "value": "{{ event.branch }}"},
                  {"name": "COMMIT", "value": "{{ event.commit_hash }}"}
                ]
              }
            # timeout: (Timeout configuration might be added later)
        
        # Shell command action
        - shell:             # Action type is the key
            command: "npm test" # Command to execute (string, required)
            working_dir: "./app" # Working directory (string, optional)
            environment:       # Environment variables (object, optional)
              KEY: "value"
              TOKEN: "{{ env.SECRET_TOKEN }}"
            # timeout: (Timeout configuration might be added later)
```

## Variable Substitution

Git-Actions uses the Tera templating engine for variable substitution with double curly braces:

```
{{ variable }}
```

### Available Variables

- `event` - Event data from the Git webhook
  - Properties available depend on the normalized `Event` structure (`src/webhook/event.rs`) and the specific webhook handler.
  - Common examples: `event.event_type`, `event.branch`, `event.changed_files`, `event.commit_hash` (may be nested in `event.payload`), `event.payload` (original raw payload).
  
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
# Note: Event type names are normalized by the specific WebhookTypeHandler.
# Refer to the handler implementation (e.g., BitbucketHandler) and the normalized event structure.

## Examples

### Example: GitHub Webhook and Docker Build Rule

This requires three configuration parts:

1.  **Server Config (`server.yaml`)**: Tells the server where to find the other configs.
    ```yaml
    apiVersion: git-actions/v1
    kind: Server
    metadata:
      name: "main-server"
    spec:
      port: 8080
      host: "0.0.0.0"
      logging:
        level: "DEBUG"
      config_files:
        - "webhooks/github-*.yaml" # Load GitHub webhooks
        - "rules/docker-rules.yaml" # Load Docker build rules
    ```

2.  **Webhook Config (`webhooks/github-main-repo.yaml`)**: Defines the GitHub webhook endpoint.
    ```yaml
    apiVersion: git-actions/v1
    kind: Webhook
    metadata:
      name: "github-main-repo" # Name used in rules
    spec:
      path: "/webhook/github-main"
      github: # GitHub-specific configuration
        secretFromEnv: "GITHUB_MAIN_SECRET"
        # No 'api' section needed if the handler doesn't call GitHub API
    ```

3.  **Rules Config (`rules/docker-rules.yaml`)**: Defines the rule to trigger the build.
    ```yaml
    apiVersion: git-actions/v1
    kind: Rules
    metadata:
      name: "docker-build-rules"
    spec:
      rules:
        "docker-build-on-main-push":
          description: "Build Docker image when Dockerfile changes on main branch"
          webhooks: [ "github-main-repo" ] # Matches WebhookConfig metadata.name
          event_types: [ "push" ]
          branches: [ { exact: "main" } ]
          paths:
            - { pattern: "Dockerfile" }
            - { pattern: "docker/**/*" }
          actions:
            - shell:
                command: "docker build -t myapp:{{ event.commit_hash | slice(end=7) }} ."
                # working_dir: "/path/to/repo" # Optional: If needed
    ```

### Example: Monorepo Service-Specific Testing (Bitbucket)

This requires three configuration parts:

1.  **Server Config (`server.yaml`)**: (Similar to above, pointing to the relevant webhook and rule files)
    ```yaml
    apiVersion: git-actions/v1
    kind: Server
    metadata:
      name: "git-actions-server"
    spec:
      port: 8080
      host: "0.0.0.0"
      config_files:
        - "webhooks/bitbucket-monorepo.yaml"
        - "rules/monorepo-tests.yaml"
    ```

2.  **Webhook Config (`webhooks/bitbucket-monorepo.yaml`)**:
    ```yaml
    apiVersion: git-actions/v1
    kind: Webhook
    metadata:
      name: "bitbucket-monorepo"
    spec:
      path: "/webhook/monorepo"
      bitbucket:
        tokenFromEnv: "BITBUCKET_MONOREPO_SECRET"
        # 'api' section might be needed if Bitbucket handler needs to fetch changed files via API
        api:
          baseUrl: "https://bitbucket.example.com/rest/api/1.0"
          # project/repo might not be needed if handler extracts from payload
          auth:
            type: "token"
            tokenFromEnv: "BITBUCKET_API_TOKEN"
    ```

3.  **Rules Config (`rules/monorepo-tests.yaml`)**:
    ```yaml
    apiVersion: git-actions/v1
    kind: Rules
    metadata:
      name: "monorepo-testing-rules"
    spec:
      rules:
        "frontend-tests-on-pr":
          description: "Run frontend tests when frontend code changes in a PR"
          webhooks: ["bitbucket-monorepo"]
          event_types:
            - "pull_request_opened"
            - "pull_request_updated"
          paths:
            - { pattern: "frontend/**/*" }
          actions:
            - shell:
                command: "npm run test --prefix frontend" # Run test in frontend subdir
                # working_dir: "/path/to/checkout" # Optional: Base checkout dir
    ```
