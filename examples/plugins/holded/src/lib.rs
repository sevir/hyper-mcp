mod pdk;

use extism_pdk::*;
use pdk::types::{
    CallToolRequest, CallToolResult, Content, ContentType, ListToolsResult, ToolDescription,
};
use serde_json::{Value as JsonValue, json};
use std::collections::HashMap;

const HOLDED_API_BASE_URL: &str = "https://api.holded.com/api";

/// Get Holded API configuration
fn get_holded_config() -> Result<String, Error> {
    let api_key = config::get("HOLDED_API_KEY")?
        .ok_or_else(|| Error::msg("HOLDED_API_KEY configuration is required but not set"))?;
    Ok(api_key)
}

/// Calculate similarity score between two strings
fn similarity_score(a: &str, b: &str) -> f64 {
    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    if a_lower == b_lower {
        return 1.0;
    }

    if a_lower.contains(&b_lower) || b_lower.contains(&a_lower) {
        let shorter_len = a_lower.len().min(b_lower.len()) as f64;
        let longer_len = a_lower.len().max(b_lower.len()) as f64;
        return shorter_len / longer_len;
    }

    // Simple Levenshtein-based similarity
    let len_a = a_lower.len();
    let len_b = b_lower.len();
    let max_len = len_a.max(len_b) as f64;

    if max_len == 0.0 {
        return 1.0;
    }

    let distance = levenshtein_distance(&a_lower, &b_lower);
    1.0 - (distance as f64 / max_len)
}

/// Levenshtein distance calculation
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();
    let mut matrix = vec![vec![0usize; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[len1][len2]
}

/// Make an HTTP request to Holded API
fn fetch_holded_api(endpoint: &str, params: Option<&str>) -> Result<JsonValue, Error> {
    let api_key = get_holded_config()?;
    let mut url = format!("{}{}", HOLDED_API_BASE_URL, endpoint);

    if let Some(query_params) = params {
        url = format!("{}?{}", url, query_params);
    }

    let req = HttpRequest::new(&url)
        .with_method("GET")
        .with_header("key", &api_key)
        .with_header("Accept", "application/json");

    match http::request::<()>(&req, None) {
        Ok(res) => {
            let body_str = String::from_utf8_lossy(&res.body()).to_string();
            if res.status_code() >= 200 && res.status_code() < 300 {
                serde_json::from_str::<JsonValue>(&body_str)
                    .map_err(|e| Error::msg(format!("Failed to parse JSON: {}", e)))
            } else {
                Err(Error::msg(format!(
                    "HTTP error {}: {}",
                    res.status_code(),
                    body_str
                )))
            }
        }
        Err(e) => Err(Error::msg(format!("HTTP request failed: {:?}", e))),
    }
}

/// Search for a contact by name with similarity matching
fn find_contact_by_name(contacts: &JsonValue, name: &str, threshold: f64) -> Option<JsonValue> {
    if let JsonValue::Array(contacts_array) = contacts {
        let mut best_match: Option<JsonValue> = None;
        let mut best_score = threshold;

        for contact in contacts_array {
            if let Some(contact_name) = contact.get("name").and_then(|v| v.as_str()) {
                let score = similarity_score(name, contact_name);
                if score > best_score {
                    best_score = score;
                    best_match = Some(contact.clone());
                }
            }
        }

        best_match
    } else {
        None
    }
}

/// Search for an employee by name with similarity matching
fn find_employee_by_name(employees: &JsonValue, name: &str, threshold: f64) -> Option<JsonValue> {
    if let JsonValue::Array(employees_array) = employees {
        let mut best_match: Option<JsonValue> = None;
        let mut best_score = threshold;

        for employee in employees_array {
            if let Some(employee_name) = employee.get("name").and_then(|v| v.as_str()) {
                let score = similarity_score(name, employee_name);
                if score > best_score {
                    best_score = score;
                    best_match = Some(employee.clone());
                }
            }
        }

        best_match
    } else {
        None
    }
}

/// Main call function that routes to specific tool implementations
pub(crate) fn call(input: CallToolRequest) -> Result<CallToolResult, Error> {
    match input.params.name.as_str() {
        "search_contact_by_name" => search_contact_by_name_tool(input),
        "list_contacts_with_invoices" => list_contacts_with_invoices_tool(input),
        "get_invoices_by_company_name" => get_invoices_by_company_name_tool(input),
        "get_invoices" => get_invoices_tool(input),
        "get_invoice_detail" => get_invoice_detail_tool(input),
        "list_employees" => list_employees_tool(input),
        "get_employee_calendar_by_name" => get_employee_calendar_by_name_tool(input),
        "get_all_employees_calendar" => get_all_employees_calendar_tool(input),
        "get_documents" => get_documents_tool(input),
        _ => Ok(CallToolResult {
            is_error: Some(true),
            content: vec![Content {
                annotations: None,
                text: Some(format!("Unknown tool: {}", input.params.name)),
                mime_type: None,
                r#type: ContentType::Text,
                data: None,
            }],
        }),
    }
}

/// Tool: Search contact by name
fn search_contact_by_name_tool(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();

    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::msg("Missing required parameter: name"))?;

    let threshold = args
        .get("threshold")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.6);

    let contacts = fetch_holded_api("/crm/v1/contacts", None)?;

    let text = if let Some(contact) = find_contact_by_name(&contacts, name, threshold) {
        format!(
            "✅ Contacto encontrado:\n\n\
            📋 Nombre: {}\n\
            🆔 ID: {}\n\
            📧 Email: {}\n\
            📱 Teléfono: {}\n\
            🏢 Tipo: {}\n\
            🔢 NIF/CIF: {}\n",
            contact
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("N/A"),
            contact.get("id").and_then(|v| v.as_str()).unwrap_or("N/A"),
            contact
                .get("email")
                .and_then(|v| v.as_str())
                .unwrap_or("N/A"),
            contact
                .get("phone")
                .and_then(|v| v.as_str())
                .unwrap_or("N/A"),
            contact
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("N/A"),
            contact
                .get("taxNumber")
                .and_then(|v| v.as_str())
                .unwrap_or("N/A")
        )
    } else {
        format!(
            "❌ No se encontró ningún contacto similar a '{}'.\n\
            Prueba con un nombre más completo o verifica la ortografía.",
            name
        )
    };

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(text),
            mime_type: Some("text/plain".to_string()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Tool: List contacts with invoices
fn list_contacts_with_invoices_tool(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();

    let mut params_vec = Vec::new();
    if let Some(date_from) = args.get("date_from").and_then(|v| v.as_str()) {
        params_vec.push(format!("dateFrom={}", urlencoding::encode(date_from)));
    }
    if let Some(date_to) = args.get("date_to").and_then(|v| v.as_str()) {
        params_vec.push(format!("dateTo={}", urlencoding::encode(date_to)));
    }
    if let Some(status) = args.get("status").and_then(|v| v.as_str()) {
        if status != "all" {
            params_vec.push(format!("status={}", urlencoding::encode(status)));
        }
    }

    let params_str = if params_vec.is_empty() {
        None
    } else {
        Some(params_vec.join("&"))
    };

    let invoices = fetch_holded_api("/invoicing/v1/documents/invoice", params_str.as_deref())?;

    let mut contact_stats: HashMap<String, ContactStats> = HashMap::new();

    if let JsonValue::Array(invoices_array) = &invoices {
        for inv in invoices_array {
            let contact_id = inv
                .get("contactId")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let contact_name = inv
                .get("contactName")
                .and_then(|v| v.as_str())
                .unwrap_or("Sin nombre")
                .to_string();

            let entry = contact_stats
                .entry(contact_id)
                .or_insert_with(|| ContactStats {
                    name: contact_name,
                    count: 0,
                    total: 0.0,
                    paid: 0.0,
                    unpaid: 0.0,
                    statuses: HashMap::new(),
                });

            entry.count += 1;
            let total = inv.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
            entry.total += total;

            let status = inv
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            *entry.statuses.entry(status.to_string()).or_insert(0) += 1;

            match status {
                "paid" => entry.paid += total,
                "sent" | "unpaid" | "partial" => entry.unpaid += total,
                _ => {}
            }
        }
    }

    let mut text = format!(
        "📊 Resumen de contactos con facturas\n\
        ==================================================\n\
        Período: {} → {}\n\
        Total contactos: {}\n\
        Total facturas: {}\n\n",
        args.get("date_from")
            .and_then(|v| v.as_str())
            .unwrap_or("inicio"),
        args.get("date_to")
            .and_then(|v| v.as_str())
            .unwrap_or("fin"),
        contact_stats.len(),
        contact_stats.values().map(|s| s.count).sum::<usize>()
    );

    let mut sorted_contacts: Vec<_> = contact_stats.iter().collect();
    sorted_contacts.sort_by(|a, b| b.1.total.partial_cmp(&a.1.total).unwrap());

    for (_, stats) in sorted_contacts.iter().take(20) {
        text.push_str(&format!(
            "\n🏢 {}\n\
            Facturas: {}\n\
            Total: {:.2}€\n\
            Cobrado: {:.2}€\n\
            Pendiente: {:.2}€\n\
            Estados: {}\n",
            stats.name,
            stats.count,
            stats.total,
            stats.paid,
            stats.unpaid,
            stats
                .statuses
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if sorted_contacts.len() > 20 {
        text.push_str(&format!(
            "\n... y {} contactos más.",
            sorted_contacts.len() - 20
        ));
    }

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(text),
            mime_type: Some("text/plain".to_string()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

struct ContactStats {
    name: String,
    count: usize,
    total: f64,
    paid: f64,
    unpaid: f64,
    statuses: HashMap<String, usize>,
}

/// Tool: Get invoices by company name
fn get_invoices_by_company_name_tool(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();

    let company_name = args
        .get("company_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::msg("Missing required parameter: company_name"))?;

    let contacts = fetch_holded_api("/crm/v1/contacts", None)?;
    let contact = find_contact_by_name(&contacts, company_name, 0.6);

    if contact.is_none() {
        return Ok(CallToolResult {
            is_error: Some(true),
            content: vec![Content {
                annotations: None,
                text: Some(format!(
                    "❌ No se encontró ningún contacto similar a '{}'.",
                    company_name
                )),
                mime_type: None,
                r#type: ContentType::Text,
                data: None,
            }],
        });
    }

    let contact = contact.unwrap();
    let contact_id = contact
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::msg("Contact ID not found"))?;

    let mut params_vec = vec![format!("contactId={}", urlencoding::encode(contact_id))];
    if let Some(date_from) = args.get("date_from").and_then(|v| v.as_str()) {
        params_vec.push(format!("dateFrom={}", urlencoding::encode(date_from)));
    }
    if let Some(date_to) = args.get("date_to").and_then(|v| v.as_str()) {
        params_vec.push(format!("dateTo={}", urlencoding::encode(date_to)));
    }
    if let Some(status) = args.get("status").and_then(|v| v.as_str()) {
        if status != "all" {
            params_vec.push(format!("status={}", urlencoding::encode(status)));
        }
    }

    let invoices = fetch_holded_api(
        "/invoicing/v1/documents/invoice",
        Some(&params_vec.join("&")),
    )?;

    let mut text = format!(
        "🏢 Facturas de: {}\n\
        ==================================================\n",
        contact
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("N/A")
    );

    let mut total_amount = 0.0;

    if let JsonValue::Array(invoices_array) = &invoices {
        text.push_str(&format!("Total facturas: {}\n\n", invoices_array.len()));

        for inv in invoices_array.iter().take(20) {
            text.push_str(&format!(
                "📄 Factura #{}\n\
                Fecha: {}\n\
                Importe: {}€\n\
                Estado: {}\n\
                ID: {}\n\n",
                inv.get("docNumber")
                    .and_then(|v| v.as_str())
                    .unwrap_or("N/A"),
                inv.get("date").and_then(|v| v.as_str()).unwrap_or("N/A"),
                inv.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0),
                inv.get("status").and_then(|v| v.as_str()).unwrap_or("N/A"),
                inv.get("id").and_then(|v| v.as_str()).unwrap_or("N/A")
            ));
            total_amount += inv.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
        }

        text.push_str(&format!("\n💰 Total: {:.2}€\n", total_amount));

        if invoices_array.len() > 20 {
            text.push_str(&format!(
                "... y {} facturas más.",
                invoices_array.len() - 20
            ));
        }
    }

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(text),
            mime_type: Some("text/plain".to_string()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Tool: Get invoices
fn get_invoices_tool(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();

    let mut params_vec = Vec::new();
    if let Some(status) = args.get("status").and_then(|v| v.as_str()) {
        if status != "all" {
            params_vec.push(format!("status={}", urlencoding::encode(status)));
        }
    }
    if let Some(date_from) = args.get("date_from").and_then(|v| v.as_str()) {
        params_vec.push(format!("dateFrom={}", urlencoding::encode(date_from)));
    }
    if let Some(date_to) = args.get("date_to").and_then(|v| v.as_str()) {
        params_vec.push(format!("dateTo={}", urlencoding::encode(date_to)));
    }
    if let Some(limit) = args.get("limit").and_then(|v| v.as_i64()) {
        params_vec.push(format!("limit={}", limit));
    }

    let params_str = if params_vec.is_empty() {
        None
    } else {
        Some(params_vec.join("&"))
    };

    let result = fetch_holded_api("/invoicing/v1/documents/invoice", params_str.as_deref())?;

    let text = if let JsonValue::Array(invoices_array) = &result {
        let total_amount: f64 = invoices_array
            .iter()
            .map(|inv| inv.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0))
            .sum();

        let mut output = format!(
            "📋 Lista de Facturas\n\
            ==================================================\n\
            Total facturas: {}\n\
            Importe total: {:.2}€\n\n",
            invoices_array.len(),
            total_amount
        );

        for inv in invoices_array.iter().take(15) {
            output.push_str(&format!(
                "📄 #{} - {}\n\
                {} | {}€ | {}\n\n",
                inv.get("docNumber")
                    .and_then(|v| v.as_str())
                    .unwrap_or("N/A"),
                inv.get("contactName")
                    .and_then(|v| v.as_str())
                    .unwrap_or("N/A"),
                inv.get("date").and_then(|v| v.as_str()).unwrap_or("N/A"),
                inv.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0),
                inv.get("status").and_then(|v| v.as_str()).unwrap_or("N/A")
            ));
        }

        if invoices_array.len() > 15 {
            output.push_str(&format!(
                "... y {} facturas más.",
                invoices_array.len() - 15
            ));
        }

        output
    } else {
        serde_json::to_string_pretty(&result)
            .unwrap_or_else(|_| "Error formatting JSON".to_string())
    };

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(text),
            mime_type: Some("text/plain".to_string()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Tool: Get invoice detail
fn get_invoice_detail_tool(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();

    let invoice_id = args
        .get("invoice_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::msg("Missing required parameter: invoice_id"))?;

    let result = fetch_holded_api(
        &format!("/invoicing/v1/documents/invoice/{}", invoice_id),
        None,
    )?;

    let text = serde_json::to_string_pretty(&result)
        .unwrap_or_else(|_| "Error formatting JSON".to_string());

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(text),
            mime_type: Some("application/json".to_string()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Tool: List employees
fn list_employees_tool(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();

    let mut params_vec = Vec::new();
    if let Some(JsonValue::Bool(active)) = args.get("active") {
        params_vec.push(format!("active={}", active));
    }
    if let Some(department) = args.get("department").and_then(|v| v.as_str()) {
        params_vec.push(format!("department={}", urlencoding::encode(department)));
    }

    let params_str = if params_vec.is_empty() {
        None
    } else {
        Some(params_vec.join("&"))
    };

    let employees = fetch_holded_api("/team/v1/employees", params_str.as_deref())?;

    let mut text = String::from(
        "👥 Lista de Empleados\n\
        ==================================================\n",
    );

    if let JsonValue::Array(employees_array) = &employees {
        text.push_str(&format!("Total: {} empleados\n\n", employees_array.len()));

        for emp in employees_array {
            text.push_str(&format!(
                "👤 {}\n\
                🆔 ID: {}\n\
                📧 Email: {}\n\
                💼 Puesto: {}\n\
                🏢 Departamento: {}\n\
                ✅ Activo: {}\n\n",
                emp.get("name").and_then(|v| v.as_str()).unwrap_or("N/A"),
                emp.get("id").and_then(|v| v.as_str()).unwrap_or("N/A"),
                emp.get("email").and_then(|v| v.as_str()).unwrap_or("N/A"),
                emp.get("position")
                    .and_then(|v| v.as_str())
                    .unwrap_or("N/A"),
                emp.get("department")
                    .and_then(|v| v.as_str())
                    .unwrap_or("N/A"),
                if emp.get("active").and_then(|v| v.as_bool()).unwrap_or(false) {
                    "Sí"
                } else {
                    "No"
                }
            ));
        }
    }

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(text),
            mime_type: Some("text/plain".to_string()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Tool: Get employee calendar by name
fn get_employee_calendar_by_name_tool(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();

    let employee_name = args
        .get("employee_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::msg("Missing required parameter: employee_name"))?;

    let employees = fetch_holded_api("/team/v1/employees", None)?;
    let employee = find_employee_by_name(&employees, employee_name, 0.6);

    if employee.is_none() {
        return Ok(CallToolResult {
            is_error: Some(true),
            content: vec![Content {
                annotations: None,
                text: Some(format!(
                    "❌ No se encontró ningún empleado similar a '{}'.",
                    employee_name
                )),
                mime_type: None,
                r#type: ContentType::Text,
                data: None,
            }],
        });
    }

    let employee = employee.unwrap();
    let employee_id = employee
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::msg("Employee ID not found"))?;

    let mut params_vec = vec![format!("employeeId={}", urlencoding::encode(employee_id))];
    if let Some(date_from) = args.get("date_from").and_then(|v| v.as_str()) {
        params_vec.push(format!("dateFrom={}", urlencoding::encode(date_from)));
    }
    if let Some(date_to) = args.get("date_to").and_then(|v| v.as_str()) {
        params_vec.push(format!("dateTo={}", urlencoding::encode(date_to)));
    }
    if let Some(event_type) = args.get("type").and_then(|v| v.as_str()) {
        if event_type != "all" {
            params_vec.push(format!("type={}", urlencoding::encode(event_type)));
        }
    }

    let events = fetch_holded_api("/team/v1/calendar", Some(&params_vec.join("&")))?;

    let mut text = format!(
        "📅 Calendario de: {}\n\
        ==================================================\n",
        employee
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("N/A")
    );

    if let JsonValue::Array(events_array) = &events {
        text.push_str(&format!("Total eventos: {}\n\n", events_array.len()));

        for event in events_array {
            text.push_str(&format!(
                "📌 {}\n\
                📅 Desde: {}\n\
                📅 Hasta: {}\n\
                📝 Notas: {}\n\n",
                event
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("N/A")
                    .to_uppercase(),
                event
                    .get("dateFrom")
                    .and_then(|v| v.as_str())
                    .unwrap_or("N/A"),
                event
                    .get("dateTo")
                    .and_then(|v| v.as_str())
                    .unwrap_or("N/A"),
                event
                    .get("notes")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Sin notas")
            ));
        }
    }

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(text),
            mime_type: Some("text/plain".to_string()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Tool: Get all employees calendar
fn get_all_employees_calendar_tool(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();

    // Get all employees first to build a map
    let employees = fetch_holded_api("/team/v1/employees", None)?;
    let mut employee_map: HashMap<String, String> = HashMap::new();

    if let JsonValue::Array(employees_array) = &employees {
        for emp in employees_array {
            if let (Some(id), Some(name)) = (
                emp.get("id").and_then(|v| v.as_str()),
                emp.get("name").and_then(|v| v.as_str()),
            ) {
                employee_map.insert(id.to_string(), name.to_string());
            }
        }
    }

    let mut params_vec = Vec::new();
    if let Some(date_from) = args.get("date_from").and_then(|v| v.as_str()) {
        params_vec.push(format!("dateFrom={}", urlencoding::encode(date_from)));
    }
    if let Some(date_to) = args.get("date_to").and_then(|v| v.as_str()) {
        params_vec.push(format!("dateTo={}", urlencoding::encode(date_to)));
    }
    if let Some(event_type) = args.get("type").and_then(|v| v.as_str()) {
        if event_type != "all" {
            params_vec.push(format!("type={}", urlencoding::encode(event_type)));
        }
    }

    let params_str = if params_vec.is_empty() {
        None
    } else {
        Some(params_vec.join("&"))
    };

    let events = fetch_holded_api("/team/v1/calendar", params_str.as_deref())?;

    let mut text = format!(
        "📅 Calendario de Todos los Empleados\n\
        ==================================================\n\
        Período: {} → {}\n",
        args.get("date_from")
            .and_then(|v| v.as_str())
            .unwrap_or("inicio"),
        args.get("date_to")
            .and_then(|v| v.as_str())
            .unwrap_or("fin")
    );

    let mut events_by_employee: HashMap<String, Vec<&JsonValue>> = HashMap::new();

    if let JsonValue::Array(events_array) = &events {
        text.push_str(&format!("Total eventos: {}\n\n", events_array.len()));

        for event in events_array {
            if let Some(emp_id) = event.get("employeeId").and_then(|v| v.as_str()) {
                let emp_name = employee_map
                    .get(emp_id)
                    .cloned()
                    .or_else(|| {
                        event
                            .get("employeeName")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                    })
                    .unwrap_or_else(|| format!("ID: {}", emp_id));

                events_by_employee
                    .entry(emp_name)
                    .or_insert_with(Vec::new)
                    .push(event);
            }
        }

        let mut sorted_employees: Vec<_> = events_by_employee.iter().collect();
        sorted_employees.sort_by(|a, b| a.0.cmp(b.0));

        for (emp_name, emp_events) in sorted_employees {
            text.push_str(&format!(
                "\n👤 {} ({} eventos)\n\
                ----------------------------------------\n",
                emp_name,
                emp_events.len()
            ));

            for event in emp_events {
                text.push_str(&format!(
                    "  📌 {} | {} → {}\n",
                    event.get("type").and_then(|v| v.as_str()).unwrap_or("N/A"),
                    event
                        .get("dateFrom")
                        .and_then(|v| v.as_str())
                        .unwrap_or("N/A"),
                    event
                        .get("dateTo")
                        .and_then(|v| v.as_str())
                        .unwrap_or("N/A")
                ));

                if let Some(notes) = event.get("notes").and_then(|v| v.as_str()) {
                    text.push_str(&format!("     💬 {}\n", notes));
                }
            }
        }
    }

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(text),
            mime_type: Some("text/plain".to_string()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Tool: Get documents
fn get_documents_tool(input: CallToolRequest) -> Result<CallToolResult, Error> {
    let args = input.params.arguments.unwrap_or_default();

    let doc_type = args
        .get("doc_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::msg("Missing required parameter: doc_type"))?;

    let mut params_vec = Vec::new();
    if let Some(limit) = args.get("limit").and_then(|v| v.as_i64()) {
        params_vec.push(format!("limit={}", limit));
    }

    let params_str = if params_vec.is_empty() {
        None
    } else {
        Some(params_vec.join("&"))
    };

    let result = fetch_holded_api(
        &format!("/invoicing/v1/documents/{}", doc_type),
        params_str.as_deref(),
    )?;

    let text = if let JsonValue::Array(docs_array) = &result {
        let mut output = format!(
            "📋 Documentos tipo '{}'\n\
            ==================================================\n\
            Total: {} documentos\n\n",
            doc_type,
            docs_array.len()
        );

        for doc in docs_array.iter().take(10) {
            output.push_str(&format!(
                "📄 #{} - {}\n\
                {} | {}€\n\n",
                doc.get("docNumber")
                    .and_then(|v| v.as_str())
                    .unwrap_or("N/A"),
                doc.get("contactName")
                    .and_then(|v| v.as_str())
                    .unwrap_or("N/A"),
                doc.get("date").and_then(|v| v.as_str()).unwrap_or("N/A"),
                doc.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0)
            ));
        }

        if docs_array.len() > 10 {
            output.push_str(&format!("... y {} documentos más.", docs_array.len() - 10));
        }

        output
    } else {
        serde_json::to_string_pretty(&result)
            .unwrap_or_else(|_| "Error formatting JSON".to_string())
    };

    Ok(CallToolResult {
        is_error: None,
        content: vec![Content {
            annotations: None,
            text: Some(text),
            mime_type: Some("text/plain".to_string()),
            r#type: ContentType::Text,
            data: None,
        }],
    })
}

/// Describe function that returns all available tools
pub(crate) fn describe() -> Result<ListToolsResult, Error> {
    Ok(ListToolsResult {
        tools: vec![
            ToolDescription {
                name: "search_contact_by_name".to_string(),
                description: "Busca un contacto (cliente/proveedor) por similitud de nombre. Útil cuando no conoces el ID exacto.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Nombre de la empresa o contacto a buscar"
                        },
                        "threshold": {
                            "type": "number",
                            "description": "Umbral de similitud (0-1, default: 0.6). Mayor = más estricto",
                            "default": 0.6
                        }
                    },
                    "required": ["name"]
                })
                .as_object()
                .unwrap()
                .clone(),
            },
            ToolDescription {
                name: "list_contacts_with_invoices".to_string(),
                description: "Lista todos los contactos que tienen facturas en un período específico, mostrando totales y estados.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "date_from": {
                            "type": "string",
                            "description": "Fecha desde (formato: YYYY-MM-DD)"
                        },
                        "date_to": {
                            "type": "string",
                            "description": "Fecha hasta (formato: YYYY-MM-DD)"
                        },
                        "status": {
                            "type": "string",
                            "description": "Filtrar por estado de factura",
                            "enum": ["draft", "sent", "paid", "partial", "unpaid", "all"]
                        }
                    }
                })
                .as_object()
                .unwrap()
                .clone(),
            },
            ToolDescription {
                name: "get_invoices_by_company_name".to_string(),
                description: "Obtiene facturas de una empresa buscando por nombre (no necesitas el ID exacto). Detecta automáticamente el contacto más similar.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "company_name": {
                            "type": "string",
                            "description": "Nombre de la empresa (búsqueda por similitud)"
                        },
                        "date_from": {
                            "type": "string",
                            "description": "Fecha desde (formato: YYYY-MM-DD)"
                        },
                        "date_to": {
                            "type": "string",
                            "description": "Fecha hasta (formato: YYYY-MM-DD)"
                        },
                        "status": {
                            "type": "string",
                            "description": "Estado de factura",
                            "enum": ["draft", "sent", "paid", "partial", "unpaid", "all"]
                        }
                    },
                    "required": ["company_name"]
                })
                .as_object()
                .unwrap()
                .clone(),
            },
            ToolDescription {
                name: "get_invoices".to_string(),
                description: "Obtiene la lista de facturas con filtros avanzados. Para buscar por empresa usa 'get_invoices_by_company_name'.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "status": {
                            "type": "string",
                            "description": "Estado de la factura",
                            "enum": ["draft", "sent", "paid", "partial", "unpaid", "all"]
                        },
                        "date_from": {
                            "type": "string",
                            "description": "Fecha desde (formato: YYYY-MM-DD)"
                        },
                        "date_to": {
                            "type": "string",
                            "description": "Fecha hasta (formato: YYYY-MM-DD)"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Número máximo de resultados",
                            "default": 100
                        }
                    }
                })
                .as_object()
                .unwrap()
                .clone(),
            },
            ToolDescription {
                name: "get_invoice_detail".to_string(),
                description: "Obtiene los detalles completos de una factura específica por su ID".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "invoice_id": {
                            "type": "string",
                            "description": "ID de la factura a consultar"
                        }
                    },
                    "required": ["invoice_id"]
                })
                .as_object()
                .unwrap()
                .clone(),
            },
            ToolDescription {
                name: "list_employees".to_string(),
                description: "Lista todos los empleados con su ID y nombre completo. Útil para luego consultar calendarios específicos.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "active": {
                            "type": "boolean",
                            "description": "Filtrar solo empleados activos (true) o todos (false/null)"
                        },
                        "department": {
                            "type": "string",
                            "description": "Filtrar por departamento"
                        }
                    }
                })
                .as_object()
                .unwrap()
                .clone(),
            },
            ToolDescription {
                name: "get_employee_calendar_by_name".to_string(),
                description: "Obtiene el calendario de un empleado buscando por nombre (no necesitas el ID). Detecta automáticamente el empleado más similar.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "employee_name": {
                            "type": "string",
                            "description": "Nombre del empleado (búsqueda por similitud)"
                        },
                        "date_from": {
                            "type": "string",
                            "description": "Fecha desde (formato: YYYY-MM-DD)"
                        },
                        "date_to": {
                            "type": "string",
                            "description": "Fecha hasta (formato: YYYY-MM-DD)"
                        },
                        "type": {
                            "type": "string",
                            "description": "Tipo de evento",
                            "enum": ["absence", "vacation", "sick_leave", "holiday", "all"]
                        }
                    },
                    "required": ["employee_name"]
                })
                .as_object()
                .unwrap()
                .clone(),
            },
            ToolDescription {
                name: "get_all_employees_calendar".to_string(),
                description: "Obtiene el calendario de TODOS los empleados para un período, con nombres completos en lugar de IDs.".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "date_from": {
                            "type": "string",
                            "description": "Fecha desde (formato: YYYY-MM-DD)"
                        },
                        "date_to": {
                            "type": "string",
                            "description": "Fecha hasta (formato: YYYY-MM-DD)"
                        },
                        "type": {
                            "type": "string",
                            "description": "Tipo de evento",
                            "enum": ["absence", "vacation", "sick_leave", "holiday", "all"]
                        }
                    }
                })
                .as_object()
                .unwrap()
                .clone(),
            },
            ToolDescription {
                name: "get_documents".to_string(),
                description: "Obtiene documentos de cualquier tipo (presupuestos, pedidos, albaranes, etc.)".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "doc_type": {
                            "type": "string",
                            "description": "Tipo de documento",
                            "enum": ["invoice", "estimate", "order", "purchaseorder", "proforma", "deliverynote"]
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Número máximo de resultados",
                            "default": 50
                        }
                    },
                    "required": ["doc_type"]
                })
                .as_object()
                .unwrap()
                .clone(),
            },
        ],
    })
}
