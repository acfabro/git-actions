use super::types::{Branch, Event, EventType, Path, WebhookTypeHandler};
use crate::app::config::{webhook, Action, Rule};
use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use bitbucket_server_rs::ApiRequest;
use serde_json::Value;
use std::collections::HashMap;
use std::env;

pub struct Bitbucket<'a> {
    pub config: webhook::Bitbucket,
    pub rules: HashMap<String, &'a Rule>,
    pub payload: Value,
}

impl Bitbucket<'_> {
    pub async fn extract_event_type(&self) -> Result<EventType> {
        match &self.payload["eventKey"] {
            Value::String(s) => event_type_from_str(s),
            _ => bail!("Invalid event type"),
        }
    }

    pub async fn extract_branch(&self, event_type: &EventType) -> Result<Branch> {
        let branch_ref = match event_type {
            EventType::Merged => &self.payload["pullRequest"]["toRef"]["displayId"],
            _ => &self.payload["pullRequest"]["fromRef"]["displayId"],
        };

        branch_ref.as_str().map_or_else(
            || Err(anyhow!("Missing branch from payload")),
            |s| Ok(s.to_string()),
        )
    }

    pub async fn extract_changed_files(&self) -> Result<Vec<Path>> {
        // bitbucket api config vars
        let bitbucket_api = &self.config.api.base_url;
        let bitbucket_api_token =
            &env::var(&self.config.api.auth.token_from_env).unwrap_or_default();

        // create bitbucket api client
        let client = bitbucket_server_rs::new(bitbucket_api, bitbucket_api_token);

        // call the api
        let response = client
            .api()
            .pull_request_changes_get(
                &self.config.api.project,
                &self.config.api.repo,
                &self.payload["pullRequest"]["id"].to_string(),
            )
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
                .iter()
                .map(|change| change.path.to_string.clone())
                .collect::<Vec<Path>>(),
        };

        Ok(changed_files)
    }
}

#[async_trait]
impl WebhookTypeHandler for Bitbucket<'_> {
    async fn extract_event(&self) -> Result<Event> {
        let event_type = self.extract_event_type().await?;
        let branch = self.extract_branch(&event_type).await?;
        let changed_files = self.extract_changed_files().await?;

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
pub fn event_type_from_str(str: &str) -> Result<EventType> {
    str.try_into()
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
        assert_eq!(event_type.unwrap(), EventType::Modified);
    }

    #[tokio::test]
    async fn test_extract_event_type_created() {
        let payload = json!({
            "eventKey": "pr:opened"
        });

        let bitbucket = create_test_bitbucket(payload);

        let event_type = bitbucket.extract_event_type().await;
        assert_eq!(event_type.unwrap(), EventType::Opened);
    }

    #[tokio::test]
    async fn test_extract_event_type_merged() {
        let payload = json!({
            "eventKey": "pr:merged"
        });

        let bitbucket = create_test_bitbucket(payload);

        let event_type = bitbucket.extract_event_type().await;
        assert_eq!(event_type.unwrap(), EventType::Merged);
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
                },
                "toRef": {
                    "displayId": "main"
                }
            }
        });

        let bitbucket = create_test_bitbucket(payload);

        // Test non-merge event uses fromRef
        let branch = bitbucket.extract_branch(&EventType::Opened).await;
        assert_eq!(branch.unwrap(), "feature/test-branch");

        // Test merge event uses toRef
        let branch = bitbucket.extract_branch(&EventType::Merged).await;
        assert_eq!(branch.unwrap(), "main");
    }

    #[tokio::test]
    async fn test_extract_branch_missing() {
        let payload = json!({
            "pullRequest": {
                "someOtherKey": "value"
            }
        });

        let bitbucket = create_test_bitbucket(payload);

        let branch = bitbucket.extract_branch(&EventType::Opened).await;
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
                },
                "toRef": {
                    "displayId": "main",
                    "id": "refs/heads/main",
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

        // Test non-merge event
        let branch = bitbucket.extract_branch(&EventType::Modified).await;
        assert_eq!(branch.unwrap(), "feature/test-push-branch-no-pr");

        // Test merge event
        let branch = bitbucket.extract_branch(&EventType::Merged).await;
        assert_eq!(branch.unwrap(), "main");
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

        let branch_result = bitbucket.extract_branch(&EventType::Opened).await;
        assert!(branch_result.is_err());
    }
}
