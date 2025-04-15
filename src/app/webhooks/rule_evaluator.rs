use crate::app::webhooks::types::{Branch, Event, EventType, Path};
use crate::config::rules_config::{BranchFilter, PathFilter};
use crate::config::Rule;
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
                let result = WildMatch::new(&pattern).matches(&event_branch);
                if result {
                    debug!("Branch matches wildcard: {}", pattern);
                    return true;
                }
            }
            BranchFilter::Regex { regex } => {
                // TODO implement config validation for regex
                let regex = match Regex::new(&regex) {
                    Ok(regex) => regex,
                    Err(e) => {
                        error!("Invalid regex: {}", e);
                        continue;
                    }
                };
                if regex.is_match(&event_branch) {
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
                    let result = pattern.matches(&event_path);
                    if result {
                        debug!("Path matches wildcard: {}", pattern);
                        return true;
                    }
                }
                PathFilter::Regex { regex } => {
                    // TODO implement config validation for regex
                    let regex = match Regex::new(&regex) {
                        Ok(regex) => regex,
                        Err(e) => {
                            error!("Invalid regex: {}", e);
                            continue;
                        }
                    };
                    if regex.is_match(&event_path) {
                        debug!("Path matches regex: {}", regex);
                        return true;
                    }
                }
            }
        }
    }

    false
}
