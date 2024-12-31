use std::io::Write;
use clap::Parser;
use log::{self, info, error};
use walkdir::WalkDir;
use std::path::PathBuf;
use std::env;

#[derive(Parser, Debug)]
#[command(
    about,
    version,
    after_help = "
    You can specify the chain id by alias or by id, supported aliases are:
    eth: Ethereum
    base: Base

    Note: You need to set the correct API_KEY environment variable.
    You can get an API key from:
    https://etherscan.io/apis
    https://basescan.org/
    "
)]
struct Args {
    /// Chain id or alias, for more info see the help
    chain: String,
    /// Address of the contract to clone
    address: String,
    /// Path to clone the contract to
    path: String,
}

fn chain_to_id(key: &str) -> Option<i32> {
    match key.to_lowercase().as_str() {
        "eth" => Some(1),
        "base" => Some(8453),
        _ => None,
    }
}

fn build_url(chainid: i32, address: &str, api_key: &str) -> String {
    format!(
        "https://api.basescan.org/api?chainid={}&module=contract&action=getsourcecode&address={}&apikey={}",
        chainid, address, api_key
    )
}

async fn fetch_contract_source(url: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let res = client.get(url).send().await?;
    let body = res.text().await?;
    
    let json: serde_json::Value = serde_json::from_str(&body)?;
    
    // Check if the API request was successful
    if let Some(status) = json["status"].as_str() {
        if status != "1" {
            let message = json["message"].as_str().unwrap_or("Unknown error");
            let result = json["result"].as_str().unwrap_or("No additional info");
            error!("API Error: {} - {}", message, result);
            return Err(format!("Etherscan API error: {} - {}", message, result).into());
        }
    }

    Ok(json)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .format_target(false)
        .format_timestamp(None)
        .init();
    
    let args = Args::parse();

    // Get API key from environment
    let api_key = env::var("ETHERSCAN_API_KEY")
        .expect("ETHERSCAN_API_KEY environment variable not set");

    // Check if supplied dir exists
    let path = PathBuf::from(&args.path);
    if path.exists() {
        error!("Path {} already exists", args.path);
        panic!("Path already exists");
    }
    
    // Create the directory
    std::fs::create_dir(&path)?;
    info!("Created directory: {}", args.path);

    // Parse the chain id
    let chainid = match args.chain.parse::<i32>() {
        Ok(id) => id,
        Err(_) => chain_to_id(&args.chain).expect("Invalid chain id"),
    };

    info!("Chain id: {}", chainid);
    info!("Cloning contract at address {} to path {}", args.address, args.path);

    // Initialize forge project
    let output = std::process::Command::new("forge")
        .arg("init")
        .arg(&args.path)
        .arg("--no-commit")
        .output()?;

    if !output.status.success() {
        error!("Failed to initialize forge project: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Forge initialization failed");
    }
    info!("Initialized forge project");

    // Find and remove Counter files
    let src_path = path.join("src");
    info!("Searching for Counter files in: {:?}", src_path);
    
    for entry in WalkDir::new(&src_path) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() && path.to_string_lossy().contains("Counter") {
                    info!("Removing Counter file: {:?}", path);
                    std::fs::remove_file(path)?;
                }
            }
            Err(e) => error!("Error walking directory: {}", e),
        }
    }

    // Fetch contract source
    let url = build_url(chainid, &args.address, &api_key);
    info!("Fetching contract from Etherscan...");
    
    let json = fetch_contract_source(&url).await?;
    
    let result = json["result"].as_array()
        .ok_or("No result array in response")?;
    
    let source_code = result[0]["SourceCode"].as_str()
        .ok_or("No source code in response")?;

    if source_code.is_empty() {
        error!("Contract source code is empty. The contract might not be verified on Etherscan.");
        return Err("Contract source code is empty".into());
    }

    // Handle different source code formats
    let sources = if source_code.starts_with('{') {
        // Handle JSON format
        let contract: serde_json::Value = if source_code.contains("{{") {
            // Handle double-braced format
            let cleaned = source_code.replace("{{", "{").replace("}}", "}");
            serde_json::from_str(&cleaned)?
        } else {
            serde_json::from_str(source_code)?
        };

        contract["sources"].as_object()
            .ok_or("No sources object in contract")?
            .clone()
    } else {
        // Handle single file format
        let mut map = serde_json::Map::new();
        map.insert(
            "Single.sol".to_string(),
            serde_json::json!({
                "content": source_code
            }),
        );
        map
    };

    // Create contract files
    for (key, value) in sources {
        let mut file_path = src_path.clone();
        
        // Create directories if needed
        let parts: Vec<&str> = key.split('/').collect();
        for dir in &parts[..parts.len()-1] {
            file_path.push(dir);
            std::fs::create_dir_all(&file_path)?;
            info!("Created directory: {:?}", file_path);
        }
        
        file_path.push(parts.last().unwrap());
        
        if let Some(content) = value["content"].as_str() {
            info!("Creating file: {:?}", file_path);
            let mut file = std::fs::File::create(file_path)?;
            file.write_all(content.as_bytes())?;
        }
    }

    info!("Contract cloning completed successfully!");
    Ok(())
}