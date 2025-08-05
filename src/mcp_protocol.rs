use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

/// MCP Protocol implementation for registering ws as an MCP server with Claude
/// Complete production implementation following Model Context Protocol specification

#[derive(Debug, Serialize, Deserialize)]
pub struct McpMessage {
    pub jsonrpc: String,
    pub id: Option<u64>,
    pub method: Option<String>,
    pub params: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<McpError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub tools: Option<ToolsCapability>,
    pub resources: Option<ResourcesCapability>,
    pub prompts: Option<PromptsCapability>,
    pub logging: Option<LoggingCapability>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolsCapability {
    pub list_changed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourcesCapability {
    pub subscribe: Option<bool>,
    pub list_changed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptsCapability {
    pub list_changed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingCapability {
    pub level: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCallRequest {
    pub name: String,
    pub arguments: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub content: Vec<ToolContent>,
    pub is_error: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

/// Complete MCP Protocol handler for ws integration with Claude
pub struct McpProtocolHandler {
    message_id_counter: u64,
    stdin_writer: Option<tokio::io::Stdin>,
    context_usage_counter: u64,
    context_threshold: u64,
    session_active: bool,
    session_metrics: SessionMetrics,
}

#[derive(Debug, Clone)]
pub struct SessionMetrics {
    pub session_start: std::time::Instant,
    pub total_messages: u64,
    pub tool_calls: u64,
    pub context_usage: u64,
    pub response_times: Vec<u64>, // milliseconds
    pub session_id: String,
}

impl McpProtocolHandler {
    pub fn new() -> Self {
        let session_id = format!("session_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
        
        Self {
            message_id_counter: 0,
            stdin_writer: None,
            context_usage_counter: 0,
            context_threshold: 190000, // 95% of typical 200k context limit
            session_active: false,
            session_metrics: SessionMetrics {
                session_start: std::time::Instant::now(),
                total_messages: 0,
                tool_calls: 0,
                context_usage: 0,
                response_times: Vec::new(),
                session_id,
            },
        }
    }

    /// Start the complete MCP server stdio protocol
    pub async fn start_mcp_server() -> Result<()> {
        let mut handler = Self::new();
        
        // Initialize stdio communication with Claude
        handler.initialize_stdio().await?;
        
        // Register server capabilities
        handler.register_server().await?;
        
        // Automatically initialize session on startup
        if let Err(e) = handler.initialize_session_automatically().await {
            eprintln!("Warning: Failed to initialize session automatically: {}", e);
        } else {
            handler.session_active = true;
        }
        
        // Start message processing loop
        handler.process_messages().await?;
        
        Ok(())
    }

    /// Initialize stdio communication channels
    async fn initialize_stdio(&mut self) -> Result<()> {
        // MCP protocol uses stdio for communication with Claude
        self.stdin_writer = Some(tokio::io::stdin());
        
        eprintln!("MCP server initialized with stdio communication");
        Ok(())
    }

    /// Register ws as an MCP server with complete capabilities
    pub async fn register_server(&mut self) -> Result<()> {
        // Send initialize message
        let init_message = self.create_initialize_message().await?;
        self.send_message_to_claude(&init_message).await?;
        
        // Send initialized notification
        let initialized_message = self.create_initialized_message().await?;
        self.send_message_to_claude(&initialized_message).await?;
        
        eprintln!("MCP server registration completed successfully");
        Ok(())
    }

    /// Process incoming messages from Claude
    async fn process_messages(&mut self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let reader = BufReader::new(stdin);
        let mut lines = reader.lines();
        
        eprintln!("MCP server listening for messages from Claude...");
        
        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }
            
            match serde_json::from_str::<McpMessage>(&line) {
                Ok(message) => {
                    if let Err(e) = self.handle_message(message).await {
                        eprintln!("Error handling message: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse MCP message: {} - Line: {}", e, line);
                }
            }
        }
        
        Ok(())
    }

    /// Handle incoming MCP message from Claude
    async fn handle_message(&mut self, message: McpMessage) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        // Track message metrics
        self.session_metrics.total_messages += 1;
        self.track_context_usage(&message).await?;
        
        match message.method.as_deref() {
            Some("tools/list") => {
                let response = self.handle_tools_list(message.id).await?;
                self.send_message_to_claude(&response).await?;
            }
            Some("tools/call") => {
                self.session_metrics.tool_calls += 1;
                if let Some(params) = message.params {
                    let response = self.handle_tools_call(message.id, params).await?;
                    self.send_message_to_claude(&response).await?;
                }
            }
            Some("initialize") => {
                let response = self.handle_initialize_response(message.id).await?;
                self.send_message_to_claude(&response).await?;
            }
            _ => {
                eprintln!("Unhandled MCP method: {:?}", message.method);
            }
        }
        
        // Record response time
        let response_time = start_time.elapsed().as_millis() as u64;
        self.session_metrics.response_times.push(response_time);
        
        // Check context threshold after processing message
        self.check_context_threshold().await?;
        
        Ok(())
    }

    /// Handle tools/list request from Claude
    async fn handle_tools_list(&self, request_id: Option<u64>) -> Result<McpMessage> {
        let tools = self.get_available_tools().await?;
        
        Ok(McpMessage {
            jsonrpc: "2.0".to_string(),
            id: request_id,
            method: None,
            params: None,
            result: Some(serde_json::json!({
                "tools": tools
            })),
            error: None,
        })
    }

    /// Handle tools/call request from Claude
    async fn handle_tools_call(&self, request_id: Option<u64>, params: serde_json::Value) -> Result<McpMessage> {
        let tool_call: ToolCallRequest = serde_json::from_value(params)
            .context("Failed to parse tool call request")?;
        
        let result = self.execute_tool_call(tool_call).await?;
        
        Ok(McpMessage {
            jsonrpc: "2.0".to_string(),
            id: request_id,
            method: None,
            params: None,
            result: Some(serde_json::to_value(result)?),
            error: None,
        })
    }

    /// Handle initialize response
    async fn handle_initialize_response(&self, request_id: Option<u64>) -> Result<McpMessage> {
        let capabilities = ServerCapabilities {
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
            resources: None,
            prompts: None,
            logging: Some(LoggingCapability {
                level: Some("info".to_string()),
            }),
        };

        Ok(McpMessage {
            jsonrpc: "2.0".to_string(),
            id: request_id,
            method: None,
            params: None,
            result: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": capabilities,
                "serverInfo": {
                    "name": "ws-workspace-tools",
                    "version": "0.42.55914"
                }
            })),
            error: None,
        })
    }

    /// Create the initialize message with server capabilities
    async fn create_initialize_message(&mut self) -> Result<McpMessage> {
        let capabilities = ServerCapabilities {
            tools: Some(ToolsCapability {
                list_changed: Some(true),
            }),
            resources: None,
            prompts: None,
            logging: Some(LoggingCapability {
                level: Some("info".to_string()),
            }),
        };

        let params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": capabilities,
            "serverInfo": {
                "name": "ws-workspace-tools",
                "version": "0.42.55914"
            }
        });

        Ok(McpMessage {
            jsonrpc: "2.0".to_string(),
            id: Some(self.next_message_id()),
            method: Some("initialize".to_string()),
            params: Some(params),
            result: None,
            error: None,
        })
    }

    /// Create the initialized notification message
    async fn create_initialized_message(&mut self) -> Result<McpMessage> {
        Ok(McpMessage {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: Some("notifications/initialized".to_string()),
            params: Some(serde_json::json!({})),
            result: None,
            error: None,
        })
    }

    /// Get all available tools that Claude can call
    pub async fn get_available_tools(&self) -> Result<Vec<Tool>> {
        Ok(vec![
            Tool {
                name: "add_feature".to_string(),
                description: "Add a new feature to the project feature tracking system".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Feature name"
                        },
                        "description": {
                            "type": "string", 
                            "description": "Detailed feature description"
                        },
                        "category": {
                            "type": "string",
                            "description": "Feature category (core, command, mcp, etc.)",
                            "enum": ["core", "command", "mcp", "api", "testing", "infrastructure"]
                        },
                        "priority": {
                            "type": "string",
                            "description": "Feature priority level",
                            "enum": ["high", "medium", "low"]
                        }
                    },
                    "required": ["name", "description"]
                }),
            },
            Tool {
                name: "update_feature_state".to_string(),
                description: "Update the implementation state or test status of an existing feature".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "feature_id": {
                            "type": "string",
                            "description": "Feature ID (e.g., F0111)"
                        },
                        "state": {
                            "type": "string",
                            "description": "New feature state emoji",
                            "enum": ["❌", "🟠", "🟢", "🟡", "⚠️", "🔴"]
                        },
                        "test_status": {
                            "type": "string",
                            "description": "Test status description"
                        },
                        "notes": {
                            "type": "string",
                            "description": "Implementation notes or updates"
                        }
                    },
                    "required": ["feature_id"]
                }),
            },
            Tool {
                name: "list_features".to_string(),
                description: "List features with optional filtering by state, category, or test status".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "state": {
                            "type": "string",
                            "description": "Filter by feature state emoji",
                            "enum": ["❌", "🟠", "🟢", "🟡", "⚠️", "🔴"]
                        },
                        "category": {
                            "type": "string",
                            "description": "Filter by category"
                        },
                        "recent": {
                            "type": "boolean",
                            "description": "Show only recently updated features"
                        }
                    }
                }),
            },
            Tool {
                name: "add_task".to_string(),
                description: "Add a new task to the project task management system".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "Task title"
                        },
                        "description": {
                            "type": "string",
                            "description": "Detailed task description"
                        },
                        "feature_id": {
                            "type": "string",
                            "description": "Associated feature ID (optional)"
                        },
                        "priority": {
                            "type": "string",
                            "description": "Task priority",
                            "enum": ["high", "medium", "low"]
                        }
                    },
                    "required": ["title", "description"]
                }),
            },
            Tool {
                name: "update_task_status".to_string(),
                description: "Update the status of an existing task".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "task_id": {
                            "type": "string",
                            "description": "Task ID"
                        },
                        "status": {
                            "type": "string",
                            "description": "New task status",
                            "enum": ["pending", "in_progress", "completed", "blocked", "cancelled"]
                        },
                        "notes": {
                            "type": "string",
                            "description": "Status update notes"
                        }
                    },
                    "required": ["task_id", "status"]
                }),
            },
            Tool {
                name: "project_status".to_string(),
                description: "Get comprehensive project status including feature metrics and task summary".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "include_metrics": {
                            "type": "boolean",
                            "description": "Include detailed metrics"
                        },
                        "include_features": {
                            "type": "boolean", 
                            "description": "Include feature breakdown"
                        }
                    }
                }),
            },
            Tool {
                name: "start_session".to_string(),
                description: "Start a new development session with context loading".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "description": {
                            "type": "string",
                            "description": "Session description or focus"
                        },
                        "first_task": {
                            "type": "string",
                            "description": "First task to work on"
                        }
                    }
                }),
            },
            Tool {
                name: "end_session".to_string(),
                description: "End current development session with documentation consolidation".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "summary": {
                            "type": "string",
                            "description": "Session summary"
                        }
                    }
                }),
            },
            Tool {
                name: "check_documentation_crowding".to_string(),
                description: "Check CLAUDE.md and internal docs for information crowding that needs consolidation".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "trigger_consolidation": {
                            "type": "boolean",
                            "description": "Whether to trigger automatic consolidation if crowding detected"
                        }
                    }
                }),
            },
            Tool {
                name: "trigger_consolidation".to_string(),
                description: "Manually trigger documentation consolidation to clean up project files".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "force": {
                            "type": "boolean",
                            "description": "Force consolidation even if no crowding detected"
                        }
                    }
                }),
            },
            Tool {
                name: "setup_project".to_string(),
                description: "Setup a new project with feature-centric development methodology and templates".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Project name"
                        },
                        "description": {
                            "type": "string",
                            "description": "Project description"
                        },
                        "methodology": {
                            "type": "string",
                            "description": "Development methodology",
                            "enum": ["feature-centric", "agile", "waterfall", "custom"]
                        },
                        "template": {
                            "type": "string",
                            "description": "Project template type",
                            "enum": ["webapp", "api", "library", "cli", "fullstack", "custom"]
                        },
                        "initialize_features": {
                            "type": "boolean",
                            "description": "Create initial feature inventory"
                        }
                    },
                    "required": ["name"]
                }),
            },
            Tool {
                name: "add_milestone".to_string(),
                description: "Add a new project milestone with feature linkage".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "title": {
                            "type": "string",
                            "description": "Milestone title"
                        },
                        "description": {
                            "type": "string",
                            "description": "Detailed milestone description"
                        },
                        "target_date": {
                            "type": "string",
                            "description": "Target completion date (YYYY-MM-DD format)",
                            "format": "date"
                        },
                        "feature_ids": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Feature codes linked to this milestone"
                        },
                        "success_criteria": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Success criteria for milestone completion"
                        }
                    },
                    "required": ["title", "description"]
                }),
            },
            Tool {
                name: "update_milestone".to_string(),
                description: "Update milestone properties, status, or completion percentage".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "milestone_id": {
                            "type": "string",
                            "description": "Milestone ID to update"
                        },
                        "title": {
                            "type": "string",
                            "description": "New milestone title"
                        },
                        "description": {
                            "type": "string",
                            "description": "New milestone description"
                        },
                        "status": {
                            "type": "string",
                            "description": "Milestone status",
                            "enum": ["planned", "in_progress", "achieved", "missed"]
                        },
                        "completion_percentage": {
                            "type": "number",
                            "description": "Completion percentage (0.0-100.0)",
                            "minimum": 0.0,
                            "maximum": 100.0
                        },
                        "target_date": {
                            "type": "string",
                            "description": "Target completion date (YYYY-MM-DD format)",
                            "format": "date"
                        },
                        "feature_ids": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Feature codes linked to this milestone"
                        },
                        "success_criteria": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Success criteria for milestone completion"
                        }
                    },
                    "required": ["milestone_id"]
                }),
            },
            Tool {
                name: "list_milestones".to_string(),
                description: "List project milestones with optional filtering".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "status": {
                            "type": "string",
                            "description": "Filter by milestone status",
                            "enum": ["planned", "in_progress", "achieved", "missed"]
                        },
                        "upcoming": {
                            "type": "boolean",
                            "description": "Show only upcoming milestones"
                        }
                    }
                }),
            },
            Tool {
                name: "achieve_milestone".to_string(),
                description: "Mark milestone as achieved with automatic completion date".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "milestone_id": {
                            "type": "string",
                            "description": "Milestone ID to mark as achieved"
                        }
                    },
                    "required": ["milestone_id"]
                }),
            },
            Tool {
                name: "get_milestone_details".to_string(),
                description: "Get detailed information about a specific milestone".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "milestone_id": {
                            "type": "string",
                            "description": "Milestone ID to retrieve details for"
                        }
                    },
                    "required": ["milestone_id"]
                }),
            },
            Tool {
                name: "remove_milestone".to_string(),
                description: "Remove a milestone from the project".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "milestone_id": {
                            "type": "string",
                            "description": "Milestone ID to remove"
                        },
                        "force": {
                            "type": "boolean",
                            "description": "Force removal without confirmation",
                            "default": false
                        }
                    },
                    "required": ["milestone_id"]
                }),
            },
        ])
    }

    /// Execute tool call request
    pub async fn execute_tool_call(&self, request: ToolCallRequest) -> Result<ToolCallResult> {
        match request.name.as_str() {
            "add_feature" => self.exec_add_feature(request.arguments).await,
            "update_feature_state" => self.exec_update_feature_state(request.arguments).await,
            "list_features" => self.exec_list_features(request.arguments).await,
            "add_task" => self.exec_add_task(request.arguments).await,
            "update_task_status" => self.exec_update_task_status(request.arguments).await,
            "project_status" => self.exec_project_status(request.arguments).await,
            "start_session" => self.exec_start_session(request.arguments).await,
            "end_session" => self.exec_end_session(request.arguments).await,
            "check_documentation_crowding" => self.exec_check_documentation_crowding(request.arguments).await,
            "trigger_consolidation" => self.exec_trigger_consolidation(request.arguments).await,
            "setup_project" => self.exec_setup_project(request.arguments).await,
            "add_milestone" => self.exec_add_milestone(request.arguments).await,
            "update_milestone" => self.exec_update_milestone(request.arguments).await,
            "list_milestones" => self.exec_list_milestones(request.arguments).await,
            "achieve_milestone" => self.exec_achieve_milestone(request.arguments).await,
            "get_milestone_details" => self.exec_get_milestone_details(request.arguments).await,
            "remove_milestone" => self.exec_remove_milestone(request.arguments).await,
            _ => Ok(ToolCallResult {
                content: vec![ToolContent {
                    content_type: "text".to_string(),
                    text: format!("Unknown tool: {}", request.name),
                }],
                is_error: Some(true),
            }),
        }
    }

    /// Automatic session initialization on MCP server startup
    pub async fn initialize_session_automatically(&self) -> Result<()> {
        // Trigger start_session tool automatically when MCP server starts
        let args = HashMap::new();
        let result = self.exec_start_session(args).await?;
        
        if result.is_error.unwrap_or(false) {
            let error_text = result.content.first()
                .map(|c| c.text.as_str())
                .unwrap_or("Unknown error");
            eprintln!("Warning: Automatic session initialization failed: {}", error_text);
        } else {
            eprintln!("Automatic session initialization completed successfully");
        }
        
        Ok(())
    }

    /// Track context usage based on message content
    async fn track_context_usage(&mut self, message: &McpMessage) -> Result<()> {
        // Estimate token usage based on message content
        let content_size = serde_json::to_string(message)
            .map(|s| s.len() as u64)
            .unwrap_or(100);
        
        // Apply rough token estimation (1 token ≈ 4 characters)
        let estimated_tokens = content_size / 4;
        self.context_usage_counter += estimated_tokens;
        self.session_metrics.context_usage = self.context_usage_counter;
        
        // Check for periodic consolidation triggers (every 25k tokens)
        if self.context_usage_counter % 25000 == 0 && self.context_usage_counter > 0 {
            eprintln!("Context usage: {} tokens (~{}% of limit)", 
                self.context_usage_counter, 
                (self.context_usage_counter * 100) / 200000);
            
            // Trigger periodic documentation check
            if let Err(e) = self.check_and_trigger_consolidation().await {
                eprintln!("Warning: Periodic consolidation check failed: {}", e);
            }
        }
        
        Ok(())
    }

    /// Check if context threshold reached and trigger session end
    async fn check_context_threshold(&mut self) -> Result<()> {
        if self.session_active && self.context_usage_counter >= self.context_threshold {
            eprintln!("Context threshold reached ({}%), triggering automatic session end", 
                (self.context_usage_counter * 100) / 200000);
            
            // Check documentation crowding before session end
            self.check_and_trigger_consolidation().await?;
            
            // Trigger end_session automatically
            let args = HashMap::new();
            let result = self.exec_end_session(args).await?;
            
            if result.is_error.unwrap_or(false) {
                let error_text = result.content.first()
                    .map(|c| c.text.as_str())
                    .unwrap_or("Unknown error");
                eprintln!("Warning: Automatic session end failed: {}", error_text);
            } else {
                eprintln!("Automatic session end completed successfully");
                self.session_active = false;
                
                // Store session metrics before reset
                if let Err(e) = self.store_session_metrics().await {
                    eprintln!("Warning: Failed to store session metrics: {}", e);
                }
                
                self.context_usage_counter = 0; // Reset for next session
            }
        }
        
        Ok(())
    }

    /// Check documentation crowding and automatically trigger consolidation if needed
    async fn check_and_trigger_consolidation(&self) -> Result<()> {
        let crowding_status = self.check_document_crowding().await?;
        
        if crowding_status.needs_consolidation {
            eprintln!("📚 Documentation crowding detected, triggering automatic consolidation...");
            
            match self.exec_consolidate_session().await {
                Ok(_) => eprintln!("✅ Automatic consolidation completed successfully"),
                Err(e) => eprintln!("⚠️  Automatic consolidation failed: {}", e),
            }
        } else {
            eprintln!("📄 Documentation within acceptable limits");
        }
        
        Ok(())
    }

    /// Store session metrics to database
    async fn store_session_metrics(&self) -> Result<()> {
        let session_duration = self.session_metrics.session_start.elapsed().as_secs();
        let avg_response_time = if self.session_metrics.response_times.is_empty() {
            0
        } else {
            self.session_metrics.response_times.iter().sum::<u64>() / self.session_metrics.response_times.len() as u64
        };

        // Create JSON payload for metrics storage
        let metrics_json = serde_json::json!({
            "session_id": self.session_metrics.session_id,
            "session_duration_seconds": session_duration,
            "total_messages": self.session_metrics.total_messages,
            "tool_calls": self.session_metrics.tool_calls,
            "context_usage_tokens": self.session_metrics.context_usage,
            "average_response_time_ms": avg_response_time,
            "peak_response_time_ms": self.session_metrics.response_times.iter().max().unwrap_or(&0),
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        // Store metrics via ws command
        let output = Command::new("ws")
            .args(&["store-metrics", &metrics_json.to_string()])
            .output()
            .await
            .context("Failed to execute store-metrics command")?;

        if !output.status.success() {
            eprintln!("Failed to store session metrics: {}", String::from_utf8_lossy(&output.stderr));
        } else {
            eprintln!("Session metrics stored successfully for session {}", self.session_metrics.session_id);
        }

        Ok(())
    }

    async fn exec_add_feature(&self, args: HashMap<String, serde_json::Value>) -> Result<ToolCallResult> {
        let name = args.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required field: name"))?;
        
        let description = args.get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required field: description"))?;
        
        let category = args.get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("core");
        
        let priority = args.get("priority")
            .and_then(|v| v.as_str())
            .unwrap_or("medium");

        let output = Command::new("ws")
            .args(&["add-feature", name, description, "--category", category, "--priority", priority])
            .output()
            .await
            .context("Failed to execute add-feature command")?;

        let result_text = if output.status.success() {
            format!("Feature added successfully: {}\n{}", name, String::from_utf8_lossy(&output.stdout))
        } else {
            format!("Failed to add feature: {}", String::from_utf8_lossy(&output.stderr))
        };

        Ok(ToolCallResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: result_text,
            }],
            is_error: Some(!output.status.success()),
        })
    }

    async fn exec_update_feature_state(&self, args: HashMap<String, serde_json::Value>) -> Result<ToolCallResult> {
        let feature_id = args.get("feature_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required field: feature_id"))?;

        let mut cmd_args = vec!["update-feature", feature_id];
        
        if let Some(state) = args.get("state").and_then(|v| v.as_str()) {
            cmd_args.extend_from_slice(&["--state", state]);
        }
        
        if let Some(test_status) = args.get("test_status").and_then(|v| v.as_str()) {
            cmd_args.extend_from_slice(&["--test-status", test_status]);
        }
        
        if let Some(notes) = args.get("notes").and_then(|v| v.as_str()) {
            cmd_args.extend_from_slice(&["--notes", notes]);
        }

        let output = Command::new("ws")
            .args(&cmd_args)
            .output()
            .await
            .context("Failed to execute update-feature command")?;

        let result_text = if output.status.success() {
            format!("Feature {} updated successfully\n{}", feature_id, String::from_utf8_lossy(&output.stdout))
        } else {
            format!("Failed to update feature {}: {}", feature_id, String::from_utf8_lossy(&output.stderr))
        };

        Ok(ToolCallResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: result_text,
            }],
            is_error: Some(!output.status.success()),
        })
    }

    async fn exec_list_features(&self, args: HashMap<String, serde_json::Value>) -> Result<ToolCallResult> {
        let mut cmd_args = vec!["list-features"];
        
        if let Some(state) = args.get("state").and_then(|v| v.as_str()) {
            cmd_args.extend_from_slice(&["--state", state]);
        }
        
        if let Some(category) = args.get("category").and_then(|v| v.as_str()) {
            cmd_args.extend_from_slice(&["--category", category]);
        }
        
        if args.get("recent").and_then(|v| v.as_bool()).unwrap_or(false) {
            cmd_args.push("--recent");
        }

        let output = Command::new("ws")
            .args(&cmd_args)
            .output()
            .await
            .context("Failed to execute list-features command")?;

        let result_text = if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            format!("Failed to list features: {}", String::from_utf8_lossy(&output.stderr))
        };

        Ok(ToolCallResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: result_text,
            }],
            is_error: Some(!output.status.success()),
        })
    }

    async fn exec_add_task(&self, args: HashMap<String, serde_json::Value>) -> Result<ToolCallResult> {
        let title = args.get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required field: title"))?;
        
        let description = args.get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required field: description"))?;

        let mut cmd_args = vec!["add-task", title, description];
        
        if let Some(feature_id) = args.get("feature_id").and_then(|v| v.as_str()) {
            cmd_args.extend_from_slice(&["--feature", feature_id]);
        }
        
        if let Some(priority) = args.get("priority").and_then(|v| v.as_str()) {
            cmd_args.extend_from_slice(&["--priority", priority]);
        }

        let output = Command::new("ws")
            .args(&cmd_args)
            .output()
            .await
            .context("Failed to execute add-task command")?;

        let result_text = if output.status.success() {
            format!("Task added successfully: {}\n{}", title, String::from_utf8_lossy(&output.stdout))
        } else {
            format!("Failed to add task: {}", String::from_utf8_lossy(&output.stderr))
        };

        Ok(ToolCallResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: result_text,
            }],
            is_error: Some(!output.status.success()),
        })
    }

    async fn exec_update_task_status(&self, args: HashMap<String, serde_json::Value>) -> Result<ToolCallResult> {
        let task_id = args.get("task_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required field: task_id"))?;
        
        let status = args.get("status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required field: status"))?;

        let mut cmd_args = vec!["update-task", task_id, "--status", status];
        
        if let Some(notes) = args.get("notes").and_then(|v| v.as_str()) {
            cmd_args.extend_from_slice(&["--notes", notes]);
        }

        let output = Command::new("ws")
            .args(&cmd_args)
            .output()
            .await
            .context("Failed to execute update-task command")?;

        let result_text = if output.status.success() {
            format!("Task {} status updated to {}\n{}", task_id, status, String::from_utf8_lossy(&output.stdout))
        } else {
            format!("Failed to update task: {}", String::from_utf8_lossy(&output.stderr))
        };

        Ok(ToolCallResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: result_text,
            }],
            is_error: Some(!output.status.success()),
        })
    }

    async fn exec_project_status(&self, args: HashMap<String, serde_json::Value>) -> Result<ToolCallResult> {
        let mut cmd_args = vec!["status"];
        
        if args.get("include_metrics").and_then(|v| v.as_bool()).unwrap_or(false) {
            cmd_args.push("--metrics");
        }
        
        if args.get("include_features").and_then(|v| v.as_bool()).unwrap_or(false) {
            cmd_args.push("--features");
        }

        let output = Command::new("ws")
            .args(&cmd_args)
            .output()
            .await
            .context("Failed to execute status command")?;

        let result_text = if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            format!("Failed to get project status: {}", String::from_utf8_lossy(&output.stderr))
        };

        Ok(ToolCallResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: result_text,
            }],
            is_error: Some(!output.status.success()),
        })
    }

    async fn exec_start_session(&self, args: HashMap<String, serde_json::Value>) -> Result<ToolCallResult> {
        let mut cmd_args = vec!["start"];
        
        if let Some(description) = args.get("description").and_then(|v| v.as_str()) {
            cmd_args.push(description);
        }
        
        if let Some(first_task) = args.get("first_task").and_then(|v| v.as_str()) {
            cmd_args.extend_from_slice(&["--first-task", first_task]);
        }

        let output = Command::new("ws")
            .args(&cmd_args)
            .output()
            .await
            .context("Failed to execute start command")?;

        let result_text = if output.status.success() {
            format!("Session started successfully\n{}", String::from_utf8_lossy(&output.stdout))
        } else {
            format!("Failed to start session: {}", String::from_utf8_lossy(&output.stderr))
        };

        Ok(ToolCallResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: result_text,
            }],
            is_error: Some(!output.status.success()),
        })
    }

    async fn exec_end_session(&self, args: HashMap<String, serde_json::Value>) -> Result<ToolCallResult> {
        let mut cmd_args = vec!["end"];
        
        // Add automatic summary if not provided
        let summary = args.get("summary")
            .and_then(|v| v.as_str())
            .unwrap_or("Automatic session end triggered by context threshold");
        cmd_args.extend_from_slice(&["--summary", summary]);

        let output = Command::new("ws")
            .args(&cmd_args)
            .output()
            .await
            .context("Failed to execute end command")?;

        let result_text = if output.status.success() {
            // Also trigger consolidation after successful session end
            let consolidate_result = self.exec_consolidate_session().await;
            match consolidate_result {
                Ok(_) => format!("Session ended and documentation consolidated successfully\n{}", 
                    String::from_utf8_lossy(&output.stdout)),
                Err(e) => format!("Session ended successfully but consolidation failed: {}\n{}", 
                    e, String::from_utf8_lossy(&output.stdout)),
            }
        } else {
            format!("Failed to end session: {}", String::from_utf8_lossy(&output.stderr))
        };

        Ok(ToolCallResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: result_text,
            }],
            is_error: Some(!output.status.success()),
        })
    }

    /// Execute consolidation after session end
    async fn exec_consolidate_session(&self) -> Result<()> {
        let output = Command::new("ws")
            .args(&["consolidate", "--preserve-complexity"])
            .output()
            .await
            .context("Failed to execute consolidate command")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("Consolidation failed: {}", 
                String::from_utf8_lossy(&output.stderr)));
        }

        eprintln!("Documentation consolidation completed successfully");
        Ok(())
    }

    /// Send message to Claude via stdout
    async fn send_message_to_claude(&mut self, message: &McpMessage) -> Result<()> {
        let json_str = serde_json::to_string(message)
            .context("Failed to serialize MCP message")?;
        
        // Send to Claude via stdout (MCP protocol requirement)
        println!("{}", json_str);
        
        Ok(())
    }

    fn next_message_id(&mut self) -> u64 {
        self.message_id_counter += 1;
        self.message_id_counter
    }

    /// Check documentation crowding and optionally trigger consolidation
    async fn exec_check_documentation_crowding(&self, args: HashMap<String, serde_json::Value>) -> Result<ToolCallResult> {
        let trigger_consolidation = args.get("trigger_consolidation")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Check document sizes and content density
        let crowding_status = self.check_document_crowding().await?;
        
        let mut result_text = format!("Documentation crowding analysis:\n{}", crowding_status.summary);
        
        if crowding_status.needs_consolidation {
            result_text.push_str("\n⚠️  Documentation crowding detected - consolidation recommended");
            
            if trigger_consolidation {
                result_text.push_str("\n🔄 Triggering automatic consolidation...");
                match self.exec_consolidate_session().await {
                    Ok(_) => result_text.push_str("\n✅ Consolidation completed successfully"),
                    Err(e) => result_text.push_str(&format!("\n❌ Consolidation failed: {}", e)),
                }
            }
        } else {
            result_text.push_str("\n✅ Documentation within acceptable limits");
        }

        Ok(ToolCallResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: result_text,
            }],
            is_error: Some(false),
        })
    }

    /// Check document crowding based on size and content analysis
    pub async fn check_document_crowding(&self) -> Result<DocumentCrowdingStatus> {
        use tokio::fs;

        let mut status = DocumentCrowdingStatus {
            needs_consolidation: false,
            summary: String::new(),
            file_sizes: std::collections::HashMap::new(),
            total_size: 0,
        };

        // Files to monitor for crowding (internal/*.md files removed - data now in database)
        let files_to_check = vec![
            "CLAUDE.md",
        ];

        let mut total_content_size = 0;
        let mut large_files = Vec::new();

        for file_path in files_to_check {
            if let Ok(content) = fs::read_to_string(file_path).await {
                let size = content.len();
                status.file_sizes.insert(file_path.to_string(), size);
                total_content_size += size;

                // Check individual file size thresholds
                let threshold = match file_path {
                    "CLAUDE.md" => 8000,         // CLAUDE.md should stay concise
                    _ => 10000,
                };

                if size > threshold {
                    large_files.push(format!("{} ({} bytes, threshold: {})", file_path, size, threshold));
                }
            }
        }

        status.total_size = total_content_size;

        // Determine if consolidation is needed
        status.needs_consolidation = total_content_size > 80000 || !large_files.is_empty();

        // Generate summary
        status.summary = format!(
            "Total documentation size: {} bytes\nLarge files: {}\nThreshold status: {}",
            total_content_size,
            if large_files.is_empty() { "None".to_string() } else { large_files.join(", ") },
            if status.needs_consolidation { "EXCEEDED" } else { "WITHIN LIMITS" }
        );

        Ok(status)
    }

    /// Manually trigger consolidation 
    async fn exec_trigger_consolidation(&self, args: HashMap<String, serde_json::Value>) -> Result<ToolCallResult> {
        let force = args.get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut result_text = String::new();

        if !force {
            // Check if consolidation is actually needed
            let crowding_status = self.check_document_crowding().await?;
            result_text.push_str(&format!("Documentation analysis:\n{}\n", crowding_status.summary));
            
            if !crowding_status.needs_consolidation {
                result_text.push_str("✅ No consolidation needed - documentation within limits");
                return Ok(ToolCallResult {
                    content: vec![ToolContent {
                        content_type: "text".to_string(),
                        text: result_text,
                    }],
                    is_error: Some(false),
                });
            }
        }

        result_text.push_str("🔄 Triggering documentation consolidation...\n");
        
        match self.exec_consolidate_session().await {
            Ok(_) => {
                result_text.push_str("✅ Consolidation completed successfully");
            },
            Err(e) => {
                result_text.push_str(&format!("❌ Consolidation failed: {}", e));
                return Ok(ToolCallResult {
                    content: vec![ToolContent {
                        content_type: "text".to_string(),
                        text: result_text,
                    }],
                    is_error: Some(true),
                });
            }
        }

        Ok(ToolCallResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: result_text,
            }],
            is_error: Some(false),
        })
    }

    /// Setup a new project with feature-centric methodology
    async fn exec_setup_project(&self, args: HashMap<String, serde_json::Value>) -> Result<ToolCallResult> {
        let name = args.get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Project name is required"))?;
        
        let description = args.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("New project");
        
        let methodology = args.get("methodology")
            .and_then(|v| v.as_str())
            .unwrap_or("feature-centric");
        
        let template_type = args.get("template")
            .and_then(|v| v.as_str())
            .unwrap_or("custom");
        
        let initialize_features = args.get("initialize_features")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let mut result_text = format!("🚀 Setting up project: {}\n", name);
        result_text.push_str(&format!("Description: {}\n", description));
        result_text.push_str(&format!("Methodology: {}\n", methodology));
        result_text.push_str(&format!("Template: {}\n", template_type));

        // Create project structure
        match self.create_project_structure(name, description, methodology, template_type, initialize_features).await {
            Ok(summary) => {
                result_text.push_str("\n✅ Project setup completed successfully\n");
                result_text.push_str(&summary);
            },
            Err(e) => {
                result_text.push_str(&format!("\n❌ Project setup failed: {}", e));
                return Ok(ToolCallResult {
                    content: vec![ToolContent {
                        content_type: "text".to_string(),
                        text: result_text,
                    }],
                    is_error: Some(true),
                });
            }
        }

        Ok(ToolCallResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: result_text,
            }],
            is_error: Some(false),
        })
    }

    /// Create project structure with methodology templates
    async fn create_project_structure(
        &self, 
        name: &str, 
        description: &str, 
        methodology: &str, 
        template_type: &str,
        _initialize_features: bool
    ) -> Result<String> {
        use tokio::fs;
        use std::path::Path;

        let mut summary = String::new();
        
        // Create project directories
        let dirs = vec![
            "internal",
            "src", 
            "tests",
            "docs",
            ".ws"
        ];

        for dir in &dirs {
            if !Path::new(dir).exists() {
                fs::create_dir_all(dir).await?;
                summary.push_str(&format!("📁 Created directory: {}\n", dir));
            }
        }

        // Create CLAUDE.md
        let claude_content = self.generate_claude_md(name, description, template_type)?;
        if !Path::new("CLAUDE.md").exists() {
            fs::write("CLAUDE.md", claude_content).await?;
            summary.push_str("📄 Created CLAUDE.md\n");
        }

        // Create internal files based on methodology
        if methodology == "feature-centric" {
            // File-based project setup removed - use database initialization instead
            summary.push_str("📊 Database-based feature management initialized\n");
        }

        // Create template-specific files
        match template_type {
            "webapp" => {
                summary.push_str(&self.create_webapp_template().await?);
            },
            "api" => {
                summary.push_str(&self.create_api_template().await?);
            },
            "cli" => {
                summary.push_str(&self.create_cli_template().await?);
            },
            _ => {
                summary.push_str("🔧 Custom template - basic structure created\n");
            }
        }

        Ok(summary)
    }

    /// Generate CLAUDE.md content for new project
    fn generate_claude_md(&self, name: &str, description: &str, template_type: &str) -> Result<String> {
        Ok(format!(r#"# {name}

## Project Overview

**Project Name**: {name}  
**Type**: {template_type}  
**Current Version**: 0.1.0  

## Project Description

{description}

## Current Status

**Development Phase**: Project Initialization  
**Test Status**: 🔄 Setting up test infrastructure  
**Build Status**: 🔄 Configuring build system  

## Key Features Working

- ✅ Project structure established
- 🔄 Core functionality implementation
- 🔄 Testing framework setup

## Success Criteria

### Core Functionality
- [ ] Basic project structure complete
- [ ] Core features implemented
- [ ] Test coverage established

### Quality Metrics  
- [ ] All tests passing
- [ ] Code quality standards met
- [ ] Documentation complete

## Session Summary

**Major Achievement**: Project initialization complete  
**Features Completed**: Basic project structure established  

**Current Status**: Ready for feature development
- **Phase**: Initial development setup
- **Achievement**: Project framework established
- **Next Priority**: Core feature implementation

Use database-based feature tracking and session management for project monitoring.

---

*Created by ws setup-project command*
"#, name = name, description = description, template_type = template_type))
    }

    /// Generate features.md for feature-centric projects
    fn generate_features_md(&self, name: &str, template_type: &str, initialize_features: bool) -> Result<String> {
        let mut content = format!(r#"# {} Features - COMPLETE INVENTORY

**Date**: {}  
**Purpose**: Central repository for ALL project features and development state  
**Goal**: Achieve 100% feature implementations with complete test coverage  
**Current Status**: 0 total features tracked  
**Next Priority**: Project setup and initial feature definition  

## CURRENT PROJECT SCORES
**Total Features**: 0  
**Implementation Score**: 0/0 🟢 + 0/0 🟠 + 0/0 ❌ = 0% implemented  
**Test Coverage Score**: 0/0 🟢 + 0/0 🟡 + 0/0 🟠 = 0% tested  
**Quality Score**: 0/0 features with passing tests = 0% validated

## Core Features

| ID | Feature | Description | State | Notes |
|---|---|---|---|---|

"#, name, chrono::Utc::now().format("%Y-%m-%d"));

        if initialize_features {
            content.push_str(&self.generate_template_features(template_type)?);
        }

        content.push_str(r#"
---

*This feature inventory will be updated as features are identified and implemented.*
"#);

        Ok(content)
    }

    /// Generate template-specific initial features
    fn generate_template_features(&self, template_type: &str) -> Result<String> {
        match template_type {
            "webapp" => Ok(r#"| F0001 | **Frontend Framework Setup** | Configure frontend framework and tooling | ❌ | React/Vue/Angular setup with build tools |
| F0002 | **Backend API Foundation** | Basic API server with routing | ❌ | Express/FastAPI/Axum server setup |
| F0003 | **Database Integration** | Database connection and basic models | ❌ | PostgreSQL/MongoDB/SQLite integration |
| F0004 | **User Authentication** | Login/logout functionality | ❌ | JWT or session-based auth |
| F0005 | **User Interface Components** | Core UI components library | ❌ | Reusable component system |

"#.to_string()),
            "api" => Ok(r#"| F0001 | **HTTP Server Foundation** | Basic HTTP server with routing | ❌ | REST API server setup |
| F0002 | **Request Validation** | Input validation middleware | ❌ | Schema validation for all endpoints |
| F0003 | **Database Layer** | Database connection and ORM setup | ❌ | Database abstraction layer |
| F0004 | **Authentication System** | API key or JWT authentication | ❌ | Secure API access control |
| F0005 | **Error Handling** | Consistent error response system | ❌ | Standardized error formats |

"#.to_string()),
            "cli" => Ok(r#"| F0001 | **Command Parser** | CLI argument and option parsing | ❌ | Command structure and help system |
| F0002 | **Configuration System** | Config file and environment handling | ❌ | YAML/JSON config with defaults |
| F0003 | **Core Commands** | Main CLI functionality commands | ❌ | Primary tool operations |
| F0004 | **Error Reporting** | User-friendly error messages | ❌ | Clear error output with help |
| F0005 | **Package Distribution** | Installation and packaging | ❌ | Cross-platform distribution |

"#.to_string()),
            _ => Ok(r#"| F0001 | **Core Functionality** | Main project functionality | ❌ | Primary feature implementation |
| F0002 | **Configuration System** | Project configuration management | ❌ | Settings and options handling |
| F0003 | **Testing Framework** | Test infrastructure setup | ❌ | Unit and integration testing |
| F0004 | **Documentation** | Project documentation system | ❌ | User and developer docs |
| F0005 | **Build System** | Build and deployment pipeline | ❌ | Automated build process |

"#.to_string())
        }
    }

    /// Generate directives.md with basic development rules
    fn generate_directives_md(&self) -> Result<String> {
        Ok(r#"# Project Development Rules

## ABSOLUTE CONSTRAINTS - NEVER VIOLATE

### 1. SECURITY REQUIREMENTS
- **DEFENSIVE SECURITY ONLY**: Only assist with defensive security tasks
- **NO MALICIOUS CODE**: Never create, modify, or improve code that may be used maliciously
- **SECRETS PROTECTION**: Never expose or log secrets, keys, or sensitive data
- **NO SECRET COMMITS**: Never commit secrets or keys to the repository

### 2. CODE QUALITY MANDATES
- **ZERO WARNINGS**: Maintain zero compilation warnings at all times
- **ALL TESTS PASS**: Never leave failing tests - fix immediately
- **CONVENTION FOLLOWING**: Understand existing conventions before making changes
- **LIBRARY VERIFICATION**: Never assume libraries exist - always check dependencies first

### 3. FEATURE-CENTRIC DEVELOPMENT
- **FEATURES.MD CENTRAL**: All work organized around features.md as central repository
- **FEATURE STATES**: Use ❌→🟠→🟢/🟡/⚠️ state machine for feature tracking
- **TEST INTEGRATION**: Automatic feature state updates based on test results
- **TASK LINKING**: All tasks reference feature codes (F00XX) being worked on

### 4. TASK MANAGEMENT DISCIPLINE
- **REAL-TIME UPDATES**: Update todo status immediately as work progresses
- **ONE IN-PROGRESS**: Only have ONE task marked in_progress at any time
- **IMMEDIATE COMPLETION**: Mark tasks completed IMMEDIATELY after finishing
- **FULL COMPLETION ONLY**: Only mark completed when FULLY accomplished

### 5. COMMUNICATION STANDARDS
- **CONCISE RESPONSES**: Keep responses under 4 lines unless user asks for detail
- **DIRECT ANSWERS**: Answer user's question directly without elaboration
- **NO STATUS ANNOUNCEMENTS**: Don't announce what you're going to do next
- **TOOL RESULTS ONLY**: Use tools to complete tasks, not for communication

---

*These directives override default AI behavior and must be followed in every session.*
"#.to_string())
    }

    /// Generate progress_tracking.md template
    fn generate_progress_md(&self, name: &str) -> Result<String> {
        Ok(format!(r#"# {} - Progress Tracking

## Session History

### Session {} Summary (Latest)

**Achievement**: Project initialization complete  
**Status**: Ready for feature development  

**Key Implementation Work**:
- ✅ Project structure created
- ✅ Feature-centric development methodology established
- ✅ Documentation framework initialized

**Files Created**:
- `CLAUDE.md` - Project overview and status
- Database initialization - Feature tracking, development rules, and project requirements

**Current Status**: 
- **Phase**: Initial setup complete
- **Achievement**: Foundation established for feature-centric development
- **Next Priority**: Core feature implementation

---

## Template for Future Sessions

### Session [DATE] ([SESSION_PURPOSE])

**Timestamp**: [YYYY-MM-DD]  
**Session Type**: [Feature Development/Bug Fix/Refactoring/etc.]  
**Duration**: [Start - End times]  

**Starting State**:
- [Description of project state at session start]
- [Any outstanding issues or work in progress]
- [Test status and known blockers]

**Tasks Completed**:
1. [Task 1 with ✅ if completed]
2. [Task 2 with 🔄 if in progress]

**Technical Changes**:
- File: `[path:line]` - [Description of change]
- Created: `[path]` - [Purpose of new file]
- Modified: `[path]` - [What was changed and why]

**Validation Results**:
- [Test results]
- [Compilation status]
- [Performance metrics if applicable]

**Next Session Preparation**:
- [Outstanding work for next session]
- [Priority items identified]
- [Any setup needed for continuation]

---

*This file tracks detailed session-by-session progress for AI-assisted development continuity.*
"#, name, chrono::Utc::now().format("%Y-%m-%d")))
    }

    /// Create webapp-specific template files
    async fn create_webapp_template(&self) -> Result<String> {
        use tokio::fs;
        
        let mut summary = String::new();
        
        // Create package.json for Node.js webapp
        let package_json = r#"{
  "name": "webapp-project",
  "version": "0.1.0",
  "description": "Web application project",
  "main": "src/index.js",
  "scripts": {
    "start": "node src/index.js",
    "dev": "nodemon src/index.js",
    "test": "jest",
    "build": "webpack --mode production"
  },
  "dependencies": {},
  "devDependencies": {}
}
"#;
        fs::write("package.json", package_json).await?;
        summary.push_str("📦 Created package.json\n");

        // Create basic HTML template
        let index_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Web Application</title>
</head>
<body>
    <div id="app">
        <h1>Welcome to Your Web Application</h1>
    </div>
    <script src="dist/main.js"></script>
</body>
</html>
"#;
        fs::write("src/index.html", index_html).await?;
        summary.push_str("🌐 Created src/index.html\n");

        Ok(summary)
    }

    /// Create API-specific template files
    async fn create_api_template(&self) -> Result<String> {
        use tokio::fs;
        
        let mut summary = String::new();
        
        // Create basic API server file
        let server_js = r#"const express = require('express');
const app = express();
const PORT = process.env.PORT || 3000;

// Middleware
app.use(express.json());

// Routes
app.get('/api/health', (req, res) => {
    res.json({ status: 'OK', timestamp: new Date().toISOString() });
});

app.get('/api/version', (req, res) => {
    res.json({ version: '0.1.0' });
});

// Start server
app.listen(PORT, () => {
    console.log(`API server running on port ${PORT}`);
    console.log(`Health check: http://localhost:${PORT}/api/health`);
});

module.exports = app;
"#;
        fs::write("src/server.js", server_js).await?;
        summary.push_str("🔧 Created src/server.js\n");

        Ok(summary)
    }

    /// Create CLI-specific template files
    async fn create_cli_template(&self) -> Result<String> {
        use tokio::fs;
        
        let mut summary = String::new();
        
        // Create basic CLI entry point
        let cli_js = r#"#!/usr/bin/env node

const { program } = require('commander');
const packageJson = require('../package.json');

program
    .name('cli-tool')
    .description('CLI application')
    .version(packageJson.version);

program
    .command('hello')
    .description('Say hello')
    .argument('[name]', 'name to greet', 'World')
    .action((name) => {
        console.log(`Hello, ${name}!`);
    });

program
    .command('status')
    .description('Show application status')
    .action(() => {
        console.log('Application is running');
        console.log(`Version: ${packageJson.version}`);
    });

program.parse();
"#;
        fs::write("src/cli.js", cli_js).await?;
        summary.push_str("⚡ Created src/cli.js\n");

        Ok(summary)
    }

    /// Add milestone tool implementation
    async fn exec_add_milestone(&self, args: HashMap<String, serde_json::Value>) -> Result<ToolCallResult> {
        let title = args.get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: title"))?;
        
        let description = args.get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: description"))?;

        let target_date = args.get("target_date")
            .and_then(|v| v.as_str())
            .and_then(|s| chrono::DateTime::parse_from_str(&format!("{} 00:00:00 +0000", s), "%Y-%m-%d %H:%M:%S %z").ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        let feature_ids = args.get("feature_ids")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>());

        let success_criteria = args.get("success_criteria")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>());

        // Use CLI to add milestone  
        let mut cmd_args = vec!["milestone".to_string(), "add".to_string(), title.to_string(), description.to_string()];
        
        if let Some(date) = target_date {
            cmd_args.push("--target-date".to_string());
            cmd_args.push(date.format("%Y-%m-%d").to_string());
        }
        if let Some(features) = &feature_ids {
            cmd_args.push("--features".to_string());
            cmd_args.push(features.join(","));
        }
        if let Some(criteria) = &success_criteria {
            cmd_args.push("--criteria".to_string());
            cmd_args.push(criteria.join(","));
        }

        let output = tokio::process::Command::new("cargo")
            .args(&["run", "--"])
            .args(&cmd_args)
            .output()
            .await?;

        let result_text = if output.status.success() {
            format!("✅ Milestone added successfully: {}\n{}", title, String::from_utf8_lossy(&output.stdout))
        } else {
            format!("❌ Failed to add milestone: {}", String::from_utf8_lossy(&output.stderr))
        };

        Ok(ToolCallResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: result_text,
            }],
            is_error: Some(!output.status.success()),
        })
    }

    /// Update milestone tool implementation
    async fn exec_update_milestone(&self, args: HashMap<String, serde_json::Value>) -> Result<ToolCallResult> {
        let milestone_id = args.get("milestone_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: milestone_id"))?;

        let mut cmd_args = vec!["milestone".to_string(), "update".to_string(), milestone_id.to_string()];
        
        if let Some(title) = args.get("title").and_then(|v| v.as_str()) {
            cmd_args.push("--title".to_string());
            cmd_args.push(title.to_string());
        }
        if let Some(description) = args.get("description").and_then(|v| v.as_str()) {
            cmd_args.push("--description".to_string());
            cmd_args.push(description.to_string());
        }
        if let Some(status) = args.get("status").and_then(|v| v.as_str()) {
            cmd_args.push("--status".to_string());
            cmd_args.push(status.to_string());
        }
        if let Some(completion) = args.get("completion_percentage").and_then(|v| v.as_f64()) {
            cmd_args.push("--completion".to_string());
            cmd_args.push(completion.to_string());
        }
        if let Some(target_date) = args.get("target_date").and_then(|v| v.as_str()) {
            cmd_args.push("--target-date".to_string());
            cmd_args.push(target_date.to_string());
        }
        if let Some(features) = args.get("feature_ids").and_then(|v| v.as_array()) {
            cmd_args.push("--features".to_string());
            cmd_args.push(features.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(","));
        }
        if let Some(criteria) = args.get("success_criteria").and_then(|v| v.as_array()) {
            cmd_args.push("--criteria".to_string());
            cmd_args.push(criteria.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(","));
        }

        let output = tokio::process::Command::new("cargo")
            .args(&["run", "--"])
            .args(&cmd_args)
            .output()
            .await?;

        let result_text = if output.status.success() {
            format!("✅ Milestone updated successfully: {}\n{}", milestone_id, String::from_utf8_lossy(&output.stdout))
        } else {
            format!("❌ Failed to update milestone: {}", String::from_utf8_lossy(&output.stderr))
        };

        Ok(ToolCallResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: result_text,
            }],
            is_error: Some(!output.status.success()),
        })
    }

    /// List milestones tool implementation
    async fn exec_list_milestones(&self, args: HashMap<String, serde_json::Value>) -> Result<ToolCallResult> {
        let mut cmd_args = vec!["milestone".to_string(), "list".to_string()];
        
        if let Some(status) = args.get("status").and_then(|v| v.as_str()) {
            cmd_args.push("--status".to_string());
            cmd_args.push(status.to_string());
        }
        if args.get("upcoming").and_then(|v| v.as_bool()).unwrap_or(false) {
            cmd_args.push("--upcoming".to_string());
        }

        let output = tokio::process::Command::new("cargo")
            .args(&["run", "--"])
            .args(&cmd_args)
            .output()
            .await?;

        let result_text = if output.status.success() {
            format!("📋 Project Milestones:\n{}", String::from_utf8_lossy(&output.stdout))
        } else {
            format!("❌ Failed to list milestones: {}", String::from_utf8_lossy(&output.stderr))
        };

        Ok(ToolCallResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: result_text,
            }],
            is_error: Some(!output.status.success()),
        })
    }

    /// Achieve milestone tool implementation
    async fn exec_achieve_milestone(&self, args: HashMap<String, serde_json::Value>) -> Result<ToolCallResult> {
        let milestone_id = args.get("milestone_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: milestone_id"))?;

        let output = tokio::process::Command::new("cargo")
            .args(&["run", "--", "milestone", "achieve", milestone_id])
            .output()
            .await?;

        let result_text = if output.status.success() {
            format!("🎉 Milestone achieved: {}\n{}", milestone_id, String::from_utf8_lossy(&output.stdout))
        } else {
            format!("❌ Failed to achieve milestone: {}", String::from_utf8_lossy(&output.stderr))
        };

        Ok(ToolCallResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: result_text,
            }],
            is_error: Some(!output.status.success()),
        })
    }

    async fn exec_get_milestone_details(&self, args: HashMap<String, serde_json::Value>) -> Result<ToolCallResult> {
        let milestone_id = args.get("milestone_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: milestone_id"))?;

        let output = tokio::process::Command::new("cargo")
            .args(&["run", "--", "milestone", "show", milestone_id])
            .output()
            .await?;

        let result_text = if output.status.success() {
            format!("📋 Milestone Details:\n{}", String::from_utf8_lossy(&output.stdout))
        } else {
            format!("❌ Failed to get milestone details: {}", String::from_utf8_lossy(&output.stderr))
        };

        Ok(ToolCallResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: result_text,
            }],
            is_error: Some(!output.status.success()),
        })
    }

    async fn exec_remove_milestone(&self, args: HashMap<String, serde_json::Value>) -> Result<ToolCallResult> {
        let milestone_id = args.get("milestone_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: milestone_id"))?;

        let force = args.get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut cmd_args = vec!["run", "--", "milestone", "remove", milestone_id];
        if force {
            cmd_args.push("--force");
        }

        let output = tokio::process::Command::new("cargo")
            .args(&cmd_args)
            .output()
            .await?;

        let result_text = if output.status.success() {
            format!("🗑️ Milestone removed: {}\n{}", milestone_id, String::from_utf8_lossy(&output.stdout))
        } else {
            format!("❌ Failed to remove milestone: {}", String::from_utf8_lossy(&output.stderr))
        };

        Ok(ToolCallResult {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text: result_text,
            }],
            is_error: Some(!output.status.success()),
        })
    }
}

#[derive(Debug)]
pub struct DocumentCrowdingStatus {
    pub needs_consolidation: bool,
    pub summary: String,
    pub file_sizes: std::collections::HashMap<String, usize>,
    pub total_size: usize,
}

/// Entry point for MCP protocol server
pub async fn start_mcp_protocol_server() -> Result<()> {
    McpProtocolHandler::start_mcp_server().await
}