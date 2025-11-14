PATTERNS_DIR := "/home/darko/workspace/kiro-projects/better-agent/pattern-library/patterns"

# Test the MCP server with the MCP inspector
mcp-test:
  npx @modelcontextprotocol/inspector -e PATTERNS_DIR={{PATTERNS_DIR}} ./target/release/dont-forget-your-code

# Debug the project
debug:
  PATTERNS_DIR={{PATTERNS_DIR}} RUST_LOG=DEBUG cargo run
