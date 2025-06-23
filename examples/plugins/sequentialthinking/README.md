# Sequential Thinking

A Model Context Protocol (MCP) plugin that implements sequential thinking for dynamic and reflective problem-solving. This plugin is based on the TypeScript sequential thinking server and adapted for the Go WASM plugin architecture.

## Description

This tool helps analyze problems through a flexible thinking process that can adapt and evolve. Each thought can build on, question, or revise previous insights as understanding deepens.

## Features

- **Dynamic thinking process**: Adjust the number of thoughts as you progress
- **Revision capability**: Question or revise previous thoughts
- **Branching logic**: Explore alternative approaches
- **Context maintenance**: Keep track of the entire thinking process
- **Hypothesis generation and verification**: Build and test solution hypotheses

## When to Use

- Breaking down complex problems into steps
- Planning and design with room for revision
- Analysis that might need course correction
- Problems where the full scope might not be clear initially
- Multi-step solutions requiring context maintenance
- Filtering out irrelevant information

## Usage

```json
{
    "plugins": [
        {
            "name": "sequentialthinking",
            "path": "oci://ghcr.io/sevir/sequentialthinking-plugin:latest"
        }
    ]
}
```

## Tool Parameters

### Required Parameters

- `thought` (string): Your current thinking step
- `nextThoughtNeeded` (boolean): Whether another thought step is needed
- `thoughtNumber` (integer): Current thought number in sequence
- `totalThoughts` (integer): Estimated total thoughts needed

### Optional Parameters

- `isRevision` (boolean): Whether this thought revises previous thinking
- `revisesThought` (integer): Which thought number is being reconsidered
- `branchFromThought` (integer): Branching point thought number
- `branchId` (string): Identifier for the current branch
- `needsMoreThoughts` (boolean): If more thoughts are needed at the end

## Example Usage

```json
{
    "thought": "Let me break down this problem into smaller components...",
    "thoughtNumber": 1,
    "totalThoughts": 5,
    "nextThoughtNeeded": true
}
```

## Building

```bash
# Build the WASM plugin
docker build -t sequentialthinking-plugin .

# Or build locally with tinygo
tinygo build -target wasi -o plugin.wasm .
```

## Implementation Notes

This plugin maintains internal state for:
- Thought history: All thoughts in chronological order
- Branches: Alternative thinking paths identified by branch IDs
- Visual formatting: Formatted output with emojis and structure

The plugin follows the sequential thinking methodology allowing for:
1. Linear progression through thoughts
2. Revision of previous thoughts when new insights emerge
3. Branching into alternative approaches
4. Dynamic adjustment of the total thought count
5. Verification and hypothesis testing
