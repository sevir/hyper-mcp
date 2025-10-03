# Holded API Plugin

A hyper-mcp plugin that provides comprehensive access to the Holded API for managing invoices, employees, contacts, and calendar events.

## Overview

This plugin allows you to interact with Holded's business management platform, providing tools to query invoices, manage contacts, view employee calendars, and access various document types. It includes intelligent name-based search with similarity matching for easy lookup without needing exact IDs.

## Features

- **Contact Management**: Search contacts by name with fuzzy matching
- **Invoice Operations**: Query invoices by company name, date ranges, and status
- **Employee Management**: List employees and access their calendar events
- **Calendar Integration**: View individual or all employee calendars
- **Document Access**: Retrieve estimates, orders, purchase orders, and delivery notes
- **Smart Search**: Similarity-based name matching for contacts and employees

## Prerequisites

Before using this plugin, you need to:

1. **Create a Holded Account**: Go to [Holded](https://www.holded.com/)
2. **Get API Access**: Visit your Holded account settings
3. **Generate API Key**: Create and copy your API key from the API section
4. **Configure Permissions**: Ensure your API key has appropriate permissions for the operations you need

## Configuration

The plugin requires the following configuration:

- `HOLDED_API_KEY`: (Required) Your Holded API key

## Usage

```json
{
  "plugins": [
    {
      "name": "holded",
      "path": "oci://ghcr.io/sevir/hyper-mcp/plugin-holded:latest",
      "runtime_config": {
        "allowed_hosts": ["api.holded.com"],
        "env_vars": {
          "HOLDED_API_KEY": "your-holded-api-key-here"
        }
      }
    }
  ]
}
```

## Available Operations

### Contact Operations

#### Search Contact by Name

```json
{
  "name": "search_contact_by_name",
  "arguments": {
    "name": "Acme Corporation",
    "threshold": 0.6
  }
}
```

Searches for a contact using similarity matching. The threshold parameter (0-1) controls how strict the matching is.

#### List Contacts with Invoices

```json
{
  "name": "list_contacts_with_invoices",
  "arguments": {
    "date_from": "2024-01-01",
    "date_to": "2024-12-31",
    "status": "paid"
  }
}
```

Lists all contacts that have invoices in a specific period with summary statistics.

### Invoice Operations

#### Get Invoices by Company Name

```json
{
  "name": "get_invoices_by_company_name",
  "arguments": {
    "company_name": "Acme Corp",
    "date_from": "2024-01-01",
    "date_to": "2024-12-31",
    "status": "all"
  }
}
```

Retrieves invoices for a company by searching for the contact name (no exact ID needed).

#### Get Invoices

```json
{
  "name": "get_invoices",
  "arguments": {
    "status": "unpaid",
    "date_from": "2024-01-01",
    "date_to": "2024-12-31",
    "limit": 50
  }
}
```

Lists invoices with various filters.

#### Get Invoice Detail

```json
{
  "name": "get_invoice_detail",
  "arguments": {
    "invoice_id": "60f7b1a2e4b0c8f3d4e5f6a7"
  }
}
```

Retrieves complete details of a specific invoice.

### Employee Operations

#### List Employees

```json
{
  "name": "list_employees",
  "arguments": {
    "active": true,
    "department": "Sales"
  }
}
```

Lists all employees with filtering options.

#### Get Employee Calendar by Name

```json
{
  "name": "get_employee_calendar_by_name",
  "arguments": {
    "employee_name": "John Doe",
    "date_from": "2024-01-01",
    "date_to": "2024-12-31",
    "type": "vacation"
  }
}
```

Retrieves calendar events for an employee by searching their name.

#### Get All Employees Calendar

```json
{
  "name": "get_all_employees_calendar",
  "arguments": {
    "date_from": "2024-01-01",
    "date_to": "2024-12-31",
    "type": "all"
  }
}
```

Retrieves calendar events for all employees in the specified period.

### Document Operations

#### Get Documents

```json
{
  "name": "get_documents",
  "arguments": {
    "doc_type": "estimate",
    "limit": 25
  }
}
```

Retrieves documents of specified type (invoice, estimate, order, purchaseorder, proforma, deliverynote).

## Parameters Reference

### Contact Tools

- **search_contact_by_name**
  - `name` (string, required): Company or contact name to search
  - `threshold` (number, optional): Similarity threshold (0-1, default: 0.6)

- **list_contacts_with_invoices**
  - `date_from` (string, optional): Start date (YYYY-MM-DD)
  - `date_to` (string, optional): End date (YYYY-MM-DD)
  - `status` (string, optional): Invoice status filter

### Invoice Tools

- **get_invoices_by_company_name**
  - `company_name` (string, required): Company name to search
  - `date_from` (string, optional): Start date (YYYY-MM-DD)
  - `date_to` (string, optional): End date (YYYY-MM-DD)
  - `status` (string, optional): Invoice status (draft, sent, paid, partial, unpaid, all)

- **get_invoices**
  - `status` (string, optional): Invoice status filter
  - `date_from` (string, optional): Start date (YYYY-MM-DD)
  - `date_to` (string, optional): End date (YYYY-MM-DD)
  - `limit` (integer, optional): Maximum results (default: 100)

- **get_invoice_detail**
  - `invoice_id` (string, required): Invoice ID

### Employee Tools

- **list_employees**
  - `active` (boolean, optional): Filter active employees only
  - `department` (string, optional): Filter by department

- **get_employee_calendar_by_name**
  - `employee_name` (string, required): Employee name to search
  - `date_from` (string, optional): Start date (YYYY-MM-DD)
  - `date_to` (string, optional): End date (YYYY-MM-DD)
  - `type` (string, optional): Event type (absence, vacation, sick_leave, holiday, all)

- **get_all_employees_calendar**
  - `date_from` (string, optional): Start date (YYYY-MM-DD)
  - `date_to` (string, optional): End date (YYYY-MM-DD)
  - `type` (string, optional): Event type filter

### Document Tools

- **get_documents**
  - `doc_type` (string, required): Document type (invoice, estimate, order, purchaseorder, proforma, deliverynote)
  - `limit` (integer, optional): Maximum results (default: 50)

## Error Handling

The plugin provides descriptive error messages for:
- Missing or invalid API keys
- Invalid parameters
- HTTP request failures
- JSON parsing errors
- Contact/employee not found

## Limitations

- API rate limits apply as per your Holded plan
- Date formats must be in YYYY-MM-DD format
- Similarity matching threshold of 0.6 is recommended for best results
- Maximum 20 results displayed in some list operations (configurable)

## Development

### Building the Plugin

```bash
cd examples/plugins/holded
cargo build --release --target wasm32-wasip1
```

### Building with Docker

```bash
docker build -t ghcr.io/sevir/hyper-mcp/plugin-holded:latest .
```

### Publishing to GitHub Container Registry

```bash
# Login to GitHub Container Registry
echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin

# Build for the target architecture
docker build -t ghcr.io/sevir/hyper-mcp/plugin-holded:latest .

# Push the image
docker push ghcr.io/sevir/hyper-mcp/plugin-holded:latest
```

## API Reference

For more information about the Holded API:
- [Holded API Documentation](https://www.holded.com/help/api)
- [API Endpoints Reference](https://api.holded.com/api)

## License

This plugin is part of the hyper-mcp project and follows the same license.

## Support

For issues or questions:
- Open an issue on the [hyper-mcp repository](https://github.com/sevir/hyper-mcp)
- Check the [Holded API documentation](https://www.holded.com/help/api)

## Tasks

### build:images

Build docker images of plugins

interactive: true

```bash
docker build -t ghcr.io/sevir/hyper-mcp/plugin-holded:latest .
```

### push:images

Push docker images of plugins

interactive: true

```bash

docker push ghcr.io/sevir/hyper-mcp/plugin-holded:latest
cosign sign --yes ghcr.io/sevir/hyper-mcp/plugin-holded:latest
MANIFEST_TAG="ghcr.io/sevir/hyper-mcp/plugin-holded:latest"
docker manifest create "$MANIFEST_TAG" "ghcr.io/sevir/hyper-mcp/plugin-holded:latest" && docker manifest push "$MANIFEST_TAG" || echo "Skipping manifest for holded"

``` 