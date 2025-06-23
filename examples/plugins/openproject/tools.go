package main

import (
	"encoding/json"
	"fmt"
)

// Project tool definitions
var (
	CreateProjectTool = ToolDescription{
		Name:        "openproject-create-project",
		Description: "Creates a new project in OpenProject",
		InputSchema: schema{
			"type": "object",
			"properties": props{
				"name":        prop("string", "Name of the project"),
				"identifier":  prop("string", "Identifier of the project (unique)"),
				"description": prop("string", "Optional description for the project"),
			},
			"required": []string{"name", "identifier"},
		},
	}

	GetProjectTool = ToolDescription{
		Name:        "openproject-get-project",
		Description: "Gets a specific project by its ID from OpenProject",
		InputSchema: schema{
			"type": "object",
			"properties": props{
				"projectId": prop("string", "The ID of the project to retrieve"),
			},
			"required": []string{"projectId"},
		},
	}

	ListProjectsTool = ToolDescription{
		Name:        "openproject-list-projects",
		Description: "Lists all projects in OpenProject",
		InputSchema: schema{
			"type": "object",
			"properties": props{
				"pageSize": prop("integer", "Number of projects per page"),
				"offset":   prop("integer", "Page number to retrieve (1-indexed)"),
			},
			"required": []string{},
		},
	}

	UpdateProjectTool = ToolDescription{
		Name:        "openproject-update-project",
		Description: "Updates an existing project in OpenProject. Only include fields to be changed.",
		InputSchema: schema{
			"type": "object",
			"properties": props{
				"projectId":   prop("string", "The ID of the project to update"),
				"name":        prop("string", "New name for the project"),
				"description": prop("string", "New description for the project"),
			},
			"required": []string{"projectId"},
		},
	}

	DeleteProjectTool = ToolDescription{
		Name:        "openproject-delete-project",
		Description: "Deletes a project from OpenProject. This action is irreversible.",
		InputSchema: schema{
			"type": "object",
			"properties": props{
				"projectId": prop("string", "The ID of the project to delete"),
			},
			"required": []string{"projectId"},
		},
	}
)

// Task (work package) tool definitions
var (
	CreateTaskTool = ToolDescription{
		Name:        "openproject-create-task",
		Description: "Creates a new task (work package) in an OpenProject project",
		InputSchema: schema{
			"type": "object",
			"properties": props{
				"projectId":   prop("string", "The ID or identifier of the project to add the task to"),
				"subject":     prop("string", "Subject/title of the task"),
				"description": prop("string", "Optional description for the task"),
				"type":        propWithDefault("string", "Type of the work package (e.g., /api/v3/types/1 for Task)", "/api/v3/types/1"),
			},
			"required": []string{"projectId", "subject"},
		},
	}

	GetTaskTool = ToolDescription{
		Name:        "openproject-get-task",
		Description: "Gets a specific task (work package) by its ID from OpenProject",
		InputSchema: schema{
			"type": "object",
			"properties": props{
				"taskId": prop("string", "The ID of the task to retrieve"),
			},
			"required": []string{"taskId"},
		},
	}

	ListTasksTool = ToolDescription{
		Name:        "openproject-list-tasks",
		Description: "Lists tasks (work packages) in OpenProject, optionally filtered by project ID",
		InputSchema: schema{
			"type": "object",
			"properties": props{
				"projectId": prop("string", "Optional ID of the project to filter tasks by"),
				"pageSize":  prop("integer", "Number of tasks per page"),
				"offset":    prop("integer", "Page number to retrieve (1-indexed)"),
			},
			"required": []string{},
		},
	}

	UpdateTaskTool = ToolDescription{
		Name:        "openproject-update-task",
		Description: "Updates an existing task (work package) in OpenProject. Only include fields to be changed.",
		InputSchema: schema{
			"type": "object",
			"properties": props{
				"taskId":      prop("string", "The ID of the task to update"),
				"lockVersion": prop("integer", "The lockVersion of the task (obtained from a GET request)"),
				"subject":     prop("string", "New subject/title for the task"),
				"description": prop("string", "New description for the task (provide as raw text)"),
			},
			"required": []string{"taskId", "lockVersion"},
		},
	}

	DeleteTaskTool = ToolDescription{
		Name:        "openproject-delete-task",
		Description: "Deletes a task (work package) from OpenProject. This action is irreversible.",
		InputSchema: schema{
			"type": "object",
			"properties": props{
				"taskId": prop("string", "The ID of the task to delete"),
			},
			"required": []string{"taskId"},
		},
	}
)

// Implementation of OpenProject API operations

// Project operations
func createProject(args map[string]interface{}) (CallToolResult, error) {
	name, _ := args["name"].(string)
	identifier, _ := args["identifier"].(string)
	description, _ := args["description"].(string)

	requestBody := map[string]interface{}{
		"name":        name,
		"identifier":  identifier,
		"description": description,
	}

	responseBytes, err := client.Request("POST", "/projects", requestBody)
	if err != nil {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some(fmt.Sprintf("Error creating project: %s", err.Error())),
			}},
		}, nil
	}

	var responseData map[string]interface{}
	if err := json.Unmarshal(responseBytes, &responseData); err != nil {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some(fmt.Sprintf("Error parsing response: %s", err.Error())),
			}},
		}, nil
	}

	successText := fmt.Sprintf("Successfully created project: %s (ID: %v)",
		responseData["name"], responseData["id"])
	responseJSON := string(responseBytes)

	return CallToolResult{
		Content: []Content{
			{
				Type: ContentTypeText,
				Text: some(successText),
			},
			{
				Type: ContentTypeText,
				Text: some(responseJSON),
			},
		},
	}, nil
}

func getProject(args map[string]interface{}) (CallToolResult, error) {
	projectId, _ := args["projectId"].(string)

	responseBytes, err := client.Request("GET", fmt.Sprintf("/projects/%s", projectId), nil)
	if err != nil {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some(fmt.Sprintf("Error getting project: %s", err.Error())),
			}},
		}, nil
	}

	var responseData map[string]interface{}
	if err := json.Unmarshal(responseBytes, &responseData); err != nil {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some(fmt.Sprintf("Error parsing response: %s", err.Error())),
			}},
		}, nil
	}

	successText := fmt.Sprintf("Successfully retrieved project: %s", responseData["name"])
	responseJSON := string(responseBytes)

	return CallToolResult{
		Content: []Content{
			{
				Type: ContentTypeText,
				Text: some(successText),
			},
			{
				Type: ContentTypeText,
				Text: some(responseJSON),
			},
		},
	}, nil
}

func listProjects(args map[string]interface{}) (CallToolResult, error) {
	// Build query parameters
	params := ""
	if pageSize, ok := args["pageSize"].(float64); ok {
		params += fmt.Sprintf("pageSize=%d&", int(pageSize))
	}
	if offset, ok := args["offset"].(float64); ok {
		params += fmt.Sprintf("offset=%d&", int(offset))
	}

	// Add query parameters to URL if present
	url := "/projects"
	if params != "" {
		url += "?" + params[:len(params)-1] // Remove trailing &
	}

	responseBytes, err := client.Request("GET", url, nil)
	if err != nil {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some(fmt.Sprintf("Error listing projects: %s", err.Error())),
			}},
		}, nil
	}

	var responseData map[string]interface{}
	if err := json.Unmarshal(responseBytes, &responseData); err != nil {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some(fmt.Sprintf("Error parsing response: %s", err.Error())),
			}},
		}, nil
	}

	// Extract embedded elements to count projects
	embedded, ok := responseData["_embedded"].(map[string]interface{})
	if !ok {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some("Error: Unexpected response format from OpenProject API"),
			}},
		}, nil
	}

	elements, ok := embedded["elements"].([]interface{})
	if !ok {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some("Error: Unable to extract projects from response"),
			}},
		}, nil
	}

	total := responseData["total"]
	successText := fmt.Sprintf("Successfully retrieved %d projects (Total: %v)", len(elements), total)
	responseJSON := string(responseBytes)

	return CallToolResult{
		Content: []Content{
			{
				Type: ContentTypeText,
				Text: some(successText),
			},
			{
				Type: ContentTypeText,
				Text: some(responseJSON),
			},
		},
	}, nil
}

func updateProject(args map[string]interface{}) (CallToolResult, error) {
	projectId, _ := args["projectId"].(string)

	// Create update payload
	updatePayload := make(map[string]interface{})

	// Add optional fields if present
	if name, ok := args["name"].(string); ok {
		updatePayload["name"] = name
	}
	if description, ok := args["description"].(string); ok {
		updatePayload["description"] = description
	}

	// Check if any fields are provided to update
	if len(updatePayload) == 0 {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some("Error: No fields provided to update for the project"),
			}},
		}, nil
	}

	responseBytes, err := client.Request("PATCH", fmt.Sprintf("/projects/%s", projectId), updatePayload)
	if err != nil {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some(fmt.Sprintf("Error updating project: %s", err.Error())),
			}},
		}, nil
	}

	var responseData map[string]interface{}
	if err := json.Unmarshal(responseBytes, &responseData); err != nil {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some(fmt.Sprintf("Error parsing response: %s", err.Error())),
			}},
		}, nil
	}

	successText := fmt.Sprintf("Successfully updated project: %s", responseData["name"])
	responseJSON := string(responseBytes)

	return CallToolResult{
		Content: []Content{
			{
				Type: ContentTypeText,
				Text: some(successText),
			},
			{
				Type: ContentTypeText,
				Text: some(responseJSON),
			},
		},
	}, nil
}

func deleteProject(args map[string]interface{}) (CallToolResult, error) {
	projectId, _ := args["projectId"].(string)

	_, err := client.Request("DELETE", fmt.Sprintf("/projects/%s", projectId), nil)
	if err != nil {
		// Check if the error is a 404 (project not found)
		if err.Error() == "HTTP error: 404" {
			return CallToolResult{
				Content: []Content{{
					Type: ContentTypeText,
					Text: some(fmt.Sprintf("Project with ID %s not found. It might have already been deleted.", projectId)),
				}},
			}, nil
		}

		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some(fmt.Sprintf("Error deleting project: %s", err.Error())),
			}},
		}, nil
	}

	return CallToolResult{
		Content: []Content{{
			Type: ContentTypeText,
			Text: some(fmt.Sprintf("Successfully deleted project with ID: %s", projectId)),
		}},
	}, nil
}

// Task operations
func createTask(args map[string]interface{}) (CallToolResult, error) {
	projectId, _ := args["projectId"].(string)
	subject, _ := args["subject"].(string)
	description, _ := args["description"].(string)
	typeHref, _ := args["type"].(string)

	// Prepare request body
	requestBody := map[string]interface{}{
		"subject": subject,
		"_links": map[string]interface{}{
			"project": map[string]interface{}{
				"href": fmt.Sprintf("/api/%s/projects/%s", client.APIVersion, projectId),
			},
			"type": map[string]interface{}{
				"href": typeHref,
			},
		},
	}

	// Add description if provided
	if description != "" {
		requestBody["description"] = map[string]interface{}{
			"raw": description,
		}
	}

	responseBytes, err := client.Request("POST", fmt.Sprintf("/projects/%s/work_packages", projectId), requestBody)
	if err != nil {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some(fmt.Sprintf("Error creating task: %s", err.Error())),
			}},
		}, nil
	}

	var responseData map[string]interface{}
	if err := json.Unmarshal(responseBytes, &responseData); err != nil {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some(fmt.Sprintf("Error parsing response: %s", err.Error())),
			}},
		}, nil
	}

	successText := fmt.Sprintf("Successfully created task: %s (ID: %v) in project %s",
		responseData["subject"], responseData["id"], projectId)
	responseJSON := string(responseBytes)

	return CallToolResult{
		Content: []Content{
			{
				Type: ContentTypeText,
				Text: some(successText),
			},
			{
				Type: ContentTypeText,
				Text: some(responseJSON),
			},
		},
	}, nil
}

func getTask(args map[string]interface{}) (CallToolResult, error) {
	taskId, _ := args["taskId"].(string)

	responseBytes, err := client.Request("GET", fmt.Sprintf("/work_packages/%s", taskId), nil)
	if err != nil {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some(fmt.Sprintf("Error getting task: %s", err.Error())),
			}},
		}, nil
	}

	var responseData map[string]interface{}
	if err := json.Unmarshal(responseBytes, &responseData); err != nil {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some(fmt.Sprintf("Error parsing response: %s", err.Error())),
			}},
		}, nil
	}

	successText := fmt.Sprintf("Successfully retrieved task: %s", responseData["subject"])
	responseJSON := string(responseBytes)

	return CallToolResult{
		Content: []Content{
			{
				Type: ContentTypeText,
				Text: some(successText),
			},
			{
				Type: ContentTypeText,
				Text: some(responseJSON),
			},
		},
	}, nil
}

func listTasks(args map[string]interface{}) (CallToolResult, error) {
	// Build query parameters
	params := ""
	if pageSize, ok := args["pageSize"].(float64); ok {
		params += fmt.Sprintf("pageSize=%d&", int(pageSize))
	}
	if offset, ok := args["offset"].(float64); ok {
		params += fmt.Sprintf("offset=%d&", int(offset))
	}

	// Determine URL based on whether projectId is provided
	var url string
	if projectId, ok := args["projectId"].(string); ok && projectId != "" {
		url = fmt.Sprintf("/projects/%s/work_packages", projectId)
	} else {
		url = "/work_packages"
	}

	// Add query parameters to URL if present
	if params != "" {
		url += "?" + params[:len(params)-1] // Remove trailing &
	}

	responseBytes, err := client.Request("GET", url, nil)
	if err != nil {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some(fmt.Sprintf("Error listing tasks: %s", err.Error())),
			}},
		}, nil
	}

	var responseData map[string]interface{}
	if err := json.Unmarshal(responseBytes, &responseData); err != nil {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some(fmt.Sprintf("Error parsing response: %s", err.Error())),
			}},
		}, nil
	}

	// Extract embedded elements to count tasks
	embedded, ok := responseData["_embedded"].(map[string]interface{})
	if !ok {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some("Error: Unexpected response format from OpenProject API"),
			}},
		}, nil
	}

	elements, ok := embedded["elements"].([]interface{})
	if !ok {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some("Error: Unable to extract tasks from response"),
			}},
		}, nil
	}

	total := responseData["total"]
	successText := fmt.Sprintf("Successfully retrieved %d tasks (Total: %v)", len(elements), total)
	responseJSON := string(responseBytes)

	return CallToolResult{
		Content: []Content{
			{
				Type: ContentTypeText,
				Text: some(successText),
			},
			{
				Type: ContentTypeText,
				Text: some(responseJSON),
			},
		},
	}, nil
}

func updateTask(args map[string]interface{}) (CallToolResult, error) {
	taskId, _ := args["taskId"].(string)
	lockVersion, ok := args["lockVersion"].(float64)
	if !ok {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some("Error: lockVersion is required to update a task"),
			}},
		}, nil
	}

	// Create update payload
	updatePayload := map[string]interface{}{
		"lockVersion": int(lockVersion),
	}

	// Add optional fields if present
	if subject, ok := args["subject"].(string); ok {
		updatePayload["subject"] = subject
	}
	if description, ok := args["description"].(string); ok {
		updatePayload["description"] = map[string]interface{}{
			"raw": description,
		}
	}

	// Check if any fields are provided to update
	if len(updatePayload) <= 1 { // <= 1 because lockVersion doesn't count as a field to update
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some("Error: No fields (besides lockVersion) provided to update for the task"),
			}},
		}, nil
	}

	responseBytes, err := client.Request("PATCH", fmt.Sprintf("/work_packages/%s", taskId), updatePayload)
	if err != nil {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some(fmt.Sprintf("Error updating task: %s", err.Error())),
			}},
		}, nil
	}

	var responseData map[string]interface{}
	if err := json.Unmarshal(responseBytes, &responseData); err != nil {
		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some(fmt.Sprintf("Error parsing response: %s", err.Error())),
			}},
		}, nil
	}

	successText := fmt.Sprintf("Successfully updated task: %s", responseData["subject"])
	responseJSON := string(responseBytes)

	return CallToolResult{
		Content: []Content{
			{
				Type: ContentTypeText,
				Text: some(successText),
			},
			{
				Type: ContentTypeText,
				Text: some(responseJSON),
			},
		},
	}, nil
}

func deleteTask(args map[string]interface{}) (CallToolResult, error) {
	taskId, _ := args["taskId"].(string)

	_, err := client.Request("DELETE", fmt.Sprintf("/work_packages/%s", taskId), nil)
	if err != nil {
		// Check if the error is a 404 (task not found)
		if err.Error() == "HTTP error: 404" {
			return CallToolResult{
				Content: []Content{{
					Type: ContentTypeText,
					Text: some(fmt.Sprintf("Task with ID %s not found. It might have already been deleted.", taskId)),
				}},
			}, nil
		}

		return CallToolResult{
			IsError: some(true),
			Content: []Content{{
				Type: ContentTypeText,
				Text: some(fmt.Sprintf("Error deleting task: %s", err.Error())),
			}},
		}, nil
	}

	return CallToolResult{
		Content: []Content{{
			Type: ContentTypeText,
			Text: some(fmt.Sprintf("Successfully deleted task with ID: %s", taskId)),
		}},
	}, nil
}
