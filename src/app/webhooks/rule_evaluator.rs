use crate::app::config::{
    rules::{BranchFilter, PathFilter},
    Rule,
};
use crate::app::webhooks::types::{Branch, Event, EventType, Path};
use glob::Pattern;
use regex::Regex;
use tracing::{debug, error};
use wildmatch::WildMatch;

pub fn check(event: &Event, rule: &Rule) -> bool {
    // check event type
    let result = check_event_type(&event.event_type, &rule.event_types);
    if !result {
        debug!("Event type eval FAILED: {}", event.event_type.to_string());
        return false;
    } else {
        debug!("Event type eval OK: {}", event.event_type.to_string());
    }

    // check branch
    let result = check_branch(&event.branch, &rule.branches);
    if !result {
        debug!("Branch eval FAILED: {}", event.branch);
        return false;
    } else {
        debug!("Branch eval OK: {}", event.branch);
    }

    // check paths / changed files
    let result = check_changed_files(&event.changed_files, &rule.paths);
    if !result {
        debug!("Changed files eval FAILED: {:?}", event.changed_files);
        return false;
    } else {
        debug!("Changed files eval OK: {:?}", event.changed_files);
    }

    true
}

fn check_event_type(event_type: &EventType, rule_event_types: &Option<Vec<String>>) -> bool {
    let rule_event_types = match rule_event_types {
        // if None or empty, then it matches any event type
        None => return true,
        Some(event_types) if event_types.is_empty() => return true,
        Some(event_types) => event_types,
    };

    // check each one
    for rule_event_type in rule_event_types {
        if &event_type.to_string() == rule_event_type {
            return true;
        }
    }

    false
}

// TODO for PRs, a rule could be matching by source / target branch
// TODO return matched branch name instead of bool
fn check_branch(event_branch: &Branch, rule_branches: &Option<Vec<BranchFilter>>) -> bool {
    let rule_branches = match rule_branches {
        // if None or empty, then it matches any branch
        None => return true,
        Some(rule_branches) if rule_branches.is_empty() => return true,
        Some(rule_branches) => rule_branches,
    };

    // check each one
    for branch_filter in rule_branches {
        match branch_filter {
            BranchFilter::Exact { exact } => {
                if exact == event_branch {
                    debug!("Branch matches exact: {}", exact);
                    return true;
                }
            }
            BranchFilter::Pattern { pattern } => {
                let result = WildMatch::new(pattern).matches(event_branch);
                if result {
                    debug!("Branch matches wildcard: {}", pattern);
                    return true;
                }
            }
            BranchFilter::Regex { regex } => {
                // TODO implement config validation for regex
                let regex = match Regex::new(regex) {
                    Ok(regex) => regex,
                    Err(e) => {
                        error!("Invalid regex: {}", e);
                        continue;
                    }
                };
                if regex.is_match(event_branch) {
                    debug!("Branch matches regex: {}", regex);
                    return true;
                }
            }
        }
    }

    false
}

// TODO return matched paths instead of bool
fn check_changed_files(event_paths: &Vec<Path>, rule_paths: &Option<Vec<PathFilter>>) -> bool {
    let rule_paths = match rule_paths {
        // if None or empty, then it matches any path
        None => return true,
        Some(rule_paths) if rule_paths.is_empty() => return true,
        Some(rule_paths) => rule_paths,
    };

    // check each event_paths
    for event_path in event_paths {
        for path_filter in rule_paths {
            match path_filter {
                PathFilter::Exact { exact } => {
                    if exact == event_path {
                        debug!("Path matches exact: {}", exact);
                        return true;
                    }
                }
                PathFilter::Pattern { pattern } => {
                    let pattern = match Pattern::new(pattern) {
                        Ok(pattern) => pattern,
                        _ => {
                            error!("Invalid pattern: {}", pattern);
                            continue;
                        }
                    };
                    let result = pattern.matches(event_path);
                    if result {
                        debug!("Path matches wildcard: {}", pattern);
                        return true;
                    }
                }
                PathFilter::Regex { regex } => {
                    // TODO implement config validation for regex
                    let regex = match Regex::new(regex) {
                        Ok(regex) => regex,
                        Err(e) => {
                            error!("Invalid regex: {}", e);
                            continue;
                        }
                    };
                    if regex.is_match(event_path) {
                        debug!("Path matches regex: {}", regex);
                        return true;
                    }
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::webhooks::types::{Event, EventType};
    use crate::app::config::rules::{BranchFilter, PathFilter, Rule};

    #[test]
    fn test_check_branch_exact_match() {
        // Setup
        let event_branch = "feature/new-feature".to_string();
        let rule_branches = Some(vec![
            BranchFilter::Exact { exact: "feature/new-feature".to_string() }
        ]);
        
        // Execute
        let result = check_branch(&event_branch, &rule_branches);
        
        // Verify
        assert!(result);
    }

    #[test]
    fn test_check_branch_no_match() {
        // Setup
        let event_branch = "feature/new-feature".to_string();
        let rule_branches = Some(vec![
            BranchFilter::Exact { exact: "main".to_string() }
        ]);
        
        // Execute
        let result = check_branch(&event_branch, &rule_branches);
        
        // Verify
        assert!(!result);
    }

    #[test]
    fn test_check_branch_pattern_match() {
        // Setup
        let event_branch = "feature/new-feature".to_string();
        let rule_branches = Some(vec![
            BranchFilter::Pattern { pattern: "feature/*".to_string() }
        ]);
        
        // Execute
        let result = check_branch(&event_branch, &rule_branches);
        
        // Verify
        assert!(result);
    }

    #[test]
    fn test_check_branch_regex_match() {
        // Setup
        let event_branch = "feature/new-feature".to_string();
        let rule_branches = Some(vec![
            BranchFilter::Regex { regex: "feature/[\\w]+-[\\w]".to_string() }
        ]);
        
        // Execute
        let result = check_branch(&event_branch, &rule_branches);
        
        // Verify
        assert!(result);
    }

    #[test]
    fn test_check_branch_empty_rules_should_pass() {
        // Setup
        let event_branch = "feature/new-feature".to_string();
        let rule_branches = Some(vec![]);
        
        // Execute
        let result = check_branch(&event_branch, &rule_branches);
        
        // Verify
        assert!(result);
    }

    #[test]
    fn test_check_branch_no_rules_should_pass() {
        // Setup
        let event_branch = "feature/new-feature".to_string();
        let rule_branches = None;
        
        // Execute
        let result = check_branch(&event_branch, &rule_branches);
        
        // Verify
        assert!(result);
    }

    #[test]
    fn test_check_event_type_match() {
        // Setup
        let event_type = EventType::Opened;
        let rule_event_types = Some(vec!["pr_created".to_string()]);
        
        // Execute
        let result = check_event_type(&event_type, &rule_event_types);
        
        // Verify
        assert!(result);
    }

    #[test]
    fn test_check_event_type_no_match() {
        // Setup
        let event_type = EventType::Opened;
        let rule_event_types = Some(vec!["pr_modified".to_string()]);
        
        // Execute
        let result = check_event_type(&event_type, &rule_event_types);
        
        // Verify
        assert!(!result);
    }

    #[test]
    fn test_check_event_type_empty_rules_should_pass() {
        // Setup
        let event_type = EventType::Opened;
        let rule_event_types = Some(vec![]);
        
        // Execute
        let result = check_event_type(&event_type, &rule_event_types);
        
        // Verify
        assert!(result);
    }

    #[test]
    fn test_check_event_type_none_rules_should_pass() {
        // Setup
        let event_type = EventType::Opened;
        let rule_event_types = None;
        
        // Execute
        let result = check_event_type(&event_type, &rule_event_types);
        
        // Verify
        assert!(result);
    }

    #[test]
    fn test_check_changed_files_exact_match() {
        // Setup
        let event_paths = vec!["src/main.rs".to_string()];
        let rule_paths = Some(vec![
            PathFilter::Exact { exact: "src/main.rs".to_string() }
        ]);
        
        // Execute
        let result = check_changed_files(&event_paths, &rule_paths);
        
        // Verify
        assert!(result);
    }

    #[test]
    fn test_check_changed_files_no_match() {
        // Setup
        let event_paths = vec!["src/main.rs".to_string()];
        let rule_paths = Some(vec![
            PathFilter::Exact { exact: "src/lib.rs".to_string() }
        ]);
        
        // Execute
        let result = check_changed_files(&event_paths, &rule_paths);
        
        // Verify
        assert!(!result);
    }

    #[test]
    fn test_check_changed_files_pattern_match() {
        // Setup
        let event_paths = vec!["src/main.rs".to_string()];
        let rule_paths = Some(vec![
            PathFilter::Pattern { pattern: "src/*.rs".to_string() }
        ]);
        
        // Execute
        let result = check_changed_files(&event_paths, &rule_paths);
        
        // Verify
        assert!(result);
    }

    #[test]
    fn test_check_changed_files_regex_match() {
        // Setup
        let event_paths = vec!["src/main.rs".to_string()];
        let rule_paths = Some(vec![
            PathFilter::Regex { regex: "src/.*\\.rs".to_string() }
        ]);
        
        // Execute
        let result = check_changed_files(&event_paths, &rule_paths);
        
        // Verify
        assert!(result);
    }

    #[test]
    fn test_check_complete_rule_match() {
        // Setup
        let event = Event {
            event_type: EventType::Opened,
            branch: "feature/new-feature".to_string(),
            changed_files: vec!["src/main.rs".to_string()]
        };
        
        let rule = Rule {
            description: Some("Test rule".to_string()),
            webhooks: vec!["test-webhook".to_string()],
            event_types: Some(vec!["pr_created".to_string()]),
            branches: Some(vec![BranchFilter::Pattern { pattern: "feature/*".to_string() }]),
            paths: Some(vec![PathFilter::Pattern { pattern: "src/*.rs".to_string() }]),
            actions: vec![] // Empty for this test
        };
        
        // Execute
        let result = check(&event, &rule);
        
        // Verify
        assert!(result);
    }

    #[test]
    fn test_check_complete_rule_no_match_event_type() {
        // Setup
        let event = Event {
            event_type: EventType::Modified,
            branch: "feature/new-feature".to_string(),
            changed_files: vec!["src/main.rs".to_string()]
        };
        
        let rule = Rule {
            description: Some("Test rule".to_string()),
            webhooks: vec!["test-webhook".to_string()],
            event_types: Some(vec!["pr_created".to_string()]),
            branches: Some(vec![BranchFilter::Pattern { pattern: "feature/*".to_string() }]),
            paths: Some(vec![PathFilter::Pattern { pattern: "src/*.rs".to_string() }]),
            actions: vec![] // Empty for this test
        };
        
        // Execute
        let result = check(&event, &rule);
        
        // Verify
        assert!(!result);
    }

    #[test]
    fn test_check_complete_rule_no_match_branch() {
        // Setup
        let event = Event {
            event_type: EventType::Opened,
            branch: "main".to_string(),
            changed_files: vec!["src/main.rs".to_string()]
        };
        
        let rule = Rule {
            description: Some("Test rule".to_string()),
            webhooks: vec!["test-webhook".to_string()],
            event_types: Some(vec!["pr_created".to_string()]),
            branches: Some(vec![BranchFilter::Pattern { pattern: "feature/*".to_string() }]),
            paths: Some(vec![PathFilter::Pattern { pattern: "src/*.rs".to_string() }]),
            actions: vec![] // Empty for this test
        };
        
        // Execute
        let result = check(&event, &rule);
        
        // Verify
        assert!(!result);
    }

    #[test]
    fn test_check_complete_rule_no_match_path() {
        // Setup
        let event = Event {
            event_type: EventType::Opened,
            branch: "feature/new-feature".to_string(),
            changed_files: vec!["docs/README.md".to_string()]
        };
        
        let rule = Rule {
            description: Some("Test rule".to_string()),
            webhooks: vec!["test-webhook".to_string()],
            event_types: Some(vec!["pr_created".to_string()]),
            branches: Some(vec![BranchFilter::Pattern { pattern: "feature/*".to_string() }]),
            paths: Some(vec![PathFilter::Pattern { pattern: "src/*.rs".to_string() }]),
            actions: vec![] // Empty for this test
        };
        
        // Execute
        let result = check(&event, &rule);
        
        // Verify
        assert!(!result);
    }
}
