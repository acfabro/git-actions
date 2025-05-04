use super::types::{Branch, Event, EventType, Path, WebhookTypeHandler};
use crate::app::config::{webhook, Action, Rule};
use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use bitbucket_server_rs::ApiRequest;
use jsonpath_rust::JsonPath;
use serde_json::Value;
use std::collections::HashMap;
use std::env;

pub struct Bitbucket<'a> {
    pub config: webhook::Bitbucket,
    pub rules: HashMap<String, &'a Rule>,
    pub payload: Value,
}

impl Bitbucket<'_> {
    async fn extract_event_type(&self) -> Result<EventType> {
        let jsonpath = "$.eventKey";
        let event_type = self
            .payload
            .query(jsonpath)
            .with_context(|| format!("jsonpath error: {}", jsonpath))?;

        match event_type.first() {
            None => Err(anyhow!("Missing event type from payload".to_string())),
            Some(e) => {
                // Extract the string value without quotes
                if let Some(s) = e.as_str() {
                    Ok(event_type_from_str(s)?)
                } else {
                    Ok(event_type_from_str(e.to_string().trim_matches('"'))?)
                }
            }
        }
    }

    async fn extract_branch(&self) -> Result<Branch> {
        let jsonpath = "$.pullRequest.fromRef.displayId";
        let branch = self
            .payload
            .query(jsonpath)
            .with_context(|| format!("jsonpath compile error: {}", jsonpath))?;

        match branch.get(0) {
            None => Err(anyhow!("Missing branch from payload")),
            Some(b) => {
                // getting an issue where a string is returned with quotes
                if let Some(s) = b.as_str() {
                    Ok(s.to_string())
                } else {
                    Ok(b.to_string().trim_matches('"').to_string())
                }
            }
        }
    }

    async fn extract_changed_files(&self) -> Result<Vec<Path>> {
        // get the pull request id from the payload
        let jsonpath = "$.pullRequest.id";
        let pr_id = self
            .payload
            .query(jsonpath)
            .with_context(|| format!("jsonpath error: {}", jsonpath))?;

        //
        let pr_id: &str = &match pr_id.get(0) {
            None => bail!("Missing pull request id from payload"),
            Some(id) => match id {
                Value::Number(n) => n.to_string(),
                Value::String(s) => s.to_owned(),
                _ => bail!("Invalid pull request id from payload"),
            },
        };

        // bitbucket api config vars
        let bitbucket_api = &self.config.api.base_url;
        let bitbucket_api_token =
            &env::var(&self.config.api.auth.token_from_env).unwrap_or_default();

        // create bitbucket api client
        let client = bitbucket_server_rs::new(&bitbucket_api, &bitbucket_api_token);

        // call the api
        let response = client
            .api()
            .pull_request_changes_get(&self.config.api.project, &self.config.api.repo, pr_id)
            .build()
            .with_context(|| "Error building bitbucket api request".to_string())?
            .send()
            .await
            .with_context(|| "Could not get changed files from bitbucket".to_string())?;

        // get the pr changes
        let pr_changes = match response {
            None => bail!("Empty response from bitbucket api".to_string()),
            Some(changes) => changes,
        };

        // get the changed files from the pr changes
        let changed_files = match pr_changes.values {
            None => bail!("No changed files found".to_string()),
            Some(c) => c
                .into_iter()
                .filter_map(|change| Some(change.path.to_string))
                .collect::<Vec<Path>>(),
        };

        Ok(changed_files)
    }
}

#[async_trait]
impl WebhookTypeHandler for Bitbucket<'_> {
    async fn extract_event(&self) -> Result<Event> {
        let branch = self.extract_branch().await?;
        let changed_files = self.extract_changed_files().await?;
        let event_type = self.extract_event_type().await?;

        //
        Ok(Event {
            event_type,
            branch,
            changed_files,
        })
    }

    async fn run(&self) -> Result<Vec<&Action>> {
        // platform-neutral event
        let event = self.extract_event().await?;

        //
        let actions = Self::evaluate_rules(&event, &self.rules);

        Ok(actions)
    }
}

/// Convert bitbucket event type string to EventType enum
fn event_type_from_str(str: &str) -> Result<EventType> {
    match str {
        "pr:opened" => Ok(EventType::PROpened),
        "pr:modified" => Ok(EventType::PRModified),
        "pr:merged" => Ok(EventType::PRMerged),
        // TODO other event types
        _ => Err(anyhow!("Unknown event type {}", str)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::config::webhook::{BitbucketApi, BitbucketAuth};
    use serde_json::json;

    fn create_test_bitbucket(payload: Value) -> Bitbucket<'static> {
        Bitbucket {
            config: webhook::Bitbucket {
                token_from_env: None,
                api: BitbucketApi {
                    base_url: "".to_string(),
                    project: "".to_string(),
                    repo: "".to_string(),
                    auth: BitbucketAuth {
                        auth_type: "".to_string(),
                        token_from_env: "".to_string(),
                    },
                },
            },
            rules: HashMap::new(),
            payload,
        }
    }

    #[tokio::test]
    async fn test_can_get_event_type() {
        let payload = json!({
            "eventKey": "pr:modified"
        });

        let bitbucket = create_test_bitbucket(payload);

        let event_type = bitbucket.extract_event_type().await;
        assert_eq!(event_type.unwrap(), EventType::PRModified);
    }

    #[tokio::test]
    async fn test_extract_event_type_created() {
        let payload = json!({
            "eventKey": "pr:opened"
        });

        let bitbucket = create_test_bitbucket(payload);

        let event_type = bitbucket.extract_event_type().await;
        assert_eq!(event_type.unwrap(), EventType::PROpened);
    }

    #[tokio::test]
    async fn test_extract_event_type_merged() {
        let payload = json!({
            "eventKey": "pr:merged"
        });

        let bitbucket = create_test_bitbucket(payload);

        let event_type = bitbucket.extract_event_type().await;
        assert_eq!(event_type.unwrap(), EventType::PRMerged);
    }

    #[tokio::test]
    async fn test_extract_event_type_invalid() {
        let payload = json!({
            "eventKey": "pr:unknown"
        });

        let bitbucket = create_test_bitbucket(payload);

        let event_type = bitbucket.extract_event_type().await;
        assert!(event_type.is_err());
    }

    #[tokio::test]
    async fn test_extract_event_type_missing() {
        let payload = json!({
            "someOtherKey": "value"
        });

        let bitbucket = create_test_bitbucket(payload);

        let event_type = bitbucket.extract_event_type().await;
        assert!(event_type.is_err());
    }

    #[tokio::test]
    async fn test_extract_branch() {
        let payload = json!({
            "pullRequest": {
                "fromRef": {
                    "displayId": "feature/test-branch"
                }
            }
        });

        let bitbucket = create_test_bitbucket(payload);

        let branch = bitbucket.extract_branch().await;
        assert_eq!(branch.unwrap(), "feature/test-branch");
    }

    #[tokio::test]
    async fn test_extract_branch_missing() {
        let payload = json!({
            "pullRequest": {
                "someOtherKey": "value"
            }
        });

        let bitbucket = create_test_bitbucket(payload);

        let branch = bitbucket.extract_branch().await;
        assert!(branch.is_err());
    }

    #[tokio::test]
    async fn test_extract_branch_nested_structure() {
        // Test with the deep nested structure from the real payload
        let payload = json!({
            "pullRequest": {
                "fromRef": {
                    "displayId": "feature/test-push-branch-no-pr",
                    "id": "refs/heads/feature/test-push-branch-no-pr",
                    "repository": {
                        "slug": "sre-infra",
                        "project": {
                            "key": "GOLF"
                        }
                    }
                }
            }
        });

        let bitbucket = create_test_bitbucket(payload);

        let branch = bitbucket.extract_branch().await;
        assert_eq!(branch.unwrap(), "feature/test-push-branch-no-pr");
    }

    #[tokio::test]
    async fn test_invalid_payload() {
        // A completely invalid payload that doesn't match Bitbucket structure
        let payload = json!({
            "type": "not-bitbucket",
            "data": {
                "random": "values"
            }
        });

        let bitbucket = create_test_bitbucket(payload);

        // The event extraction should fail
        let event_result = bitbucket.extract_event().await;
        assert!(event_result.is_err());

        // Each individual extraction method should also fail
        let event_type_result = bitbucket.extract_event_type().await;
        assert!(event_type_result.is_err());

        let branch_result = bitbucket.extract_branch().await;
        assert!(branch_result.is_err());
    }

    // Integration tests using the custom HTTP client approach
    #[tokio::test]
    async fn integration_extract_changed_files_success() {
        use serde_json::json;
        use std::collections::HashMap;
        use wiremock::matchers::{method, path_regex};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        // Import config types
        use crate::app::config::webhook::{
            Bitbucket as BitbucketConfig, BitbucketApi, BitbucketAuth,
        };

        // Start a mock server
        let mock_server = MockServer::start().await;

        // Prepare mock Bitbucket API response
        let api_response = json!({
            "fromHash": "deadbeef",
            "toHash": "beefdead",
            "values": [
                {
                    "contentId": "abc123",
                    "type": "MODIFY",
                    "path": { "toString": "src/main.rs", "fromHash": "deadbeef" },
                    "fromHash": "deadbeef",
                    "toHash": "beefdead"
                },
                {
                    "contentId": "def456",
                    "type": "ADD",
                    "path": { "toString": "README.md", "fromHash": "cafebabe" },
                    "fromHash": "cafebabe",
                    "toHash": "babecafe"
                }
            ]
        });

        // Mock the Bitbucket API endpoint for PR changes
        Mock::given(method("GET"))
            .and(path_regex(".*/pull-requests/123/changes$"))
            .respond_with(ResponseTemplate::new(200).set_body_json(api_response))
            .mount(&mock_server)
            .await;

        // Prepare payload with pull request ID
        let payload = json!({
            "pullRequest": {
                "id": 123
            }
        });

        // Build config to use the mock server
        let config = BitbucketConfig {
            token_from_env: Some("".to_string()),
            api: BitbucketApi {
                base_url: mock_server.uri(),
                project: "PROJ".to_string(),
                repo: "REPO".to_string(),
                auth: BitbucketAuth {
                    auth_type: "token".to_string(),
                    token_from_env: "".to_string(),
                },
            },
        };

        // Create Bitbucket instance with config and payload
        let bitbucket = Bitbucket {
            config,
            rules: HashMap::new(),
            payload,
        };

        // Call extract_changed_files and assert result
        let files = bitbucket.extract_changed_files().await.unwrap();
        assert_eq!(
            files,
            vec!["src/main.rs".to_string(), "README.md".to_string()]
        );
    }

    #[tokio::test]
    async fn integration_rule_evaluation_with_changed_files() {
        use crate::app::config::webhook::{Bitbucket as BitbucketConfig, BitbucketApi, BitbucketAuth};
        use crate::app::config::Rule;
        use serde_json::json;
        use std::collections::HashMap;
        use wiremock::matchers::{method, path_regex};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        // Start a mock server
        let mock_server = MockServer::start().await;

        // Prepare mock Bitbucket API response
        let api_response = json!({
        "fromHash": "deadbeef",
        "toHash": "beefdead",
        "values": [
            {
                "contentId": "abc123",
                "type": "MODIFY",
                "path": { "toString": "src/main.rs", "fromHash": "deadbeef" },
                "fromHash": "deadbeef",
                "toHash": "beefdead"
            },
            {
                "contentId": "def456",
                "type": "ADD",
                "path": { "toString": "README.md", "fromHash": "cafebabe" },
                "fromHash": "cafebabe",
                "toHash": "babecafe"
            }
        ]
    });

        // Mock the Bitbucket API endpoint for PR changes
        Mock::given(method("GET"))
            .and(path_regex(".*/pull-requests/123/changes$"))
            .respond_with(ResponseTemplate::new(200).set_body_json(api_response))
            .mount(&mock_server)
            .await;

        // Prepare payload with pull request ID
        let payload = json!({
        "pullRequest": {
            "id": 123
        }
    });

        // Build config to use the mock server
        let config = BitbucketConfig {
            token_from_env: Some("".to_string()),
            api: BitbucketApi {
                base_url: mock_server.uri(),
                project: "PROJ".to_string(),
                repo: "REPO".to_string(),
                auth: BitbucketAuth {
                    auth_type: "token".to_string(),
                    token_from_env: "".to_string(),
                },
            },
        };

        // Define a rule that matches on changed file "src/main.rs"
        let mut rules = HashMap::new();
        use crate::app::config::rules::PathFilter;

        let rule = Rule {
            description: Some("main_rs_change".to_string()),
            webhooks: vec![],
            event_types: None,
            branches: None,
            paths: Some(vec![PathFilter::Exact {
                exact: "src/main.rs".to_string(),
            }]),
            actions: vec![],
        };
        rules.insert("main_rs_change".to_string(), &rule);

        // Create Bitbucket instance with config, rules, and payload
        let bitbucket = Bitbucket {
            config,
            rules: rules.clone(),
            payload,
        };

        // Extract changed files
        let changed_files = bitbucket.extract_changed_files().await.unwrap();

        // Evaluate rules: check if any rule's paths match any changed file
        let matched = rules.values().any(|rule| {
            if let Some(ref paths) = rule.paths {
                paths.iter().any(|filter| match filter {
                    PathFilter::Exact { exact } => changed_files.contains(exact),
                    _ => false,
                })
            } else {
                false
            }
        });

        assert!(matched, "Rule should match changed file src/main.rs");
    }
}
