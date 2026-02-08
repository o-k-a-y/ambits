# Session Management

## How Claude Code Stores Sessions

Claude Code stores session logs as JSONL files in a projects directory:

```
~/.claude/projects/<project-slug>/<session-id>.jsonl
```

The **project slug** is derived from the absolute project path with `/` replaced by `-`. For example:
- `/Users/joshlong/Documents/code/ambit` becomes `-Users-joshlong-Documents-code-ambit`

Sub-agent sessions are stored as `agent-<hash>.jsonl` in the same directory.

## Listing Available Sessions

```bash
# List all sessions for a project (most recent first)
ls -lt ~/.claude/projects/<project-slug>/*.jsonl | head -10

# Sessions modified in last 7 days
find ~/.claude/projects/<project-slug> -name "*.jsonl" -mtime -7
```

## Session ID Extraction

The session ID is the filename without the `.jsonl` extension:

```
34e212cf-a176-4059-ba12-eca94b56e43b.jsonl
# Session ID: 34e212cf-a176-4059-ba12-eca94b56e43b
```

Use it with:
```bash
ambits -p . --coverage --session 34e212cf-a176-4059-ba12-eca94b56e43b
```

## Auto-Detection

When no `--session` flag is provided, ambits automatically detects the most recently modified session file for the project. This is usually the current or most recent Claude Code session.

## Session Index

Claude Code also maintains a `sessions-index.json` file in the projects directory. The JSONL log files contain message types: `"user"`, `"assistant"` (or `"A"`), `"progress"`, and `"queue-operation"`. Tool calls appear in assistant messages as `message.content[].type == "tool_use"`.
