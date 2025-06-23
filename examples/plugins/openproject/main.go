package main

import (
	"encoding/base64"
	"encoding/json"
	"fmt"

	pdk "github.com/extism/go-pdk"
)

// OpenProject API client structure
type OpenProjectClient struct {
	BaseURL    string
	APIKey     string
	APIVersion string
}

// Global client instance
var client *OpenProjectClient

// Initialize the OpenProject client
func initOpenProjectClient() error {
	apiKey, ok := pdk.GetConfig("api-key")
	if !ok {
		return fmt.Errorf("no api-key configured")
	}

	baseURL, ok := pdk.GetConfig("base-url")
	if !ok {
		return fmt.Errorf("no base-url configured")
	}

	apiVersion, ok := pdk.GetConfig("api-version")
	if !ok {
		apiVersion = "v3" // Default API version
	}

	client = &OpenProjectClient{
		BaseURL:    baseURL,
		APIKey:     apiKey,
		APIVersion: apiVersion,
	}

	return nil
}

// Make an HTTP request to the OpenProject API
func (c *OpenProjectClient) Request(method, path string, body interface{}) ([]byte, error) {
	url := fmt.Sprintf("%s/api/%s%s", c.BaseURL, c.APIVersion, path)

	// Convert string method to pdk.HTTPMethod
	var httpMethod pdk.HTTPMethod
	switch method {
	case "GET":
		httpMethod = pdk.MethodGet
	case "POST":
		httpMethod = pdk.MethodPost
	case "PUT":
		httpMethod = pdk.MethodPut
	case "PATCH":
		httpMethod = pdk.MethodPatch
	case "DELETE":
		httpMethod = pdk.MethodDelete
	default:
		return nil, fmt.Errorf("unsupported HTTP method: %s", method)
	}

	req := pdk.NewHTTPRequest(httpMethod, url)

	// Add authorization header
	auth := fmt.Sprintf("Basic %s", base64.StdEncoding.EncodeToString([]byte(fmt.Sprintf("apikey:%s", c.APIKey))))
	req.SetHeader("Authorization", auth)
	req.SetHeader("Content-Type", "application/json")

	// Add body if provided
	if body != nil {
		bodyBytes, err := json.Marshal(body)
		if err != nil {
			return nil, fmt.Errorf("failed to marshal request body: %v", err)
		}
		req.SetBody(bodyBytes)
	}

	resp := req.Send()

	// Check for HTTP errors
	if resp.Status() >= 400 {
		return nil, fmt.Errorf("HTTP error: %d - %s", resp.Status(), string(resp.Body()))
	}

	return resp.Body(), nil
}

// Called when the tool is invoked
func Call(input CallToolRequest) (CallToolResult, error) {
	// Initialize client if not already done
	if client == nil {
		if err := initOpenProjectClient(); err != nil {
			return CallToolResult{
				IsError: some(true),
				Content: []Content{{
					Type: ContentTypeText,
					Text: some(fmt.Sprintf("Failed to initialize OpenProject client: %s", err.Error())),
				}},
			}, nil
		}
	}

	args := input.Params.Arguments.(map[string]interface{})
	pdk.Log(pdk.LogDebug, fmt.Sprintf("Tool: %s, Args: %+v", input.Params.Name, args))

	switch input.Params.Name {
	case CreateProjectTool.Name:
		return createProject(args)
	case GetProjectTool.Name:
		return getProject(args)
	case ListProjectsTool.Name:
		return listProjects(args)
	case UpdateProjectTool.Name:
		return updateProject(args)
	case DeleteProjectTool.Name:
		return deleteProject(args)
	case CreateTaskTool.Name:
		return createTask(args)
	case GetTaskTool.Name:
		return getTask(args)
	case ListTasksTool.Name:
		return listTasks(args)
	case UpdateTaskTool.Name:
		return updateTask(args)
	case DeleteTaskTool.Name:
		return deleteTask(args)
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

// List available tools
func Describe() (ListToolsResult, error) {
	tools := []ToolDescription{
		CreateProjectTool,
		GetProjectTool,
		ListProjectsTool,
		UpdateProjectTool,
		DeleteProjectTool,
		CreateTaskTool,
		GetTaskTool,
		ListTasksTool,
		UpdateTaskTool,
		DeleteTaskTool,
	}

	return ListToolsResult{
		Tools: tools,
	}, nil
}

// Helper function to create a pointer to a value
func some[T any](t T) *T {
	return &t
}

// Helper function for schema properties
type SchemaProperty struct {
	Type        string  `json:"type"`
	Description string  `json:"description,omitempty"`
	Default     *string `json:"default,omitempty"`
	Items       *schema `json:"items,omitempty"`
}

func prop(tpe, description string) SchemaProperty {
	return SchemaProperty{Type: tpe, Description: description}
}

func propWithDefault(tpe, description, defaultValue string) SchemaProperty {
	return SchemaProperty{Type: tpe, Description: description, Default: &defaultValue}
}

func arrprop(tpe, description, itemstpe string) SchemaProperty {
	items := schema{"type": itemstpe}
	return SchemaProperty{Type: tpe, Description: description, Items: &items}
}

type schema = map[string]interface{}
type props = map[string]SchemaProperty
