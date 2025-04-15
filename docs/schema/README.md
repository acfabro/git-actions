# Git-Actions Configuration Schema

This directory contains the configuration schema for the Git-Actions tool, which is designed to listen for Git events and execute configurable actions based on customizable rules. Git-Actions uses three main kinds of configuration files:

1. **Server** (`kind: Server`): Configures the HTTP server that listens for webhook events
2. **Webhook** (`kind: Webhook`): Defines webhook endpoints and authentication for different Git platforms
3. **Rules** (`kind: Rules`): Defines rules that respond to Git events with configurable actions

## Example Configuration Files

- [**`server.yaml`**](server.yaml): Server configuration schema for the HTTP listener
- [**`webhook.yaml`**](webhook.yaml): Example webhook configuration
- [**`rules.yaml`**](rules.yaml): Rules configuration schema

## Key Features of the Schema

1. **Kubernetes-Inspired Structure**:
   - Three main resource types: `ServerConfig`, `WebhookConfig`, and `RulesConfig`
   - Each resource has `apiVersion`, `kind`, `metadata`, and `spec` sections
   - Clear separation of concerns between server, webhook endpoints, and rules

2. **Type-Specific Webhook Handlers**:
   - Each webhook has its own configuration file
   - Support for different webhook types (Bitbucket only for now) with type-specific sections
   - Type-specific API configurations for external calls
   - Flexible authentication options

3. **Powerful Rule Matching**:
   - Event type filtering
   - Branch and path pattern matching
   - Reference to specific webhooks by name
   - Centralized rule evaluation

4. **Variable Templating (TODO)**:
   - Access to event data and environment variables
   - Template interpolation in commands, URLs, and payloads

## Configuration Files

### 1. Server Configuration

The `server.yaml` file configures the HTTP server that listens for webhook events and points to other configuration files:

```yaml
# Server configuration for Git-Actions
apiVersion: "git-actions/v1"
kind: Server
metadata:
  name: "git-actions-server"

spec:
  port: 8080
  host: "0.0.0.0"
  tls:
    enabled: false
    cert_file: "/path/to/cert.pem"
    key_file: "/path/to/key.pem"
  logging:
    level: "INFO"
    format: "json"
    file: "/var/log/git-actions.log"
  config_files:
    - "webhooks/*.yaml"  # Load all webhook configurations
    - "rules.yaml"       # Load rules configuration
```

### 2. Webhook Configuration

Each webhook has its own configuration file that defines its endpoint, type, and authentication:

```yaml
# Example Bitbucket webhook configuration
apiVersion: "git-actions/v1"
kind: Webhook
metadata:
  name: "bitbucket-repo-a"  # Unique name referenced by rules

spec:
  path: "/webhook/bitbucket/repo-a"  # URL path for this webhook
  
  # Webhook type-specific configuration
  bitbucket:
    # Environment variable containing the webhook token
    tokenFromEnv: "BITBUCKET_MYREPO_TOKEN"
    
    # API configuration for making calls to Bitbucket API
    api:
      baseUrl: "https://bitbucket.example.com/rest/api/1.0"
      project: "PROJ"
      repo: "repo-a"
      auth:
        type: "token"
        tokenFromEnv: "BITBUCKET_API_TOKEN_A"
```

### 3. Rules Configuration

The `rules.yaml` file defines rules that respond to Git events from specific webhooks:

```yaml
# Git-Actions rules configuration
apiVersion: "git-actions/v1"
kind: Rules
metadata:
  name: "default-rules"

spec:
  rules:
    "build-on-push":
      description: "Build when code is pushed to main"
      webhooks: ["bitbucket-repo-a"]  # References webhook by name
      event_types: ["push"]
      branches:
        - exact: "main"
      actions:
        - shell:
            command: "make build"
```

## Examples

The `examples/` directory contains sample configurations for different use cases:

- **server-example.yaml**: Example server configuration
- **webhook-bitbucket.yaml**: Example Bitbucket webhook configuration
- **webhook-github.yaml**: Example GitHub webhook configuration
- **rules-monorepo.yaml**: Example rules for a monorepo setup
- More examples will be added for other common scenarios

## Documentation

For detailed information about all configuration options, refer to `schema.md` which contains:

- Descriptions of all schema properties
- Data types and requirements
- Examples of common patterns

## Variable Substitution

- Variable substitution uses double curly braces: `{{ variable }}`
- Event data is accessed with the `event` prefix: `{{ event.branch }}`
- Original payload data is available at `{{ event.payload }}`
- Environment variables are accessed with the `env` prefix: `{{ env.MY_VAR }}`
