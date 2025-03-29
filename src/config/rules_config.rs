use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rules configuration for Git-Actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesConfig {
    /// API version of the configuration
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    
    /// Kind of the configuration
    pub kind: String,
    
    /// Configuration specification
    pub spec: Spec,
}

/// Configuration specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spec {
    /// Webhook configurations
    pub webhooks: Vec<Webhook>,
    
    /// Rules configurations
    pub rules: Vec<Rule>,
}

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    /// Path of the webhook
    pub path: String,
    
    /// Type of the webhook
    #[serde(rename = "type")]
    pub webhook_type: String,
    
    /// Environment variable name containing the webhook secret
    #[serde(rename = "secretFromEnv")]
    pub secret_from_env: String,
}

/// Rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// Name of the rule
    pub name: String,
    
    /// Description of the rule
    pub description: Option<String>,
    
    /// Event types to match
    #[serde(rename = "event_types")]
    pub event_types: Option<Vec<String>>,
    
    /// Branch filters to match
    pub branches: Option<Vec<BranchFilter>>,
    
    /// Path filters to match
    pub paths: Option<Vec<PathFilter>>,
    
    /// Actions to perform when the rule matches
    pub actions: Vec<Action>,
}

/// Branch filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BranchFilter {
    /// Exact branch name match
    Exact { exact: String },
    
    /// Pattern branch name match
    Pattern { pattern: String },
    
    /// Regex branch name match
    Regex { regex: String },
}

/// Path filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PathFilter {
    /// Exact path match
    Exact { exact: String },
    
    /// Pattern path match
    Pattern { pattern: String },
    
    /// Regex path match
    Regex { regex: String },
}

/// Action configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// HTTP action
    pub http: Option<HttpAction>,
    
    /// Shell action
    pub shell: Option<ShellAction>,
}

/// HTTP action configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpAction {
    /// HTTP method
    pub method: String,
    
    /// URL to call
    pub url: String,
    
    /// HTTP headers
    pub headers: Option<HashMap<String, String>>,
    
    /// HTTP body
    pub body: Option<String>,
}

/// Shell action configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellAction {
    /// Command to execute
    pub command: String,
    
    /// Working directory
    pub working_dir: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;
    
    #[test]
    fn test_deserialize() {
        let yaml = r#"
# Git-Actions rules configuration for Project A
apiVersion: git-actions/v1
kind: RulesConfig

# Configuration specification
spec:
  webhooks:
    - path: "/webhook/github"
      type: "bitbucket"
      secretFromEnv: "GITHUB_SECRET"

  rules:
    # Example rule: Run tests on PR creation
    - name: "run-tests"
      description: "Run tests when a PR is created or updated"
      # (optional) Array of event types to match. If not specified the rule will match all even types.
      event_types:
        # Event type as string
        - "pull_request-opened"
        - "pull_request-updated"
      # (optional) Array of branch filters to match. If not specified the rule will match all branches.
      branches:
        # `exact` means the branch name must match exactly
        - exact: "main"
        # `pattern` means the branch name must match the pattern
        - pattern: "feature/*"
        # `regex` means the branch name must match the regex
        - regex: "feature/*"
      # (optional) Array of path filters to match. If not specified the rule will match modifications from all paths.
      paths:
        # `exact` means the path must match exactly
        - exact: "project-a/index.js"
        # `pattern` means the path must match the wildcard pattern
        - pattern: "project-a/**/*.ts"
        # `regex` means the path must match the regex
        - regex: "project-a/[\\w]+/*.ts"
      # list of actions to perform when the rule matches
      actions:
        # call an http webhook
        - http:
            method: "POST"
            url: "http://localhost:18080/webhook/gitlab-also"
            headers:
              "Content-Type": "application/json"
            body: |
              {
                "event": "pull_request",
                "action": "opened",
                "pull_request": {
                  "number": 123,
                  "title": "Test PR",
                  "body": "This is a test PR",
                  "head": {
                    "ref": "feature/test-pr",
                    "sha": "1234567890abcdef"
                  },
                  "base": {
                    "ref": "main",
                    "sha": "fedcba0987654321"
                  }
                }
              }
        # shell command
        - shell:
            command: "cd project-a && npm run test"
            working_dir: "./projects"
"#;
        
        let config: RulesConfig = serde_yaml::from_str(yaml).unwrap();
        
        assert_eq!(config.api_version, "git-actions/v1");
        assert_eq!(config.kind, "RulesConfig");
        assert_eq!(config.spec.webhooks.len(), 1);
        assert_eq!(config.spec.rules.len(), 1);
        
        let webhook = &config.spec.webhooks[0];
        assert_eq!(webhook.path, "/webhook/github");
        assert_eq!(webhook.webhook_type, "bitbucket");
        assert_eq!(webhook.secret_from_env, "GITHUB_SECRET");
        
        let rule = &config.spec.rules[0];
        assert_eq!(rule.name, "run-tests");
        assert_eq!(rule.description, Some("Run tests when a PR is created or updated".to_string()));
        
        let event_types = rule.event_types.as_ref().unwrap();
        assert_eq!(event_types.len(), 2);
        assert_eq!(event_types[0], "pull_request-opened");
        assert_eq!(event_types[1], "pull_request-updated");
        
        let branches = rule.branches.as_ref().unwrap();
        assert_eq!(branches.len(), 3);
        
        if let BranchFilter::Exact { exact } = &branches[0] {
            assert_eq!(exact, "main");
        }
        
        if let BranchFilter::Pattern { pattern } = &branches[1] {
            assert_eq!(pattern, "feature/*");
        }
        
        let paths = rule.paths.as_ref().unwrap();
        assert_eq!(paths.len(), 3);
        
        if let PathFilter::Exact { exact } = &paths[0] {
            assert_eq!(exact, "project-a/index.js");
        }
        
        if let PathFilter::Pattern { pattern } = &paths[1] {
            assert_eq!(pattern, "project-a/**/*.ts");
        }
        
        let actions = &rule.actions;
        assert_eq!(actions.len(), 2);
        
        let http_action = actions[0].http.as_ref().unwrap();
        assert_eq!(http_action.method, "POST");
        assert_eq!(http_action.url, "http://localhost:18080/webhook/gitlab-also");
        
        let headers = http_action.headers.as_ref().unwrap();
        assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
        
        let shell_action = actions[1].shell.as_ref().unwrap();
        assert_eq!(shell_action.command, "cd project-a && npm run test");
        assert_eq!(shell_action.working_dir, "./projects");
    }
    
    #[test]
    fn test_serialize() {
        let config = RulesConfig {
            api_version: "git-actions/v1".to_string(),
            kind: "RulesConfig".to_string(),
            spec: Spec {
                webhooks: vec![
                    Webhook {
                        path: "/webhook/github".to_string(),
                        webhook_type: "bitbucket".to_string(),
                        secret_from_env: "GITHUB_SECRET".to_string(),
                    },
                ],
                rules: vec![
                    Rule {
                        name: "run-tests".to_string(),
                        description: Some("Run tests when a PR is created or updated".to_string()),
                        event_types: Some(vec![
                            "pull_request-opened".to_string(),
                            "pull_request-updated".to_string(),
                        ]),
                        branches: Some(vec![
                            BranchFilter::Exact { exact: "main".to_string() },
                            BranchFilter::Pattern { pattern: "feature/*".to_string() },
                        ]),
                        paths: Some(vec![
                            PathFilter::Exact { exact: "project-a/index.js".to_string() },
                            PathFilter::Pattern { pattern: "project-a/**/*.ts".to_string() },
                        ]),
                        actions: vec![
                            Action {
                                http: Some(HttpAction {
                                    method: "POST".to_string(),
                                    url: "http://localhost:18080/webhook/gitlab-also".to_string(),
                                    headers: Some({
                                        let mut map = HashMap::new();
                                        map.insert("Content-Type".to_string(), "application/json".to_string());
                                        map
                                    }),
                                    body: Some("{}".to_string()),
                                }),
                                shell: None,
                            },
                            Action {
                                http: None,
                                shell: Some(ShellAction {
                                    command: "cd project-a && npm run test".to_string(),
                                    working_dir: "./projects".to_string(),
                                }),
                            },
                        ],
                    },
                ],
            },
        };
        
        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: RulesConfig = serde_yaml::from_str(&yaml).unwrap();
        
        assert_eq!(deserialized.api_version, config.api_version);
        assert_eq!(deserialized.kind, config.kind);
        assert_eq!(deserialized.spec.webhooks.len(), config.spec.webhooks.len());
        assert_eq!(deserialized.spec.rules.len(), config.spec.rules.len());
    }
}