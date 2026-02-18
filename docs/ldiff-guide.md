---
layout: default
title: Ldiff Guide
---

# Ldiff Guide

`ldiff` is a line difference visualizer that processes input lines, replacing repeated tokens with a substitute character for easy pattern recognition. It's particularly useful for analyzing log files, command output, and any text with recurring patterns.

## Core Concepts

### Pattern Recognition
`ldiff` reads input line by line and compares each line to the previous one. When it finds tokens (words, numbers) that repeat from the previous line, it replaces them with a substitute character, making unique content stand out.

### Token Types
The tool recognizes several types of tokens:
- **Words**: Alphanumeric sequences (`hello`, `world`, `API`)
- **Numbers**: Numeric sequences (`2023`, `404`, `127`)
- **Separators**: Punctuation and symbols (`:`, `-`, `/`, `@`, etc.)
- **Whitespace**: Spaces, tabs, and other whitespace
- **ANSI Codes**: Color codes and terminal escape sequences

### Preservation
While replacing repeated tokens, `ldiff` carefully preserves:
- **ANSI color codes** - Terminal colors remain intact
- **Separators and punctuation** - Structure is maintained
- **Whitespace** - Original spacing and alignment
- **Case sensitivity** - `Hello` and `hello` are different tokens

## Basic Operations

### Simple Usage
```bash
# Read from stdin with default substitute character (░)
echo -e "hello world\nhello universe" | ws ldiff
# Output:
# hello world
# ░░░░░ universe
```

### Custom Substitute Character
```bash
# Use a custom character for substitution
echo -e "test line\ntest another" | ws ldiff "*"
# Output:
# test line
# **** another
```

### Reading from Files
```bash
# Process a file
ws ldiff < input.txt

# Process compressed logs
gunzip -c app.log.gz | ws ldiff

# Use with other commands
cat /var/log/system.log | tail -n 100 | ws ldiff
```

## Log Analysis

### System Logs
```bash
# Analyze system logs for patterns
tail -f /var/log/syslog | ws ldiff

# Find repeated patterns in auth logs
grep "Failed password" /var/log/auth.log | ws ldiff
```

### Application Logs
```bash
# Monitor application logs in real-time
tail -f /var/log/myapp.log | ws ldiff

# Analyze historical logs
cat /var/log/myapp.log.1 | ws ldiff > patterns.txt
```

### Web Server Logs
```bash
# Analyze access patterns
cat /var/log/nginx/access.log | ws ldiff "■"

# Find common request patterns
awk '{print $7}' /var/log/apache2/access.log | sort | ws ldiff
```

### Example Log Analysis
```bash
# Input log lines:
# 2023-01-01 10:00:00 INFO Starting application
# 2023-01-01 10:00:01 INFO Loading configuration
# 2023-01-01 10:00:02 ERROR Failed to connect

cat app.log | ws ldiff
# Output:
# 2023-01-01 10:00:00 INFO Starting application
# ░░░░-░░-░░ ░░:░░:01 ░░░░ Loading configuration
# ░░░░-░░-░░ ░░:░░:02 ERROR Failed to connect
```

## Command Analysis

### File Listings
```bash
# Analyze directory structures
find /usr/local -type f | ws ldiff

# Compare file paths
ls -la /usr/bin/ | ws ldiff "●"
```

### Process Lists
```bash
# Analyze running processes
ps aux | ws ldiff

# Monitor process changes
watch -n 1 "ps aux | head -20" | ws ldiff
```

### Network Analysis
```bash
# Analyze network connections
netstat -tulpn | ws ldiff

# Monitor network activity
ss -tulpn | ws ldiff "█"
```

## Advanced Features

### Unicode Support
```bash
# Use Unicode characters for substitution
echo -e "hello world\nhello universe" | ws ldiff "█"
echo -e "test line\ntest another" | ws ldiff "▓"
echo -e "data point\ndata value" | ws ldiff "◆"
```

### Piping and Redirection
```bash
# Save patterns to file
cat large.log | ws ldiff > patterns.txt

# Combine with other tools
cat access.log | grep "GET" | ws ldiff | head -50

# Chain multiple filters
journalctl -f | grep "ERROR" | ws ldiff "*" | tee error_patterns.log
```

### Real-time Monitoring
```bash
# Monitor multiple log sources
tail -f /var/log/app1.log /var/log/app2.log | ws ldiff

# Monitor system activity
dmesg -w | ws ldiff

# Watch command output
watch -n 2 "df -h" | ws ldiff
```

## Workflow Examples

### Development Debugging
```bash
# 1. Start monitoring your application logs
tail -f /var/log/myapp.log | ws ldiff &

# 2. Reproduce the issue
./trigger_bug.sh

# 3. Analyze the pattern differences
# Repeated timestamp/process info is masked, errors stand out

# 4. Save interesting patterns
pkill tail
cat /var/log/myapp.log | tail -100 | ws ldiff > debug_patterns.txt
```

### System Administration
```bash
# 1. Monitor system health
tail -f /var/log/syslog | ws ldiff "■" &

# 2. Check for unusual patterns
# Normal startup messages will show patterns
# Anomalies will show unique content

# 3. Investigate specific issues
grep "$(date +%Y-%m-%d)" /var/log/syslog | ws ldiff
```

### Performance Analysis
```bash
# 1. Capture performance data
top -b -n 10 | ws ldiff > perf_patterns.txt

# 2. Analyze patterns
cat perf_patterns.txt | grep -v "░"

# 3. Find anomalies
# Look for lines without substitute characters
```

## Configuration Patterns

### Log Timestamp Formats
`ldiff` automatically handles various timestamp formats:
```bash
# ISO 8601: 2023-01-01T10:00:00Z
# Syslog: Jan 01 10:00:00
# Apache: [01/Jan/2023:10:00:00 +0000]
# Custom: 2023-01-01 10:00:00.123
```

### Common Separators
The tool recognizes these separator patterns:
- **Paths**: `/usr/local/bin` → `/░░░/░░░░░/bin`
- **URLs**: `https://api.example.com` → `░░░░░://░░░.░░░░░░░.com`
- **Email**: `user@domain.com` → `░░░░@░░░░░░.com`
- **IPs**: `192.168.1.100` → `░░░.░░░.░.░░░`

## Integration with Other Tools

### With grep
```bash
# Filter then analyze patterns
grep "ERROR" /var/log/app.log | ws ldiff
```

### With awk
```bash
# Extract specific fields then analyze
awk '{print $1, $7}' /var/log/access.log | ws ldiff
```

### With sort/uniq
```bash
# Find unique patterns
cat /var/log/app.log | ws ldiff | sort | uniq -c
```

### With head/tail
```bash
# Analyze recent entries
tail -100 /var/log/app.log | ws ldiff

# Check first patterns
head -50 /var/log/app.log | ws ldiff
```

## Safety Features

### Signal Handling
- **Broken Pipe**: Gracefully handles when output is piped to `head`, `less`, etc.
- **Keyboard Interrupt**: Clean exit on Ctrl+C
- **Input Validation**: Validates substitute character input

### Error Handling
- **No Input**: Clear error message when no stdin is provided
- **Invalid Character**: Warnings for multi-character substitutes
- **File Permissions**: Appropriate error messages for access issues

## Best Practices

### Performance
1. **Large Files**: Use with `tail` or `head` for very large files
2. **Real-time**: Use `-f` flag with `tail` for live monitoring
3. **Memory**: Tool processes line-by-line, memory usage is minimal

### Analysis
1. **Start Simple**: Begin with default substitute character
2. **Use Contrast**: Choose substitute characters that stand out
3. **Save Patterns**: Redirect output to files for later analysis
4. **Combine Tools**: Use with `grep`, `awk`, and other text tools

### Troubleshooting
1. **No Patterns**: If no substitution occurs, lines may be completely different
2. **Too Much Substitution**: Try processing smaller chunks
3. **ANSI Issues**: Some terminals may not display substitute characters correctly

## Common Use Cases

### Security Monitoring
```bash
# Monitor failed login attempts
tail -f /var/log/auth.log | grep "Failed" | ws ldiff

# Analyze access patterns
cat /var/log/nginx/access.log | ws ldiff | grep -v "░"
```

### Application Monitoring
```bash
# Track API response patterns
tail -f /var/log/api.log | grep "response_time" | ws ldiff

# Monitor database queries
tail -f /var/log/mysql/slow.log | ws ldiff
```

### System Analysis
```bash
# Analyze process startup patterns
dmesg | grep "systemd" | ws ldiff

# Monitor resource usage patterns
vmstat 1 | ws ldiff
```

## See Also

- **[Installation Guide]({{ '/installation/' | relative_url }})** - How to install ldiff
- **[Usage Guide]({{ '/usage/' | relative_url }})** - General usage examples
- **[Scrap Guide]({{ '/scrap-guide/' | relative_url }})** - Manage temporary files
- **[Examples]({{ '/examples/' | relative_url }})** - More real-world examples