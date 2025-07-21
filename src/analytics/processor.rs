use anyhow::{Result, Context};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::models::UsageEntry;
use super::calculator::CostCalculator;

/// Processes JSONL files from usage data
pub struct UsageProcessor {
    data_dir: PathBuf,
    cost_calculator: CostCalculator,
}

impl UsageProcessor {
    /// Create a new processor instance
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().context("Could not find home directory")?;
        let data_dir = home_dir.join(".claude").join("projects");
        
        if !data_dir.exists() {
            return Err(anyhow::anyhow!("Data directory not found at ~/.claude/projects"));
        }
        
        Ok(Self {
            data_dir,
            cost_calculator: CostCalculator::new(),
        })
    }

    // Removed unused new_fallback method during cleanup

    /// Find all JSONL files in the data directory
    pub fn find_jsonl_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        for entry in WalkDir::new(&self.data_dir) {
            let entry = entry?;
            if let Some(extension) = entry.path().extension() {
                if extension == "jsonl" {
                    files.push(entry.path().to_path_buf());
                }
            }
        }
        
        // Sort by modification time (newest first)
        files.sort_by(|a, b| {
            b.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH)
                .cmp(&a.metadata()
                    .and_then(|m| m.modified())
                    .unwrap_or(std::time::UNIX_EPOCH))
        });
        
        Ok(files)
    }

    /// Process all JSONL files and return usage entries
    pub fn process_all_files(&self) -> Result<Vec<UsageEntry>> {
        let files = self.find_jsonl_files()?;
        println!("📁 Found {} JSONL files to process", files.len());
        
        let mut all_entries = Vec::new();
        let mut global_deduplication = HashSet::new();
        
        for (i, file_path) in files.iter().enumerate() {
            println!("📄 Processing file {}/{}: {:?}", i + 1, files.len(), file_path);
            
            match self.process_file(file_path, &mut global_deduplication) {
                Ok(entries) => {
                    println!("  ✅ Processed {} entries", entries.len());
                    all_entries.extend(entries);
                }
                Err(e) => {
                    eprintln!("  ❌ Error processing file: {}", e);
                    continue;
                }
            }
        }
        
        // Sort by timestamp (newest first)
        all_entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        println!("✅ Total entries processed: {}", all_entries.len());
        Ok(all_entries)
    }

    // Removed unused process_recent_files method during cleanup

    /// Process a single JSONL file
    pub fn process_file(&self, file_path: &Path, global_dedup: &mut HashSet<String>) -> Result<Vec<UsageEntry>> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {:?}", file_path))?;
        
        let mut entries = Vec::new();
        let mut local_dedup = HashSet::new();
        
        // Extract session ID from file path
        let session_id = file_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_string());
        
        for (line_num, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            
            match self.process_line(line, &session_id, &mut local_dedup, global_dedup) {
                Ok(Some(entry)) => entries.push(entry),
                Ok(None) => continue, // Filtered out or duplicate
                Err(e) => {
                    eprintln!("  Warning: Line {}: {}", line_num + 1, e);
                    continue;
                }
            }
        }
        
        Ok(entries)
    }

    /// Process a single line from a JSONL file
    fn process_line(
        &self, 
        line: &str, 
        session_id: &Option<String>,
        local_dedup: &mut HashSet<String>,
        global_dedup: &mut HashSet<String>
    ) -> Result<Option<UsageEntry>> {
        let json_value: Value = serde_json::from_str(line)
            .context("Failed to parse JSON")?;
        
        // Extract basic information
        let timestamp = self.extract_timestamp(&json_value)?;
        let message = json_value.get("message")
            .context("Missing message field")?;
        
        // Check if this entry has usage data
        let usage = match message.get("usage") {
            Some(usage_value) if !usage_value.is_null() => usage_value,
            _ => return Ok(None), // No valid usage data
        };
        
        // Extract identifiers for deduplication
        let message_id = message.get("id").and_then(|v| v.as_str());
        let request_id = json_value.get("requestId").and_then(|v| v.as_str());
        
        // Create deduplication key
        if let (Some(msg_id), Some(req_id)) = (message_id, request_id) {
            let dedup_key = format!("{}:{}", msg_id, req_id);
            
            // Check both local and global deduplication
            if local_dedup.contains(&dedup_key) || global_dedup.contains(&dedup_key) {
                return Ok(None); // Duplicate
            }
            
            local_dedup.insert(dedup_key.clone());
            global_dedup.insert(dedup_key);
        }
        
        // Extract token counts
        let input_tokens = usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let output_tokens = usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let cache_read_tokens = usage.get("cache_read_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let cache_creation_tokens = usage.get("cache_creation_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        
        // Filter out zero-token entries (like Claudia does)
        if input_tokens == 0 && output_tokens == 0 && cache_read_tokens == 0 && cache_creation_tokens == 0 {
            return Ok(None);
        }
        
        // Extract model and project information
        let model = message.get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        let project_path = json_value.get("cwd")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        // Calculate cost (use provided cost or calculate it)
        let cost = json_value.get("costUSD")
            .and_then(|v| v.as_f64())
            .unwrap_or_else(|| {
                self.cost_calculator.calculate_cost(&model, input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens)
            });
        
        Ok(Some(UsageEntry {
            timestamp,
            model,
            project_path,
            session_id: session_id.clone(),
            request_id: request_id.map(|s| s.to_string()),
            input_tokens,
            output_tokens,
            cache_read_tokens,
            cache_creation_tokens,
            cost,
        }))
    }

    /// Extract timestamp from JSON value
    fn extract_timestamp(&self, json_value: &Value) -> Result<DateTime<Utc>> {
        let timestamp_str = json_value.get("timestamp")
            .and_then(|v| v.as_str())
            .context("Missing or invalid timestamp")?;
        
        DateTime::parse_from_rfc3339(timestamp_str)
            .map(|dt| dt.with_timezone(&Utc))
            .with_context(|| format!("Failed to parse timestamp: {}", timestamp_str))
    }
}