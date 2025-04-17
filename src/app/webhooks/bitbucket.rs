use super::rule_evaluator;
use crate::app::webhooks::types::{Branch, Event, EventType, Path, WebhookTypeHandler};
use crate::config::Rule;
use crate::Error;
use async_trait::async_trait;
use bitbucket_server_rs::ApiRequest;
use jsonpath_rust::JsonPath;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use tracing::debug;
use crate::config::rules_config::Action;

pub struct Bitbucket<'a> {
    pub config: crate::config::webhook_config::Bitbucket,
    pub rules: HashMap<String, &'a Rule>,
    pub payload: Value,
}

#[async_trait]
impl WebhookTypeHandler for Bitbucket<'_> {
    async fn extract_event_type(&self) -> Result<EventType, Error> {
        let event_type = self
            .payload
            .query("$.eventKey")
            .map_err(|e| Error::WebhookPayloadError(format!("jsonpath error: {}", e)))?;

        match event_type.first() {
            None => Err(Error::WebhookPayloadError(
                "Missing event type from payload".to_string(),
            )),
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

    async fn extract_branch(&self) -> Result<Branch, Error> {
        let branch = self
            .payload
            .query("$.pullRequest.fromRef.displayId")
            .map_err(|e| Error::WebhookPayloadError(format!("jsonpath error: {}", e)))?;

        match branch.get(0) {
            None => Err(Error::WebhookPayloadError(
                "Missing branch from payload".to_string(),
            )),
            Some(b) => {
                // Extract the string value without quotes
                if let Some(s) = b.as_str() {
                    Ok(s.to_string())
                } else {
                    Ok(b.to_string().trim_matches('"').to_string())
                }
            }
        }
    }

    async fn extract_changed_files(&self) -> Result<Vec<Path>, Error> {
        // get the pull request id from the payload
        let pr_id = self
            .payload
            .query("$.pullRequest.id")
            .map_err(|e| Error::WebhookPayloadError(format!("jsonpath error: {}", e)))?;

        // as u32
        let pr_id: u32 = match pr_id.get(0) {
            None => {
                return Err(Error::WebhookPayloadError(
                    "Missing pull request id from payload".to_string(),
                ))
            }
            Some(id) => {
                if let Some(s) = id.as_u64() {
                    s as u32
                } else {
                    return Err(Error::WebhookPayloadError(
                        "Invalid pull request id from payload".to_string(),
                    ));
                }
            }
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
            .pull_request_changes_get(
                &self.config.api.project,
                &self.config.api.repo,
                &pr_id.to_string(),
            )
            .build()
            .map_err(|_| {
                Error::WebhookPayloadError("Error building bitbucket api request".to_string())
            })?
            .send()
            .await
            .map_err(|e| {
                Error::WebhookPayloadError(format!(
                    "Could not get changed files from bitbucket: {}",
                    e
                ))
            })?;

        // get the pr changes
        let pr_changes = match response {
            Some(changes) => changes,
            None => {
                return Err(Error::WebhookPayloadError(
                    "Empty response from bitbucket api".to_string(),
                ));
            }
        };

        // get the changed files from the pr changes
        let changed_files = match pr_changes.values {
            None => {
                return Err(Error::WebhookPayloadError(
                    "No changed files found".to_string(),
                ))
            }
            Some(c) => c
                .into_iter()
                .filter_map(|change| Some(change.path.to_string))
                .collect::<Vec<Path>>(),
        };

        Ok(changed_files)
    }

    async fn run(&self) -> Result<Vec<&Action>, Error> {
        // extract needed data from the payload
        let branch = self.extract_branch().await?;
        let changed_files = self.extract_changed_files().await?;
        let event_type = self.extract_event_type().await?;

        //
        let event = Event {
            event_type,
            branch,
            changed_files,
        };

        let mut actions: Vec<&Action> = Vec::new();
        // This is where you would handle the webhook payload and apply rules
        for (rule_name, rule) in &self.rules {
            // call matches_rule to check if the rule applies
            let result = rule_evaluator::check(&event, rule);
            if result {
                debug!("Rule {} matched. Adding actions", rule_name);
                for action in &rule.actions {
                    actions.push(action);
                }
            }
        }

        Ok(actions)
    }
}

/// Convert bitbucket event type string to EventType enum
fn event_type_from_str(str: &str) -> Result<EventType, Error> {
    match str {
        "pr:created" => Ok(EventType::PRCreated),
        "pr:modified" => Ok(EventType::PRModified),
        "pr:merged" => Ok(EventType::PRMerged),
        _ => Err(Error::WebhookPayloadError(format!(
            "Unknown event type {}",
            str
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::webhook_config::{BitbucketApi, BitbucketAuth};
    use serde_json::json;

    fn create_test_bitbucket(payload: Value) -> Bitbucket<'static> {
        Bitbucket {
            config: crate::config::webhook_config::Bitbucket {
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
            "eventKey": "pr:created"
        });

        let bitbucket = create_test_bitbucket(payload);

        let event_type = bitbucket.extract_event_type().await;
        assert_eq!(event_type.unwrap(), EventType::PRCreated);
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

}
