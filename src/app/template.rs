use crate::app::webhooks::types::Event;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use tera::{Context, Tera};
use tracing::{debug, error};

pub fn build_template_context(event: &Event) -> Context {
    let mut context = Context::new();
    
    // Serialize the entire Event object to a Value
    let event_value = serde_json::to_value(event).unwrap_or_else(|e| {
        error!("Failed to serialize event: {}", e);
        Value::Null
    });
    
    // Insert the event data into the context
    context.insert("event", &event_value);
    
    // Add all environment variables to the context
    let mut env_map = serde_json::Map::new();
    for (key, value) in env::vars() {
        env_map.insert(key, Value::String(value));
    }
    context.insert("env", &Value::Object(env_map));
    
    context
}

pub fn render_template(template_str: &str, context: &Context) -> Result<String, tera::Error> {
    // Create a one-off Tera instance for this template
    let mut tera = Tera::default();
    tera.add_raw_template("template", template_str)?;
    
    // Render the template
    let result = tera.render("template", context)?;
    Ok(result)
}

pub fn render_template_map(
    map: &HashMap<String, String>,
    context: &Context,
) -> HashMap<String, String> {
    let mut result = HashMap::new();
    
    for (key, value) in map {
        match render_template(value, context) {
            Ok(rendered) => {
                result.insert(key.clone(), rendered);
            }
            Err(e) => {
                error!("Failed to render template for key {}: {}", key, e);
                result.insert(key.clone(), value.clone());
            }
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::webhooks::types::{Event, EventType};
    use std::env;
    
    #[test]
    fn test_build_template_context() {
        let event = Event {
            event_type: EventType::PROpened,
            branch: "feature/test".to_string(),
            changed_files: vec!["src/main.rs".to_string(), "Cargo.toml".to_string()],
        };
        
        let context = build_template_context(&event);
        
        // Get the event object
        let event_obj = context.get("event").unwrap().as_object().unwrap();
        
        assert_eq!(event_obj.get("type").unwrap().as_str().unwrap(), "pr_created");
        assert_eq!(event_obj.get("branch").unwrap().as_str().unwrap(), "feature/test");
        
        let changed_files = event_obj.get("changed_files").unwrap().as_array().unwrap();
        assert_eq!(changed_files.len(), 2);
        assert_eq!(changed_files[0].as_str().unwrap(), "src/main.rs");
        assert_eq!(changed_files[1].as_str().unwrap(), "Cargo.toml");
    }
    
    #[test]
    fn test_build_template_context_with_env() {
        let event = Event {
            event_type: EventType::PROpened,
            branch: "feature/test".to_string(),
            changed_files: vec!["src/main.rs".to_string()],
        };
        
        // Set a test environment variable
        env::set_var("TEST_TOKEN", "test-value");
        
        // Build context with all environment variables
        let context = build_template_context(&event);
        
        // Check that the environment variable is in the context
        let env_map = context.get("env").unwrap().as_object().unwrap();
        assert_eq!(env_map.get("TEST_TOKEN").unwrap().as_str().unwrap(), "test-value");
        
        // Clean up
        env::remove_var("TEST_TOKEN");
    }
    
    #[test]
    fn test_http_action_template() {
        let event = Event {
            event_type: EventType::PROpened,
            branch: "feature/test".to_string(),
            changed_files: vec!["src/main.rs".to_string()],
        };
        
        // Set a test environment variable
        env::set_var("CI_API_TOKEN", "secret-token");
        
        // Build context with all environment variables
        let context = build_template_context(&event);
        
        // Test URL template
        let url_template = "https://ci-server/api/build/{{ event.branch }}";
        let rendered_url = render_template(url_template, &context).unwrap();
        assert_eq!(rendered_url, "https://ci-server/api/build/feature/test");
        
        // Test header template
        let header_template = "Bearer {{ env.CI_API_TOKEN }}";
        let rendered_header = render_template(header_template, &context).unwrap();
        assert_eq!(rendered_header, "Bearer secret-token");
        
        // Test body template with JSON
        let body_template = r#"{
            "repository": "test-repo",
            "branch": "{{ event.branch }}",
            "type": "{{ event.type }}"
        }"#;
        let rendered_body = render_template(body_template, &context).unwrap();
        assert!(rendered_body.contains(r#""branch": "feature/test""#));
        assert!(rendered_body.contains(r#""type": "pr_created""#));
        
        // Clean up
        env::remove_var("CI_API_TOKEN");
    }
    
    #[test]
    fn test_render_template() {
        let mut context = Context::new();
        context.insert("name", "world");
        
        let result = render_template("Hello, {{ name }}!", &context).unwrap();
        assert_eq!(result, "Hello, world!");
    }
    
    #[test]
    fn test_render_template_no_template_markers() {
        let context = Context::new();
        
        let result = render_template("Hello, world!", &context).unwrap();
        assert_eq!(result, "Hello, world!");
    }
    
    #[test]
    fn test_render_template_map() {
        let mut context = Context::new();
        context.insert("name", "world");
        context.insert("value", "123");
        
        let mut map = HashMap::new();
        map.insert("greeting".to_string(), "Hello, {{ name }}!".to_string());
        map.insert("number".to_string(), "Value: {{ value }}".to_string());
        map.insert("plain".to_string(), "No template here".to_string());
        
        let result = render_template_map(&map, &context);
        
        assert_eq!(result.get("greeting").unwrap(), "Hello, world!");
        assert_eq!(result.get("number").unwrap(), "Value: 123");
        assert_eq!(result.get("plain").unwrap(), "No template here");
    }
}