package main

import (
	"encoding/json"
	"fmt"

	"github.com/extism/go-pdk"
)

// ThoughtData represents a single thought in the sequential thinking process
type ThoughtData struct {
	Thought           string  `json:"thought"`
	ThoughtNumber     int     `json:"thoughtNumber"`
	TotalThoughts     int     `json:"totalThoughts"`
	NextThoughtNeeded bool    `json:"nextThoughtNeeded"`
	IsRevision        *bool   `json:"isRevision,omitempty"`
	RevisesThought    *int    `json:"revisesThought,omitempty"`
	BranchFromThought *int    `json:"branchFromThought,omitempty"`
	BranchId          *string `json:"branchId,omitempty"`
	NeedsMoreThoughts *bool   `json:"needsMoreThoughts,omitempty"`
}

// Global storage for thought history and branches
var thoughtHistory []ThoughtData
var branches map[string][]ThoughtData

func init() {
	thoughtHistory = make([]ThoughtData, 0)
	branches = make(map[string][]ThoughtData)
}

var (
	SequentialThinkingTool = ToolDescription{
		Name: "sequentialthinking",
		Description: `A detailed tool for dynamic and reflective problem-solving through thoughts.
This tool helps analyze problems through a flexible thinking process that can adapt and evolve.
Each thought can build on, question, or revise previous insights as understanding deepens.

When to use this tool:
- Breaking down complex problems into steps
- Planning and design with room for revision
- Analysis that might need course correction
- Problems where the full scope might not be clear initially
- Problems that require a multi-step solution
- Tasks that need to maintain context over multiple steps
- Situations where irrelevant information needs to be filtered out

Key features:
- You can adjust total_thoughts up or down as you progress
- You can question or revise previous thoughts
- You can add more thoughts even after reaching what seemed like the end
- You can express uncertainty and explore alternative approaches
- Not every thought needs to build linearly - you can branch or backtrack
- Generates a solution hypothesis
- Verifies the hypothesis based on the Chain of Thought steps
- Repeats the process until satisfied
- Provides a correct answer

Parameters explained:
- thought: Your current thinking step, which can include:
  * Regular analytical steps
  * Revisions of previous thoughts
  * Questions about previous decisions
  * Realizations about needing more analysis
  * Changes in approach
  * Hypothesis generation
  * Hypothesis verification
- next_thought_needed: True if you need more thinking, even if at what seemed like the end
- thought_number: Current number in sequence (can go beyond initial total if needed)
- total_thoughts: Current estimate of thoughts needed (can be adjusted up/down)
- is_revision: A boolean indicating if this thought revises previous thinking
- revises_thought: If is_revision is true, which thought number is being reconsidered
- branch_from_thought: If branching, which thought number is the branching point
- branch_id: Identifier for the current branch (if any)
- needs_more_thoughts: If reaching end but realizing more thoughts needed

You should:
1. Start with an initial estimate of needed thoughts, but be ready to adjust
2. Feel free to question or revise previous thoughts
3. Don't hesitate to add more thoughts if needed, even at the "end"
4. Express uncertainty when present
5. Mark thoughts that revise previous thinking or branch into new paths
6. Ignore information that is irrelevant to the current step
7. Generate a solution hypothesis when appropriate
8. Verify the hypothesis based on the Chain of Thought steps
9. Repeat the process until satisfied with the solution
10. Provide a single, ideally correct answer as the final output
11. Only set next_thought_needed to false when truly done and a satisfactory answer is reached`,
		InputSchema: schema{
			"type": "object",
			"properties": props{
				"thought":           prop("string", "Your current thinking step"),
				"nextThoughtNeeded": prop("boolean", "Whether another thought step is needed"),
				"thoughtNumber": SchemaProperty{
					Type:        "integer",
					Description: "Current thought number",
				},
				"totalThoughts": SchemaProperty{
					Type:        "integer",
					Description: "Estimated total thoughts needed",
				},
				"isRevision": prop("boolean", "Whether this revises previous thinking"),
				"revisesThought": SchemaProperty{
					Type:        "integer",
					Description: "Which thought is being reconsidered",
				},
				"branchFromThought": SchemaProperty{
					Type:        "integer",
					Description: "Branching point thought number",
				},
				"branchId":          prop("string", "Branch identifier"),
				"needsMoreThoughts": prop("boolean", "If more thoughts are needed"),
			},
			"required": []string{"thought", "nextThoughtNeeded", "thoughtNumber", "totalThoughts"},
		},
	}
)

var SequentialThinkingTools = []ToolDescription{
	SequentialThinkingTool,
}

// Called when the tool is invoked.
func Call(input CallToolRequest) (CallToolResult, error) {
	args := input.Params.Arguments.(map[string]interface{})
	pdk.Log(pdk.LogDebug, fmt.Sprint("Args: ", args))

	switch input.Params.Name {
	case SequentialThinkingTool.Name:
		return processThought(args)
	default:
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some("Unknown tool " + input.Params.Name),
			}},
		}, nil
	}
}

func Describe() (ListToolsResult, error) {
	tools := []ToolDescription{}
	tools = append(tools, SequentialThinkingTools...)

	// Ensure each tool's InputSchema has a required field
	for i := range tools {
		// Check if InputSchema is a map[string]interface{}
		if schema, ok := tools[i].InputSchema.(map[string]interface{}); ok {
			// Check if required field is missing
			if _, exists := schema["required"]; !exists {
				// Add an empty required array if it doesn't exist
				schema["required"] = []string{}
				tools[i].InputSchema = schema
			}
		}
	}

	return ListToolsResult{
		Tools: tools,
	}, nil
}

func processThought(args map[string]interface{}) (CallToolResult, error) {
	// Validate input
	thoughtData, err := validateThoughtData(args)
	if err != nil {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some(fmt.Sprintf("Error: %s", err.Error())),
			}},
		}, nil
	}

	// Adjust total thoughts if current thought number exceeds it
	if thoughtData.ThoughtNumber > thoughtData.TotalThoughts {
		thoughtData.TotalThoughts = thoughtData.ThoughtNumber
	}

	// Add to thought history
	thoughtHistory = append(thoughtHistory, thoughtData)

	// Handle branching
	if thoughtData.BranchFromThought != nil && thoughtData.BranchId != nil {
		if branches[*thoughtData.BranchId] == nil {
			branches[*thoughtData.BranchId] = make([]ThoughtData, 0)
		}
		branches[*thoughtData.BranchId] = append(branches[*thoughtData.BranchId], thoughtData)
	}

	// Format thought for display
	formattedThought := formatThought(thoughtData)

	// Log to stderr for visual feedback (like the TypeScript version)
	pdk.Log(pdk.LogInfo, formattedThought)

	// Prepare response data
	responseData := map[string]interface{}{
		"thoughtNumber":        thoughtData.ThoughtNumber,
		"totalThoughts":        thoughtData.TotalThoughts,
		"nextThoughtNeeded":    thoughtData.NextThoughtNeeded,
		"branches":             getBranchNames(),
		"thoughtHistoryLength": len(thoughtHistory),
	}

	responseJSON, err := json.MarshalIndent(responseData, "", "  ")
	if err != nil {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some(fmt.Sprintf("Failed to marshal response: %s", err.Error())),
			}},
		}, nil
	}

	return CallToolResult{
		Content: []Content{{
			Type: ContentTypeText,
			Text: some(string(responseJSON)),
		}},
	}, nil
}

func validateThoughtData(args map[string]interface{}) (ThoughtData, error) {
	data := ThoughtData{}

	// Required fields
	thought, ok := args["thought"].(string)
	if !ok || thought == "" {
		return data, fmt.Errorf("invalid thought: must be a string")
	}
	data.Thought = thought

	thoughtNumber, ok := args["thoughtNumber"].(float64)
	if !ok {
		return data, fmt.Errorf("invalid thoughtNumber: must be a number")
	}
	data.ThoughtNumber = int(thoughtNumber)

	totalThoughts, ok := args["totalThoughts"].(float64)
	if !ok {
		return data, fmt.Errorf("invalid totalThoughts: must be a number")
	}
	data.TotalThoughts = int(totalThoughts)

	nextThoughtNeeded, ok := args["nextThoughtNeeded"].(bool)
	if !ok {
		return data, fmt.Errorf("invalid nextThoughtNeeded: must be a boolean")
	}
	data.NextThoughtNeeded = nextThoughtNeeded

	// Optional fields
	if isRevision, ok := args["isRevision"].(bool); ok {
		data.IsRevision = &isRevision
	}

	if revisesThought, ok := args["revisesThought"].(float64); ok {
		revises := int(revisesThought)
		data.RevisesThought = &revises
	}

	if branchFromThought, ok := args["branchFromThought"].(float64); ok {
		branch := int(branchFromThought)
		data.BranchFromThought = &branch
	}

	if branchId, ok := args["branchId"].(string); ok {
		data.BranchId = &branchId
	}

	if needsMoreThoughts, ok := args["needsMoreThoughts"].(bool); ok {
		data.NeedsMoreThoughts = &needsMoreThoughts
	}

	return data, nil
}

func formatThought(thoughtData ThoughtData) string {
	prefix := ""
	context := ""

	if thoughtData.IsRevision != nil && *thoughtData.IsRevision {
		prefix = "🔄 Revision"
		if thoughtData.RevisesThought != nil {
			context = fmt.Sprintf(" (revising thought %d)", *thoughtData.RevisesThought)
		}
	} else if thoughtData.BranchFromThought != nil {
		prefix = "🌿 Branch"
		branchId := ""
		if thoughtData.BranchId != nil {
			branchId = *thoughtData.BranchId
		}
		context = fmt.Sprintf(" (from thought %d, ID: %s)", *thoughtData.BranchFromThought, branchId)
	} else {
		prefix = "💭 Thought"
		context = ""
	}

	header := fmt.Sprintf("%s %d/%d%s", prefix, thoughtData.ThoughtNumber, thoughtData.TotalThoughts, context)

	// Create a simple text-based format since we can't use advanced formatting in WASM
	return fmt.Sprintf("\n=== %s ===\n%s\n================", header, thoughtData.Thought)
}

func getBranchNames() []string {
	names := make([]string, 0, len(branches))
	for name := range branches {
		names = append(names, name)
	}
	return names
}

func some[T any](t T) *T {
	return &t
}

type SchemaProperty struct {
	Type                 string  `json:"type"`
	Description          string  `json:"description,omitempty"`
	AdditionalProperties *schema `json:"additionalProperties,omitempty"`
	Items                *schema `json:"items,omitempty"`
}

func prop(tpe, description string) SchemaProperty {
	return SchemaProperty{Type: tpe, Description: description}
}

func arrprop(tpe, description, itemstpe string) SchemaProperty {
	items := schema{"type": itemstpe}
	return SchemaProperty{Type: tpe, Description: description, Items: &items}
}

type schema = map[string]interface{}
type props = map[string]SchemaProperty
