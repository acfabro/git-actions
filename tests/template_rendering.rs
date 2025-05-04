use git_actions::app::template;
use git_actions::app::webhooks::types::{Event, EventType};
use std::collections::HashMap;
use std::env;
use tera::Context;

#[test]
fn test_template_rendering_in_http_action() {
    // Create a test event
    let event = Event {
        event_type: EventType::PROpened,
        branch: "feature/test-branch".to_string(),
        changed_files: vec!["src/main.rs".to_string(), "Cargo.toml".to_string()],
    };

    // Set up environment variables
    env::set_var("TEST_API_TOKEN", "secret-token-123");
    
    // Build the template context with all environment variables
    let context = template::build_template_context(&event);
    
    // Test URL template rendering
    let url_template = "https://api.example.com/build/{{ event.branch }}";
    let rendered_url = template::render_template(url_template, &context).unwrap();
    assert_eq!(rendered_url, "https://api.example.com/build/feature/test-branch");
    
    // Test headers template rendering
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer {{ env.TEST_API_TOKEN }}".to_string());
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    
    let rendered_headers = template::render_template_map(&headers, &context);
    assert_eq!(rendered_headers.get("Authorization").unwrap(), "Bearer secret-token-123");
    assert_eq!(rendered_headers.get("Content-Type").unwrap(), "application/json");
    
    // Test body template rendering
    let body_template = r#"{
        "branch": "{{ event.branch }}",
        "type": "{{ event.type }}",
        "files": [
            {% for file in event.changed_files %}
                "{{ file }}"{% if not loop.last %},{% endif %}
            {% endfor %}
        ]
    }"#;
    
    let rendered_body = template::render_template(body_template, &context).unwrap();
    assert!(rendered_body.contains(r#""branch": "feature/test-branch""#));
    assert!(rendered_body.contains(r#""type": "pr_created""#));
    assert!(rendered_body.contains(r#""src/main.rs""#));
    assert!(rendered_body.contains(r#""Cargo.toml""#));
    
    // Clean up
    env::remove_var("TEST_API_TOKEN");
}

#[test]
fn test_non_template_strings() {
    // Create a simple context
    let mut context = Context::new();
    context.insert("name", "world");
    
    // Test strings without template markers
    let plain_string = "This is a plain string with no templates";
    let rendered = template::render_template(plain_string, &context).unwrap();
    assert_eq!(rendered, plain_string);
    
    // Test with a map of plain strings
    let mut plain_map = HashMap::new();
    plain_map.insert("key1".to_string(), "value1".to_string());
    plain_map.insert("key2".to_string(), "value2".to_string());
    
    let rendered_map = template::render_template_map(&plain_map, &context);
    assert_eq!(rendered_map.get("key1").unwrap(), "value1");
    assert_eq!(rendered_map.get("key2").unwrap(), "value2");
}