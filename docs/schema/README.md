# Git-Actions Configuration Schema

This directory contains the configuration schema for the Git-Actions tool, which is designed to listen for Git events and execute configurable actions based on customizable rules.

## Directory Structure

- **`server.yaml`**: Server configuration schema for the HTTP listener
- **`git-actions.yaml`**: Main configuration schema for webhooks and rules
- **`schema.md`**: Detailed schema documentation with types, descriptions, and examples
- **`examples/`**: Example configurations for specific use cases

## Key Features of the Schema

1. **Separation of Concerns**:
   - Server configuration is separate from webhook and rule definitions
   - Operational vs. business logic separation

2. **Flexible Webhook Configuration**:
   - Multiple webhooks with different paths and authentication
   - Support for GitHub, GitLab, and Bitbucket

3. **Powerful Rule Matching**:
   - Event type filtering
   - Branch and path pattern matching
   - Conditional expressions

4. **Variable Templating**:
   - Uses Tera templating engine (similar to Jinja2/Django)
   - Access to event data and environment variables
   - Template interpolation in commands, URLs, and payloads

## Configuration Files

### Server Configuration

The `server.yaml` file configures the HTTP server that listens for webhook events. Example:

```yaml
# Server configuration for Git-Actions
apiVersion: "git-actions/v1"
kind: "ServerConfig"

# Server specification
spec:
  port: 8080
  host: "0.0.0.0"
  tls:
    enabled: true
    cert_file: "/path/to/cert.pem"
    key_file: "/path/to/key.pem"
```

### Webhook and Rules Configuration

The `git-actions.yaml` file defines webhook endpoints and the rules that respond to Git events. Example:

```yaml
# Git-Actions rules configuration
apiVersion: "git-actions/v1"
kind: "RulesConfig"

# Configuration specification
spec:
  webhooks:
    github_main:
      path: "/webhook/github"
      type: "github"
      secretFromEnv: "GITHUB_SECRET"

  rules:
    - name: "build-on-push"
      event_types: ["push"]
      branches:
        - exact: "main"
      actions:
        - type: "shell"
          command: "make build"
```

## Examples

The `examples/` directory contains sample configurations for different use cases:

- **monorepo-integration-tests.yaml**: Configuration for running integration tests in a monorepo
- More examples will be added for other common scenarios

## Documentation

For detailed information about all configuration options, refer to `schema.md` which contains:

- Descriptions of all schema properties
- Data types and requirements
- Examples of common patterns

## Implementation Notes

- Text templating is provided by the [Tera](https://keats.github.io/tera/) crate
- Variable substitution uses double curly braces: `{{ variable }}`
- Environment variables are accessed with the `env` prefix: `{{ env.MY_VAR }}`
