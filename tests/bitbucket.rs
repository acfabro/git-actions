use git_actions::app::config::rules::{HttpAction, PathFilter};
use git_actions::app::config::webhook::{
    Bitbucket as BitbucketConfig, BitbucketApi, BitbucketAuth,
};
use git_actions::app::config::{Action, Rule};
use git_actions::app::webhooks::bitbucket::Bitbucket;
use git_actions::app::webhooks::types::WebhookTypeHandler;
use serde_json::{json, Value};
use std::collections::HashMap;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

// Helper functions
fn create_pr_opened_payload() -> Value {
    json!({
        "eventKey": "pr:opened",
        "date": "2025-05-03T18:28:59+0900",
        "pullRequest": {
            "id": 1373,
            "fromRef": {
                "displayId": "feature/test-branch",
                "id": "refs/heads/feature/test-branch",
                "repository": {
                    "slug": "sre-infra",
                    "project": {
                        "key": "GOLF"
                    }
                }
            },
            "toRef": {
                "displayId": "main",
                "id": "refs/heads/main"
            },
            "title": "Test PR",
            "state": "OPEN"
        }
    })
}

fn create_bitbucket_api_mock_response(changed_files: Vec<&str>) -> Value {
    let mut values = Vec::new();

    for (index, file) in changed_files.iter().enumerate() {
        values.push(json!({
            "contentId": format!("id{}", index),
            "type": "MODIFY",
            "path": {
                "toString": file,
                "fromHash": "deadbeef"
            },
            "fromHash": "deadbeef",
            "toHash": "beefdead"
        }));
    }

    json!({
        "fromHash": "deadbeef",
        "toHash": "beefdead",
        "values": values
    })
}

// Integration tests
#[tokio::test]
async fn integration_extract_changed_files_success() {
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

// Structure for table-driven tests
#[derive(Clone)]
struct WebhookHandlerTestCase {
    name: &'static str,
    mock_changed_files: Vec<&'static str>,
    rules_data: Vec<(&'static str, Rule)>, // (rule_name, rule_definition)
    payload: Value,
    expected_actions_count: usize,
    expected_action_urls: Vec<&'static str>,
}

// Helper function to run a single test case
async fn run_webhook_handler_test_case(case: WebhookHandlerTestCase) {
    println!("Running test case: {}", case.name);

    // Start a mock server for each case to ensure isolation
    let mock_server = MockServer::start().await;

    // Create mock API response
    let api_response = create_bitbucket_api_mock_response(case.mock_changed_files.clone());

    // Mock the Bitbucket API endpoint for PR changes (using PR ID 1373 from fixture)
    Mock::given(method("GET"))
        .and(path_regex(".*/pull-requests/1373/changes$"))
        .respond_with(ResponseTemplate::new(200).set_body_json(api_response))
        .mount(&mock_server)
        .await;

    // Prepare rules HashMap for this case
    let rules_map: HashMap<String, &Rule> = case
        .rules_data
        .iter()
        .map(|(name, rule)| (name.to_string(), rule))
        .collect();

    // Configure Bitbucket instance
    let config = BitbucketConfig {
        token_from_env: Some("".to_string()), // Assuming token not needed for mock
        api: BitbucketApi {
            base_url: mock_server.uri(),
            project: "GOLF".to_string(),   // From fixture payload
            repo: "sre-infra".to_string(), // From fixture payload
            auth: BitbucketAuth {
                auth_type: "token".to_string(),
                token_from_env: "".to_string(),
            },
        },
    };

    // Create Bitbucket instance
    let bitbucket = Bitbucket {
        config,
        rules: rules_map,
        payload: case.payload.clone(), // Clone payload for this iteration
    };

    // Test the run method
    let actions_result = bitbucket.run().await;
    assert!(
        actions_result.is_ok(),
        "Test case '{}' failed: {:?}",
        case.name,
        actions_result.err()
    );
    let actions = actions_result.unwrap();

    // Verify action count
    assert_eq!(
        actions.len(),
        case.expected_actions_count,
        "Test case '{}': Expected {} actions, got {}",
        case.name,
        case.expected_actions_count,
        actions.len()
    );

    // Verify action URLs if expected URLs are provided
    if !case.expected_action_urls.is_empty() {
        let actual_urls: Vec<&str> = actions
            .iter()
            .filter_map(|a| a.http.as_ref().map(|h| h.url.as_str()))
            .collect();

        // Use HashSet for order-independent comparison
        use std::collections::HashSet;
        let expected_urls_set: HashSet<&str> = case.expected_action_urls.iter().cloned().collect();
        let actual_urls_set: HashSet<&str> = actual_urls.iter().cloned().collect();

        assert_eq!(
            actual_urls_set, expected_urls_set,
            "Test case '{}': URL mismatch. Expected {:?}, got {:?}",
            case.name, expected_urls_set, actual_urls_set
        );
    }
}

// Individual test functions calling the helper

#[tokio::test]
async fn test_webhook_handler_rules_match() {
    let case = WebhookHandlerTestCase {
        name: "rules_match",
        mock_changed_files: vec!["src/main.rs", "README.md"],
        rules_data: vec![(
            "matching-rule",
            Rule {
                description: Some("PR opened with main.rs changes".to_string()),
                webhooks: vec!["test-webhook".to_string()],
                event_types: Some(vec!["pr_created".to_string()]),
                branches: None,
                paths: Some(vec![PathFilter::Exact {
                    exact: "src/main.rs".to_string(),
                }]),
                actions: vec![Action {
                    http: Some(HttpAction {
                        method: "POST".to_string(),
                        url: "https://example.com/webhook".to_string(),
                        headers: None,
                        body: None,
                    }),
                    shell: None,
                }],
            },
        )],
        payload: create_pr_opened_payload(),
        expected_actions_count: 1,
        expected_action_urls: vec!["https://example.com/webhook"],
    };
    run_webhook_handler_test_case(case).await;
}

#[tokio::test]
async fn test_webhook_handler_no_rules_match() {
    let case = WebhookHandlerTestCase {
        name: "no_rules_match",
        mock_changed_files: vec!["docs/README.md"],
        rules_data: vec![(
            "non-matching-rule",
            Rule {
                description: Some("Rule that won't match".to_string()),
                webhooks: vec!["test-webhook".to_string()],
                event_types: Some(vec!["pr_created".to_string()]),
                branches: None,
                paths: Some(vec![PathFilter::Exact {
                    exact: "src/main.rs".to_string(), // Expects main.rs, gets docs/README.md
                }]),
                actions: vec![Action {
                    http: Some(HttpAction {
                        method: "POST".to_string(),
                        url: "https://example.com/webhook".to_string(),
                        headers: None,
                        body: None,
                    }),
                    shell: None,
                }],
            },
        )],
        payload: create_pr_opened_payload(),
        expected_actions_count: 0,
        expected_action_urls: vec![],
    };
    run_webhook_handler_test_case(case).await;
}

#[tokio::test]
async fn test_webhook_handler_event_type_filtering() {
    let case = WebhookHandlerTestCase {
        name: "event_type_filtering",
        mock_changed_files: vec!["src/main.rs"],
        rules_data: vec![(
            "wrong-event-rule",
            Rule {
                description: Some("Rule for PR modified events".to_string()),
                webhooks: vec!["test-webhook".to_string()],
                event_types: Some(vec!["pr_modified".to_string()]), // Expects modified, gets opened
                branches: None,
                paths: Some(vec![PathFilter::Exact {
                    exact: "src/main.rs".to_string(),
                }]),
                actions: vec![Action {
                    http: Some(HttpAction {
                        method: "POST".to_string(),
                        url: "https://example.com/webhook".to_string(),
                        headers: None,
                        body: None,
                    }),
                    shell: None,
                }],
            },
        )],
        payload: create_pr_opened_payload(),
        expected_actions_count: 0,
        expected_action_urls: vec![],
    };
    run_webhook_handler_test_case(case).await;
}

#[tokio::test]
async fn test_webhook_handler_multiple_rules() {
    let case = WebhookHandlerTestCase {
        name: "multiple_rules",
        mock_changed_files: vec!["src/main.rs", "README.md"],
        rules_data: vec![
            (
                "matching-rule-1",
                Rule {
                    description: Some("PR opened with main.rs changes".to_string()),
                    webhooks: vec!["test-webhook".to_string()],
                    event_types: Some(vec!["pr_created".to_string()]),
                    branches: None,
                    paths: Some(vec![PathFilter::Exact {
                        exact: "src/main.rs".to_string(),
                    }]),
                    actions: vec![Action {
                        http: Some(HttpAction {
                            method: "POST".to_string(),
                            url: "https://example.com/webhook1".to_string(),
                            headers: None,
                            body: None,
                        }),
                        shell: None,
                    }],
                },
            ),
            (
                "matching-rule-2",
                Rule {
                    description: Some("PR opened with README.md changes".to_string()),
                    webhooks: vec!["test-webhook".to_string()],
                    event_types: Some(vec!["pr_created".to_string()]),
                    branches: None,
                    paths: Some(vec![PathFilter::Exact {
                        exact: "README.md".to_string(),
                    }]),
                    actions: vec![Action {
                        http: Some(HttpAction {
                            method: "POST".to_string(),
                            url: "https://example.com/webhook2".to_string(),
                            headers: None,
                            body: None,
                        }),
                        shell: None,
                    }],
                },
            ),
            (
                "non-matching-rule", // Doesn't match (wrong event type)
                Rule {
                    description: Some("Rule for PR modified events".to_string()),
                    webhooks: vec!["test-webhook".to_string()],
                    event_types: Some(vec!["pr_modified".to_string()]),
                    branches: None,
                    paths: Some(vec![PathFilter::Exact {
                        exact: "src/main.rs".to_string(),
                    }]),
                    actions: vec![Action {
                        http: Some(HttpAction {
                            method: "POST".to_string(),
                            url: "https://example.com/webhook3".to_string(),
                            headers: None,
                            body: None,
                        }),
                        shell: None,
                    }],
                },
            ),
        ],
        payload: create_pr_opened_payload(),
        expected_actions_count: 2,
        expected_action_urls: vec![
            "https://example.com/webhook1",
            "https://example.com/webhook2",
        ],
    };
    run_webhook_handler_test_case(case).await;
}

#[tokio::test]
async fn test_webhook_handler_rule_without_event_type() {
    let case = WebhookHandlerTestCase {
        name: "match_any_event_type",
        mock_changed_files: vec!["src/main.rs"],
        rules_data: vec![(
            "any-event-rule",
            Rule {
                description: Some("Rule matching any event type".to_string()),
                webhooks: vec!["test-webhook".to_string()],
                event_types: None, // Should match pr:opened
                branches: None,
                paths: Some(vec![PathFilter::Exact {
                    exact: "src/main.rs".to_string(),
                }]),
                actions: vec![Action {
                    http: Some(HttpAction {
                        method: "POST".to_string(),
                        url: "https://example.com/any_event".to_string(),
                        headers: None,
                        body: None,
                    }),
                    shell: None,
                }],
            },
        )],
        payload: create_pr_opened_payload(),
        expected_actions_count: 1,
        expected_action_urls: vec!["https://example.com/any_event"],
    };
    run_webhook_handler_test_case(case).await;
}

#[tokio::test]
async fn test_webhook_handler_rule_without_branch() {
    let case = WebhookHandlerTestCase {
        name: "match_any_branch",
        mock_changed_files: vec!["src/main.rs"],
        rules_data: vec![(
            "any-branch-rule",
            Rule {
                description: Some("Rule matching any branch".to_string()),
                webhooks: vec!["test-webhook".to_string()],
                event_types: Some(vec!["pr_created".to_string()]),
                branches: None, // Should match feature/test-branch
                paths: Some(vec![PathFilter::Exact {
                    exact: "src/main.rs".to_string(),
                }]),
                actions: vec![Action {
                    http: Some(HttpAction {
                        method: "POST".to_string(),
                        url: "https://example.com/any_branch".to_string(),
                        headers: None,
                        body: None,
                    }),
                    shell: None,
                }],
            },
        )],
        payload: create_pr_opened_payload(),
        expected_actions_count: 1,
        expected_action_urls: vec!["https://example.com/any_branch"],
    };
    run_webhook_handler_test_case(case).await;
}

#[tokio::test]
async fn test_webhook_handler_rule_without_path() {
    let case = WebhookHandlerTestCase {
        name: "match_any_path",
        mock_changed_files: vec!["docs/other.md"],
        rules_data: vec![(
            "any-path-rule",
            Rule {
                description: Some("Rule matching any path".to_string()),
                webhooks: vec!["test-webhook".to_string()],
                event_types: Some(vec!["pr_created".to_string()]),
                branches: None,
                paths: None, // Should match any changed file
                actions: vec![Action {
                    http: Some(HttpAction {
                        method: "POST".to_string(),
                        url: "https://example.com/any_path".to_string(),
                        headers: None,
                        body: None,
                    }),
                    shell: None,
                }],
            },
        )],
        payload: create_pr_opened_payload(),
        expected_actions_count: 1,
        expected_action_urls: vec!["https://example.com/any_path"],
    };
    run_webhook_handler_test_case(case).await;
}
