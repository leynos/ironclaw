//! Tests for `validate_strict_schema`, including complex fixture-backed tool schemas.

use super::*;

fn representative_complex_tool_schemas() -> Vec<(&'static str, serde_json::Value)> {
    vec![
        (
            "tool_search",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query (name, keyword, or description fragment)"
                    },
                    "discover": {
                        "type": "boolean",
                        "description": "If true, also search online (slower, 5-15s). Try without first.",
                        "default": false
                    }
                },
                "required": ["query"]
            }),
        ),
        (
            "tool_install",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Extension name (from search results or custom)"
                    },
                    "url": {
                        "type": "string",
                        "description": "Explicit URL (for extensions not in the registry)"
                    },
                    "kind": {
                        "type": "string",
                        "enum": ["mcp_server", "wasm_tool", "wasm_channel"],
                        "description": "Extension type (auto-detected if omitted)"
                    }
                },
                "required": ["name"]
            }),
        ),
        (
            "tool_auth",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Extension name to authenticate"
                    }
                },
                "required": ["name"]
            }),
        ),
        (
            "tool_activate",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Extension name to activate"
                    }
                },
                "required": ["name"]
            }),
        ),
        (
            "tool_remove",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name of the installed extension to remove"
                    }
                },
                "required": ["name"]
            }),
        ),
        (
            "routine_create",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Unique name for the routine (e.g. 'daily-pr-review')"
                    },
                    "description": {
                        "type": "string",
                        "description": "What this routine does"
                    },
                    "trigger_type": {
                        "type": "string",
                        "enum": ["cron", "event", "system_event", "manual"],
                        "description": "When the routine fires"
                    },
                    "schedule": {
                        "type": "string",
                        "description": "Cron expression (for cron trigger). E.g. '0 9 * * MON-FRI' for weekdays at 9am. Uses 6-field cron (sec min hour day month weekday)."
                    },
                    "event_pattern": {
                        "type": "string",
                        "description": "Regex pattern to match messages (for event trigger)"
                    },
                    "event_channel": {
                        "type": "string",
                        "description": "Optional channel filter for event trigger (e.g. 'telegram')"
                    },
                    "event_source": {
                        "type": "string",
                        "description": "Event source for system_event triggers (e.g. 'github')"
                    },
                    "event_type": {
                        "type": "string",
                        "description": "Event type for system_event triggers (e.g. 'issue.opened')"
                    },
                    "event_filters": {
                        "type": "object",
                        "description": "Optional exact-match filters against payload fields for system_event triggers. Values can be strings, numbers, or booleans.",
                        "additionalProperties": {
                            "type": ["string", "number", "boolean"]
                        }
                    },
                    "prompt": {
                        "type": "string",
                        "description": "The prompt/instructions for the routine"
                    },
                    "context_paths": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Workspace paths to load as context (e.g. ['context/priorities.md'])"
                    },
                    "action_type": {
                        "type": "string",
                        "enum": ["lightweight", "full_job"],
                        "description": "Execution mode: 'lightweight' (single LLM call, default) or 'full_job' (multi-turn with tools)"
                    },
                    "cooldown_secs": {
                        "type": "integer",
                        "description": "Minimum seconds between fires (default: 300)"
                    },
                    "tool_permissions": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Tool names pre-authorized for Always-approval tools in full_job mode (e.g. ['shell']). UnlessAutoApproved tools are automatically permitted in routines."
                    },
                    "notify_channel": {
                        "type": "string",
                        "description": "Channel to send results to (e.g. 'telegram', 'slack', 'tui'). Sets the default channel for message tool calls in routine jobs."
                    },
                    "notify_user": {
                        "type": "string",
                        "description": "User/target to notify (e.g. username, chat ID). Defaults to 'default'."
                    },
                    "timezone": {
                        "type": "string",
                        "description": "IANA timezone for cron schedule evaluation (e.g. 'America/New_York'). Defaults to UTC."
                    }
                },
                "required": ["name", "trigger_type", "prompt"]
            }),
        ),
        (
            "routine_update",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name of the routine to update"
                    },
                    "enabled": {
                        "type": "boolean",
                        "description": "Enable or disable the routine"
                    },
                    "prompt": {
                        "type": "string",
                        "description": "New prompt/instructions"
                    },
                    "schedule": {
                        "type": "string",
                        "description": "New cron schedule (for cron triggers)"
                    },
                    "timezone": {
                        "type": "string",
                        "description": "IANA timezone for cron schedule (e.g. 'America/New_York'). Only valid for cron triggers."
                    },
                    "description": {
                        "type": "string",
                        "description": "New description"
                    }
                },
                "required": ["name"]
            }),
        ),
        (
            "job_events",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "job_id": {
                        "type": "string",
                        "description": "The job ID (full UUID or short prefix, e.g. 'f2854dd8')"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of events to return (default 50, most recent)"
                    }
                },
                "required": ["job_id"]
            }),
        ),
        (
            "event_emit",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "event_source": {
                        "type": "string",
                        "description": "Event source (e.g. 'github', 'workflow', 'tool')"
                    },
                    "event_type": {
                        "type": "string",
                        "description": "Event type (e.g. 'issue.opened', 'pr.ready')"
                    },
                    "payload": {
                        "type": "object",
                        "description": "Structured event payload",
                        "additionalProperties": {
                            "type": ["string", "number", "boolean"]
                        }
                    }
                },
                "required": ["event_source", "event_type"]
            }),
        ),
        (
            "job_prompt",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "job_id": {
                        "type": "string",
                        "description": "The job ID (full UUID or short prefix, e.g. 'f2854dd8')"
                    },
                    "content": {
                        "type": "string",
                        "description": "The follow-up prompt text to send"
                    },
                    "done": {
                        "type": "boolean",
                        "description": "If true, signals the sub-agent that no more prompts are coming                                     and it should finish up. Default false."
                    }
                },
                "required": ["job_id", "content"]
            }),
        ),
    ]
}

#[test]
fn test_valid_schema_passes() {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "name": { "type": "string", "description": "A name" }
        },
        "required": ["name"]
    });
    assert!(validate_strict_schema(&schema, "test").is_ok());
}

#[test]
fn test_missing_type_fails() {
    let schema = serde_json::json!({
        "properties": {
            "name": { "type": "string" }
        }
    });
    let err = validate_strict_schema(&schema, "test").unwrap_err();
    assert!(err[0].contains("missing \"type\": \"object\""));
}

#[test]
fn test_wrong_type_fails() {
    let schema = serde_json::json!({ "type": "string" });
    let err = validate_strict_schema(&schema, "test").unwrap_err();
    assert!(err[0].contains("expected type \"object\""));
}

#[test]
fn test_required_not_in_properties_fails() {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" }
        },
        "required": ["name", "age"]
    });
    let err = validate_strict_schema(&schema, "test").unwrap_err();
    assert!(err.iter().any(|e| e.contains("\"age\" not found")));
}

#[test]
fn test_nested_object_validated() {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "config": {
                "type": "object",
                "properties": {
                    "key": { "type": "string" }
                },
                "required": ["key", "missing"]
            }
        }
    });
    let err = validate_strict_schema(&schema, "test").unwrap_err();
    assert!(
        err.iter()
            .any(|e| e.contains("test.config") && e.contains("\"missing\""))
    );
}

#[test]
fn test_array_missing_items_fails() {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "tags": { "type": "array", "description": "Tags" }
        }
    });
    let err = validate_strict_schema(&schema, "test").unwrap_err();
    assert!(
        err.iter()
            .any(|e| e.contains("array property missing \"items\""))
    );
}

#[test]
fn test_array_with_items_passes() {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "tags": {
                "type": "array",
                "items": { "type": "string" }
            }
        }
    });
    assert!(validate_strict_schema(&schema, "test").is_ok());
}

#[test]
fn test_forbidden_top_level_keywords_fail() {
    for keyword in ["oneOf", "anyOf", "allOf", "enum", "not"] {
        let mut schema = serde_json::json!({
            "type": "object"
        });
        let root = schema
            .as_object_mut()
            .expect("top-level schema should be an object");

        match keyword {
            "enum" => {
                root.insert(keyword.to_string(), serde_json::json!(["get_repo"]));
            }
            "not" => {
                root.insert(keyword.to_string(), serde_json::json!({ "type": "null" }));
            }
            _ => {
                root.insert(
                    keyword.to_string(),
                    serde_json::json!([
                        {
                            "properties": {
                                "action": { "const": "get_repo" }
                            },
                            "required": ["action"]
                        }
                    ]),
                );
            }
        };

        let err = validate_strict_schema(&schema, "test").unwrap_err();
        assert!(
            err.iter().any(|message| {
                message.contains(&format!("top-level \"{keyword}\" is not allowed"))
            }),
            "expected top-level {keyword} failure, got: {err:?}"
        );
    }
}

#[test]
fn test_nested_one_of_is_allowed() {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "action": { "type": "string" },
            "inputs": {
                "type": "object",
                "properties": {
                    "mode": {
                        "oneOf": [
                            { "type": "string" },
                            { "type": "integer" }
                        ]
                    }
                },
                "required": ["mode"],
                "additionalProperties": false
            }
        },
        "required": ["action"]
    });

    assert!(
        validate_strict_schema(&schema, "test").is_ok(),
        "nested combinators should not be rejected by root-only validation"
    );
}

#[test]
fn test_enum_type_mismatch_fails() {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "mode": {
                "type": "string",
                "enum": ["fast", 42, "slow"]
            }
        }
    });
    let err = validate_strict_schema(&schema, "test").unwrap_err();
    assert!(err.iter().any(|e| e.contains("enum[1]")));
}

#[test]
fn test_enum_matching_type_passes() {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "mode": {
                "type": "string",
                "enum": ["fast", "slow"]
            }
        }
    });
    assert!(validate_strict_schema(&schema, "test").is_ok());
}

#[test]
fn test_nested_array_items_object_validated() {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "headers": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" }
                    },
                    "required": ["name", "ghost"]
                }
            }
        }
    });
    let err = validate_strict_schema(&schema, "test").unwrap_err();
    assert!(
        err.iter()
            .any(|e| e.contains("headers.items") && e.contains("\"ghost\""))
    );
}

#[test]
fn test_additional_properties_false_passes() {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "header": {
                "type": "object",
                "properties": {
                    "name": { "type": "string" }
                },
                "additionalProperties": false
            }
        }
    });
    assert!(validate_strict_schema(&schema, "test").is_ok());
}

#[test]
fn test_additional_properties_type_schema_passes() {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "credentials": {
                "type": "object",
                "description": "Map of secret names to env var names",
                "additionalProperties": { "type": "string" }
            }
        }
    });
    assert!(validate_strict_schema(&schema, "test").is_ok());
}

#[test]
fn test_all_simple_tool_schemas() {
    use crate::tools::Tool;
    use crate::tools::builtin::{
        ApplyPatchTool, EchoTool, HttpTool, JsonTool, ListDirTool, ReadFileTool, ShellTool,
        TimeTool, WriteFileTool,
    };

    let tools: Vec<Box<dyn Tool>> = vec![
        Box::new(EchoTool),
        Box::new(TimeTool),
        Box::new(JsonTool),
        Box::new(HttpTool::new()),
        Box::new(ShellTool::new()),
        Box::new(ReadFileTool::new()),
        Box::new(WriteFileTool::new()),
        Box::new(ListDirTool::new()),
        Box::new(ApplyPatchTool::new()),
    ];

    let mut failures = Vec::new();

    for tool in &tools {
        let schema = tool.parameters_schema();
        if let Err(errors) = validate_strict_schema(&schema, tool.name()) {
            failures.push(format!("Tool '{}': {}", tool.name(), errors.join("; ")));
        }
    }

    assert!(
        failures.is_empty(),
        "Schema validation failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn test_job_tool_schemas() {
    use std::sync::Arc;

    use crate::context::ContextManager;
    use crate::tools::Tool;
    use crate::tools::builtin::{CancelJobTool, CreateJobTool, JobStatusTool, ListJobsTool};

    let ctx_mgr = Arc::new(ContextManager::new(5));

    let tools: Vec<Box<dyn Tool>> = vec![
        Box::new(CreateJobTool::new(Arc::clone(&ctx_mgr))),
        Box::new(ListJobsTool::new(Arc::clone(&ctx_mgr))),
        Box::new(JobStatusTool::new(Arc::clone(&ctx_mgr))),
        Box::new(CancelJobTool::new(Arc::clone(&ctx_mgr))),
    ];

    let mut failures = Vec::new();

    for tool in &tools {
        let schema = tool.parameters_schema();
        if let Err(errors) = validate_strict_schema(&schema, tool.name()) {
            failures.push(format!("Tool '{}': {}", tool.name(), errors.join("; ")));
        }
    }

    assert!(
        failures.is_empty(),
        "Schema validation failures:\n{}",
        failures.join("\n")
    );
}

#[test]
fn test_skill_tool_schemas() {
    use std::sync::Arc;

    use crate::skills::catalog::SkillCatalog;
    use crate::skills::registry::SkillRegistry;
    use crate::tools::Tool;
    use crate::tools::builtin::{
        SkillInstallTool, SkillListTool, SkillRemoveTool, SkillSearchTool,
    };

    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.keep();
    let registry = Arc::new(std::sync::RwLock::new(SkillRegistry::new(path)));
    let catalog = Arc::new(SkillCatalog::with_url("http://127.0.0.1:1"));

    let tools: Vec<Box<dyn Tool>> = vec![
        Box::new(SkillListTool::new(Arc::clone(&registry))),
        Box::new(SkillSearchTool::new(
            Arc::clone(&registry),
            Arc::clone(&catalog),
        )),
        Box::new(SkillInstallTool::new(
            Arc::clone(&registry),
            Arc::clone(&catalog),
        )),
        Box::new(SkillRemoveTool::new(Arc::clone(&registry))),
    ];

    let mut failures = Vec::new();

    for tool in &tools {
        let schema = tool.parameters_schema();
        if let Err(errors) = validate_strict_schema(&schema, tool.name()) {
            failures.push(format!("Tool '{}': {}", tool.name(), errors.join("; ")));
        }
    }

    assert!(
        failures.is_empty(),
        "Schema validation failures:\n{}",
        failures.join("\n")
    );
}

/// Validate representative schemas from tools whose constructors require
/// heavier dependencies than this unit test should need to assemble.
#[test]
fn test_inline_schemas_for_complex_tools() {
    let schemas = representative_complex_tool_schemas();

    let mut failures = Vec::new();

    for (name, schema) in &schemas {
        if let Err(errors) = validate_strict_schema(schema, name) {
            failures.push(format!("Tool '{}': {}", name, errors.join("; ")));
        }
    }

    assert!(
        failures.is_empty(),
        "Schema validation failures for inline schemas:\n{}",
        failures.join("\n")
    );
}

/// Verify the validator catches common issues in externally-sourced schemas.
/// WASM modules and MCP servers may produce schemas with defects that
/// built-in tools wouldn't have.
#[test]
fn test_external_schema_defects_detected() {
    let bad_no_type = serde_json::json!({
        "properties": {
            "query": { "type": "string" }
        }
    });
    assert!(validate_strict_schema(&bad_no_type, "ext_no_type").is_err());

    let bad_required = serde_json::json!({
        "type": "object",
        "properties": {
            "input": { "type": "string" }
        },
        "required": ["inpt"]
    });
    assert!(validate_strict_schema(&bad_required, "ext_typo").is_err());

    let bad_array = serde_json::json!({
        "type": "object",
        "properties": {
            "tags": { "type": "array" }
        }
    });
    assert!(validate_strict_schema(&bad_array, "ext_no_items").is_err());

    let bad_enum = serde_json::json!({
        "type": "object",
        "properties": {
            "mode": {
                "type": "string",
                "enum": [1, 2, 3]
            }
        }
    });
    assert!(validate_strict_schema(&bad_enum, "ext_enum_mismatch").is_err());

    let bad_nested = serde_json::json!({
        "type": "object",
        "properties": {
            "config": {
                "type": "object",
                "properties": {
                    "setting": { "description": "missing type field" }
                }
            }
        }
    });
    let _ = validate_strict_schema(&bad_nested, "ext_nested_no_type");
}
