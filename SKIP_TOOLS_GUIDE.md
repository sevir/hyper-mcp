# Skip Tools Pattern Guide

This guide provides comprehensive documentation for using the `skip_tools` configuration in hyper-mcp, which allows you to filter out unwanted tools using powerful regex patterns.

## Overview

The `skip_tools` field in your plugin's `runtime_config` allows you to specify a list of regex patterns that will be used to exclude tools from being loaded at runtime. This is useful for:

- Removing debug tools in production environments
- Filtering out deprecated or experimental tools
- Excluding tools that conflict with your workflow
- Customizing the available tool set per environment

## How It Works

### Automatic Pattern Anchoring

All patterns in `skip_tools` are automatically anchored to match the entire tool name. This means:

```yaml
skip_tools:
  - "debug"  # Becomes "^debug$" - matches exactly "debug"
```

This prevents unintended partial matches. If you want to match parts of tool names, use explicit wildcards:

```yaml
skip_tools:
  - "debug.*"  # Matches "debug", "debugger", "debug_info", etc.
```

### Regex Compilation

All patterns are compiled into a single optimized `RegexSet` for efficient matching:
- O(1) lookup time regardless of pattern count
- Single compilation at startup
- Memory-efficient pattern storage

## Basic Patterns

### Exact Matches

Match specific tool names exactly:

```yaml
skip_tools:
  - "debug_tool"      # Matches only "debug_tool"
  - "test_runner"     # Matches only "test_runner"
  - "admin_panel"     # Matches only "admin_panel"
```

### Prefix Matching

Match tools that start with a specific string:

```yaml
skip_tools:
  - "debug.*"         # Matches "debug", "debugger", "debug_info"
  - "test_.*"         # Matches "test_unit", "test_integration", "test_e2e"
  - "dev_.*"          # Matches "dev_server", "dev_tools", "dev_helper"
```

### Suffix Matching

Match tools that end with a specific string:

```yaml
skip_tools:
  - ".*_test"         # Matches "unit_test", "integration_test", "load_test"
  - ".*_backup"       # Matches "data_backup", "config_backup", "db_backup"
  - ".*_deprecated"   # Matches "old_deprecated", "legacy_deprecated"
```

### Contains Matching

Match tools that contain a specific substring:

```yaml
skip_tools:
  - ".*debug.*"       # Matches "pre_debug_tool", "debug", "tool_debug_info"
  - ".*temp.*"        # Matches "temp_file", "cleanup_temp", "temp_storage_tool"
```

## Advanced Patterns

### Character Classes

Use character classes for flexible matching:

```yaml
skip_tools:
  - "tool_[0-9]+"           # Matches "tool_1", "tool_42", "tool_999"
  - "test_[a-z]+"           # Matches "test_unit", "test_api", "test_db"
  - "[A-Z][a-z]+Tool"       # Matches "DebugTool", "TestTool", "AdminTool"
  - "log_[0-9]{4}_[0-9]{2}" # Matches "log_2023_12", "log_2024_01"
```

### Alternation (OR Logic)

Match multiple alternatives:

```yaml
skip_tools:
  - "test_(unit|integration|e2e)"     # Matches "test_unit", "test_integration", "test_e2e"
  - "(debug|trace|log)_.*"            # Matches tools starting with "debug_", "trace_", or "log_"
  - ".*(temp|tmp|cache).*"            # Matches tools containing "temp", "tmp", or "cache"
  - "system_(admin|user|guest)_.*"    # Matches tools for different user types
```

### Quantifiers

Control how many characters or groups to match:

```yaml
skip_tools:
  - "tool_v[0-9]+"          # Matches "tool_v1", "tool_v10", "tool_v123"
  - "backup_[0-9]{8}"       # Matches exactly 8 digits: "backup_20240101"
  - "temp_[a-f0-9]{6,}"     # Matches 6+ hex chars: "temp_abc123", "temp_def456789"
  - "log_[0-9]{4}-[0-9]{2}" # Matches "log_2024-01", "log_2023-12"
```

### Negation with Character Classes

Skip tools that DON'T match certain patterns:

```yaml
skip_tools:
  - "[^a-z].*"              # Skip tools starting with non-lowercase letters
  - ".*[^0-9]$"             # Skip tools not ending with numbers
  - "tool_[^v].*"           # Skip tools starting with "tool_" but not "tool_v"
```

## Common Use Cases

### Environment-Specific Filtering

#### Development Environment
```yaml
skip_tools:
  - "prod_.*"               # Skip production tools
  - "deploy_.*"             # Skip deployment tools
  - "monitor_.*"            # Skip monitoring tools
```

#### Production Environment
```yaml
skip_tools:
  - "debug.*"               # Skip all debug tools
  - "test_.*"               # Skip all test tools
  - "dev_.*"                # Skip development tools
  - "mock_.*"               # Skip mock/stub tools
  - ".*_experimental"       # Skip experimental features
```

#### Testing Environment
```yaml
skip_tools:
  - "prod_.*"               # Skip production tools
  - "deploy_.*"             # Skip deployment tools
  - ".*_live"               # Skip live/production tools
```

### Tool Category Filtering

#### Skip Administrative Tools
```yaml
skip_tools:
  - "admin_.*"
  - "system_admin_.*"
  - "user_management_.*"
  - "permission_.*"
```

#### Skip Deprecated Tools
```yaml
skip_tools:
  - ".*_deprecated"
  - ".*_old"
  - "legacy_.*"
  - "v[0-9]_.*"             # Skip versioned legacy tools
```

#### Skip Resource-Heavy Tools
```yaml
skip_tools:
  - ".*_benchmark"
  - "load_test_.*"
  - "stress_.*"
  - "heavy_.*"
```

### Version-Based Filtering

```yaml
skip_tools:
  - ".*_v[0-9]"             # Skip v1, v2, etc. (keep latest)
  - ".*_beta"               # Skip beta tools
  - ".*_alpha"              # Skip alpha tools
  - "tool_[0-9]+\\.[0-9]+"  # Skip versioned tools like "tool_1.0"
```

## Special Character Escaping

When matching literal special characters, escape them with backslashes:

```yaml
skip_tools:
  - "file\\.exe"            # Matches "file.exe" literally
  - "script\\?"             # Matches "script?" literally
  - "temp\\*data"           # Matches "temp*data" literally
  - "path\\\\tool"          # Matches "path\tool" literally (double escape for backslash)
  - "price\\$calculator"    # Matches "price$calculator" literally
  - "regex\\[test\\]"       # Matches "regex[test]" literally
```

## Configuration Examples

### Simple Configuration
```yaml
plugins:
  my_plugin:
    url: "oci://registry.io/my-plugin:latest"
    runtime_config:
      skip_tools:
        - "debug_tool"
        - "test_runner"
```

### Comprehensive Configuration
```yaml
plugins:
  production_plugin:
    url: "oci://registry.io/prod-plugin:latest"
    runtime_config:
      skip_tools:
        # Exact matches
        - "debug_console"
        - "test_runner"
        
        # Pattern matches
        - "dev_.*"              # All dev tools
        - ".*_test"             # All test tools
        - "temp_.*"             # All temp tools
        - "mock_.*"             # All mock tools
        
        # Advanced patterns
        - "tool_v[0-9]"         # Versioned tools
        - "admin_(user|role)_.*" # Specific admin tools
        - "[0-9]+_backup"       # Numbered backups
        
      allowed_hosts: ["api.example.com"]
      memory_limit: "512Mi"
```

### Multi-Environment Setup
```yaml
# config.dev.yaml
plugins:
  app_plugin:
    url: "oci://registry.io/app-plugin:dev"
    runtime_config:
      skip_tools:
        - "prod_.*"
        - "deploy_.*"

---
# config.prod.yaml  
plugins:
  app_plugin:
    url: "oci://registry.io/app-plugin:latest"
    runtime_config:
      skip_tools:
        - "debug.*"
        - "test_.*"
        - "dev_.*"
        - ".*_experimental"
```

## Best Practices

### 1. Start Simple, Then Refine
```yaml
# Start with broad patterns
skip_tools:
  - "debug.*"
  - "test_.*"

# Refine to be more specific as needed
skip_tools:
  - "debug_(console|panel)"  # Only skip specific debug tools
  - "test_(unit|integration)" # Only skip specific test types
```

### 2. Use Comments for Complex Patterns
```yaml
skip_tools:
  - "tool_[0-9]+"             # Skip numbered tools (tool_1, tool_2, etc.)
  - ".*_(alpha|beta|rc[0-9]+)" # Skip pre-release versions
  - "temp_[0-9]{8}_.*"        # Skip dated temporary tools
```

### 3. Group Related Patterns
```yaml
skip_tools:
  # Debug and development tools
  - "debug.*"
  - "dev_.*"
  - ".*_dev"
  
  # Testing tools
  - "test_.*"
  - ".*_test"
  - "mock_.*"
  
  # Administrative tools
  - "admin_.*"
  - "system_.*"
```

### 4. Consider Performance
```yaml
# Good: Specific patterns
skip_tools:
  - "debug_tool"
  - "test_runner"

# Less optimal: Overly broad patterns that might match many tools
skip_tools:
  - ".*"  # This would skip everything - not useful
```

## Troubleshooting

### Pattern Not Working?

1. **Check anchoring**: Remember patterns are auto-anchored
   ```yaml
   # This matches only "debug" exactly
   - "debug"
   
   # This matches "debug", "debugger", "debug_tool", etc.
   - "debug.*"
   ```

2. **Escape special characters**:
   ```yaml
   # Wrong: Will treat . as wildcard
   - "file.exe"
   
   # Correct: Escapes the literal dot
   - "file\\.exe"
   ```

3. **Test your patterns**: Use a regex tester to validate complex patterns

### Debugging Skip Rules

Enable debug logging to see which tools are being skipped:

```bash
RUST_LOG=debug hyper-mcp --config config.yaml
```

## Migration from Old Format

If you were using simple string arrays before:

```yaml
# Old format (if it existed)
skip_tools: ["debug_tool", "test_runner"]

# New format (same result, but now with regex support)
skip_tools: ["debug_tool", "test_runner"]

# New format with patterns (more powerful)
skip_tools: ["debug.*", "test_.*"]
```

## Error Handling

### Invalid Regex Patterns

If you provide an invalid regex pattern, configuration loading will fail:

```yaml
# This will cause an error - unclosed bracket
skip_tools:
  - "tool_[invalid"
```

Error message will indicate the problematic pattern and suggest corrections.

### Empty Patterns

These configurations are all valid:

```yaml
# No skip_tools field - no tools skipped
runtime_config:
  allowed_hosts: ["*"]

# Empty array - no tools skipped  
runtime_config:
  skip_tools: []

# Null value - no tools skipped
runtime_config:
  skip_tools: null
```

## Performance Characteristics

- **Startup**: O(n) pattern compilation where n = number of patterns
- **Runtime**: O(1) tool name checking regardless of pattern count
- **Memory**: Minimal overhead, patterns compiled into efficient state machine
- **Scalability**: Handles hundreds of patterns efficiently

## Advanced Topics

### Complex Business Logic

```yaml
skip_tools:
  # Skip tools for specific environments
  - "prod_(?!api_).*"         # Skip prod tools except prod_api_*
  - "test_(?!smoke_).*"       # Skip test tools except smoke tests
  
  # Skip based on naming conventions
  - "[A-Z]{2,}_.*"            # Skip tools starting with 2+ capitals
  - ".*_[0-9]{4}[0-9]{2}[0-9]{2}" # Skip daily-dated tools
```

### Integration with External Tools

You can generate `skip_tools` patterns dynamically:

```bash
# Generate patterns from external source
echo "skip_tools:" > config.yaml
external-tool --list-deprecated | sed 's/^/  - "/' | sed 's/$/\"/' >> config.yaml
```

### Conditional Configuration

Use different configs for different scenarios:

```yaml
# Base configuration
base_skip_patterns: &base_skip
  - "debug.*"
  - "test_.*"

# Environment-specific additions
prod_additional: &prod_additional
  - "dev_.*"
  - ".*_experimental"

plugins:
  my_plugin:
    runtime_config:
      skip_tools:
        - *base_skip
        - *prod_additional  # YAML doesn't support this directly, 
                            # but you can use templating tools
```

This guide should help you make full use of the powerful `skip_tools` pattern matching capabilities in hyper-mcp!