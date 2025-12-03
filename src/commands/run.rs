use crate::result::Result;
use crate::utils::process::ProcessManager;

pub async fn execute(server_path: Option<&str>) -> Result<()> {
    log::info!("Starting server with path: {:?}", server_path);
    let mut process_manager = ProcessManager::new();
    let result = process_manager
        .exec_server(vec![], server_path.map(|s| s.to_string()))
        .await;

    match &result {
        Ok(_) => log::info!("Server executed successfully"),
        Err(e) => log::error!("Server execution failed: {}", e),
    }

    result
}
