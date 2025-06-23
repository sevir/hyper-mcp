use crate::Cli;
use crate::config::Config;
use crate::oci::OciDownloader;
use anyhow::Result;
use bytesize::ByteSize;
use extism::{Manifest, Plugin, Wasm};
use rmcp::service::{NotificationContext, RequestContext, RoleServer};
use rmcp::{Error as McpError, ServerHandler, model::*};
use std::str::FromStr;

use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct PluginService {
    config: Config,
    plugins: Arc<RwLock<HashMap<String, Arc<Mutex<Plugin>>>>>,
    tool_plugin_map: Arc<RwLock<HashMap<String, String>>>,
    oci_downloader: Arc<OciDownloader>,
}

impl PluginService {
    pub async fn new(config: Config, cli: &Cli) -> Result<Self> {
        // Create OCI downloader with CLI object
        let oci_downloader = Arc::new(OciDownloader::new(cli));

        let service = Self {
            config,
            plugins: Arc::new(RwLock::new(HashMap::new())),
            tool_plugin_map: Arc::new(RwLock::new(HashMap::new())),
            oci_downloader,
        };

        service.load_plugins().await?;
        Ok(service)
    }

    async fn load_plugins(&self) -> Result<()> {
        for plugin_cfg in &self.config.plugins {
            let wasm_content = if plugin_cfg.path.starts_with("http") {
                reqwest::get(&plugin_cfg.path)
                    .await?
                    .bytes()
                    .await?
                    .to_vec()
            } else if plugin_cfg.path.starts_with("oci") {
                // ref should be like oci://tuananh/qr-code
                let image_reference = plugin_cfg.path.strip_prefix("oci://").unwrap();
                let target_file_path = "/plugin.wasm";
                let mut hasher = Sha256::new();
                hasher.update(image_reference);
                let hash = hasher.finalize();
                let short_hash = &hex::encode(hash)[..7];
                let cache_dir = dirs::cache_dir()
                    .map(|mut path| {
                        path.push("hyper-mcp");
                        path
                    })
                    .unwrap();
                std::fs::create_dir_all(&cache_dir)?;

                let local_output_path =
                    cache_dir.join(format!("{}-{}.wasm", plugin_cfg.name, short_hash));
                let local_output_path = local_output_path.to_str().unwrap();

                if let Err(e) = self
                    .oci_downloader
                    .pull_and_extract(image_reference, target_file_path, local_output_path)
                    .await
                {
                    log::error!("Error pulling oci plugin: {}", e);
                    return Err(anyhow::anyhow!("Failed to pull OCI plugin: {}", e));
                }
                log::info!(
                    "cache plugin `{}` to : {}",
                    plugin_cfg.name,
                    local_output_path
                );
                tokio::fs::read(local_output_path).await?
            } else {
                tokio::fs::read(&plugin_cfg.path).await?
            };
            let mut manifest = Manifest::new([Wasm::data(wasm_content)]);
            if let Some(runtime_cfg) = &plugin_cfg.runtime_config {
                log::info!("runtime_cfg: {:?}", runtime_cfg);
                if let Some(hosts) = &runtime_cfg.allowed_hosts {
                    for host in hosts {
                        manifest = manifest.with_allowed_host(host);
                    }
                }
                if let Some(paths) = &runtime_cfg.allowed_paths {
                    for path in paths {
                        // path will be available in the plugin with exact same path
                        manifest = manifest.with_allowed_path(path.clone(), path.clone());
                    }
                }

                // Add plugin configurations if present
                if let Some(env_vars) = &runtime_cfg.env_vars {
                    for (key, value) in env_vars {
                        manifest = manifest.with_config_key(key, value);
                    }
                }

                if let Some(memory_limit) = &runtime_cfg.memory_limit {
                    match ByteSize::from_str(memory_limit) {
                        Ok(b) => {
                            // Wasm page size 64KiB, convert to number of pages
                            let num_pages = b.as_u64() / (64 * 1024);
                            manifest = manifest.with_memory_max(num_pages as u32);
                        }
                        Err(e) => {
                            log::error!(
                                "Failed to parse memory_limit '{}': {}. Using default memory limit.",
                                memory_limit,
                                e
                            );
                        }
                    }
                }
            }
            let plugin = Arc::new(Mutex::new(Plugin::new(&manifest, [], true).unwrap()));
            let plugin_clone = Arc::clone(&plugin);

            // Try to get tool information from the plugin and populate the cache
            let describe_result = tokio::task::spawn_blocking(move || {
                let mut plugin = plugin_clone.lock().unwrap();
                plugin.call::<&str, String>("describe", "")
            })
            .await;

            if let Ok(Ok(result)) = describe_result {
                if let Ok(parsed) = serde_json::from_str::<ListToolsResult>(&result) {
                    let mut cache = self.tool_plugin_map.write().await;
                    let skip_tools = plugin_cfg
                        .runtime_config
                        .as_ref()
                        .and_then(|rc| rc.skip_tools.clone())
                        .unwrap_or_default();
                    for tool in parsed.tools {
                        if skip_tools.iter().any(|s| s == tool.name.as_ref() as &str) {
                            log::info!("Skipping tool {} as requested in skip_tools", tool.name);
                            continue;
                        }
                        log::info!("Saving tool {}/{} to cache", plugin_cfg.name, tool.name);
                        // Check if the tool name already exists in another plugin
                        if let Some(existing_plugin) = cache.get(&tool.name.to_string()) {
                            if existing_plugin != &plugin_cfg.name {
                                log::error!(
                                    "Tool name collision detected: {} is provided by both {} and {} plugins",
                                    tool.name,
                                    existing_plugin,
                                    plugin_cfg.name
                                );
                                panic!(
                                    "Tool name collision detected: {} is provided by both {} and {} plugins",
                                    tool.name, existing_plugin, plugin_cfg.name
                                );
                            }
                        }
                        cache.insert(tool.name.to_string(), plugin_cfg.name.clone());
                    }
                }
            }

            let plugin_name = plugin_cfg.name.clone();
            self.plugins
                .write()
                .await
                .insert(plugin_name.clone(), plugin);
            log::info!("Loaded plugin {}", plugin_name);
        }
        Ok(())
    }
}

impl ServerHandler for PluginService {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            server_info: Implementation {
                name: "hyper-mcp".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
            capabilities: ServerCapabilities::builder().enable_tools().build(),

            ..Default::default()
        }
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let plugins = self.plugins.read().await;
        let tool_cache = self.tool_plugin_map.read().await;

        let tool_name = request.name.clone();
        let call_payload = json!({
            "params": request,
        });
        let json_string =
            serde_json::to_string(&call_payload).expect("Failed to serialize request");

        // Check if the tool exists in the cache
        if let Some(plugin_name) = tool_cache.get(&tool_name.to_string()) {
            if let Some(plugin_arc) = plugins.get(plugin_name) {
                let plugin_clone = Arc::clone(plugin_arc);
                let plugin_name_clone = plugin_name.clone();

                let result = tokio::task::spawn_blocking(move || {
                    let mut plugin = plugin_clone.lock().unwrap();
                    plugin.call::<&str, String>("call", &json_string)
                })
                .await;

                return match result {
                    Ok(Ok(result)) => match serde_json::from_str::<CallToolResult>(&result) {
                        Ok(parsed) => Ok(parsed),
                        Err(e) => Err(McpError::internal_error(
                            format!("Failed to deserialize data: {}", e),
                            None,
                        )),
                    },
                    Ok(Err(e)) => Err(McpError::internal_error(
                        format!("Failed to execute plugin {}: {}", plugin_name_clone, e),
                        None,
                    )),
                    Err(e) => Err(McpError::internal_error(
                        format!(
                            "Failed to spawn blocking task for plugin {}: {}",
                            plugin_name_clone, e
                        ),
                        None,
                    )),
                };
            }
        }

        Err(McpError::method_not_found::<CallToolRequestMethod>())
    }

    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListToolsResult, McpError> {
        tracing::info!("got tools/list request {:?}", request);
        let plugins = self.plugins.write().await;
        let mut tool_cache = self.tool_plugin_map.write().await;

        let mut payload = ListToolsResult::default();

        // Clear the existing cache when listing tools
        tool_cache.clear();

        for plugin_cfg in &self.config.plugins {
            if let Some(plugin_arc) = plugins.get(&plugin_cfg.name) {
                let plugin_clone = Arc::clone(plugin_arc);
                let plugin_name = plugin_cfg.name.clone();

                let result = tokio::task::spawn_blocking(move || {
                    let mut plugin = plugin_clone.lock().unwrap();
                    plugin.call::<&str, String>("describe", "")
                })
                .await;

                match result {
                    Ok(Ok(result)) => {
                        if let Ok(parsed) = serde_json::from_str::<ListToolsResult>(&result) {
                            let skip_tools = plugin_cfg
                                .runtime_config
                                .as_ref()
                                .and_then(|rc| rc.skip_tools.clone())
                                .unwrap_or_default();
                            for tool in parsed.tools {
                                if skip_tools.iter().any(|s| s == tool.name.as_ref() as &str) {
                                    log::info!(
                                        "Skipping tool {} as requested in skip_tools",
                                        tool.name
                                    );
                                    continue;
                                }
                                tool_cache.insert(tool.name.to_string(), plugin_cfg.name.clone());
                                payload.tools.push(tool);
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        log::error!("tool {} describe() error: {}", plugin_name, e);
                    }
                    Err(e) => {
                        log::error!("tool {} spawn_blocking error: {}", plugin_name, e);
                    }
                }
            }
        }

        Ok(payload)
    }

    // fn list_tools(
    //     &self,
    //     _request: Option<PaginatedRequestParam>,
    //     _context: RequestContext<RoleServer>,
    // ) -> impl Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
    //     tracing::info!("got tools/list request {:?}", _request);
    //     std::future::ready(Ok(ListToolsResult::default()))
    // }

    fn initialize(
        &self,
        request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<InitializeResult, McpError>> + Send + '_ {
        tracing::info!("got initialize request {:?}", request);
        std::future::ready(Ok(self.get_info()))
    }

    fn ping(
        &self,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = std::result::Result<(), McpError>> + Send + '_ {
        tracing::info!("got ping request");
        std::future::ready(Ok(()))
    }

    fn on_initialized(
        &self,
        _context: NotificationContext<RoleServer>,
    ) -> impl Future<Output = ()> + Send + '_ {
        tracing::info!("got initialized notification");
        std::future::ready(())
    }

    fn on_cancelled(
        &self,
        _notification: CancelledNotificationParam,
        _context: NotificationContext<RoleServer>,
    ) -> impl Future<Output = ()> + Send + '_ {
        std::future::ready(())
    }

    fn on_progress(
        &self,
        _notification: ProgressNotificationParam,
        _context: NotificationContext<RoleServer>,
    ) -> impl Future<Output = ()> + Send + '_ {
        std::future::ready(())
    }

    fn complete(
        &self,
        request: CompleteRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = std::result::Result<CompleteResult, McpError>> + Send + '_ {
        tracing::info!("got complete request {:?}", request);
        std::future::ready(Err(McpError::method_not_found::<CompleteRequestMethod>()))
    }
}
