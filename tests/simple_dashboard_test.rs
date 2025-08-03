// Simple test to validate F0097 HTTP Dashboard System core functionality
use anyhow::Result;
use std::time::Duration;
use tokio::time::timeout;

/// Test that the MCP server starts and serves basic endpoints
#[tokio::test]
async fn test_http_server_basic_functionality() -> Result<()> {
    let port = 3004; // Use unique port to avoid conflicts
    
    // Start server in background
    let server_handle = tokio::spawn(async move {
        workspace::mcp_server::start_mcp_server(port, false).await
    });
    
    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Test health endpoint
    let client = reqwest::Client::new();
    let response = timeout(
        Duration::from_secs(5),
        client.get(&format!("http://localhost:{}/health", port)).send()
    ).await??;
    
    assert_eq!(response.status(), 200);
    
    let health_data: serde_json::Value = response.json().await?;
    assert_eq!(health_data["status"], "healthy");
    assert_eq!(health_data["service"], "ws-mcp-server");
    
    // Test dashboard HTML serving
    let dashboard_response = timeout(
        Duration::from_secs(5),
        client.get(&format!("http://localhost:{}/", port)).send()
    ).await??;
    
    assert_eq!(dashboard_response.status(), 200);
    let content_type = dashboard_response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/html"));
    
    // Test static JavaScript file
    let js_response = timeout(
        Duration::from_secs(5),
        client.get(&format!("http://localhost:{}/dashboard/app.js", port)).send()
    ).await??;
    
    assert_eq!(js_response.status(), 200);
    let js_content_type = js_response.headers().get("content-type").unwrap();
    assert!(js_content_type.to_str().unwrap().contains("application/javascript"));
    
    // Clean shutdown
    server_handle.abort();
    
    Ok(())
}