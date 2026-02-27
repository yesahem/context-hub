use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::utils::config::OllamaConfig;

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: usize,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedContext {
    pub summary: String,
    pub files_changed: Vec<String>,
    pub key_details: Vec<String>,
    pub technologies: Vec<String>,
    pub impact: String,
}

pub struct LlmProcessor {
    client: Client,
    config: OllamaConfig,
}

impl LlmProcessor {
    pub fn new(config: OllamaConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    pub fn is_ollama_running(&self) -> bool {
        // Use a blocking reqwest call instead of shelling out to curl
        let url = format!("{}/api/tags", self.config.endpoint);
        reqwest::blocking::get(&url)
            .map(|resp| resp.status().is_success())
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub async fn check_ollama(&self) -> anyhow::Result<bool> {
        let url = format!("{}/api/tags", self.config.endpoint);
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    pub async fn extract_context(
        &self,
        commit_message: &str,
        diff: &str,
        files_changed: &[String],
        previous_context: Option<&str>,
    ) -> anyhow::Result<ExtractedContext> {
        let prompt = Self::build_prompt(commit_message, diff, files_changed, previous_context);
        
        let request = OllamaRequest {
            model: self.config.model.clone(),
            prompt,
            stream: false,
            options: OllamaOptions {
                temperature: self.config.temperature,
                num_predict: self.config.max_tokens,
            },
        };

        let url = format!("{}/api/generate", self.config.endpoint);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            return Err(anyhow::anyhow!("Ollama returned error: {}", status));
        }

        let ollama_resp: OllamaResponse = response.json().await?;
        
        Self::parse_response(&ollama_resp.response)
    }

    fn build_prompt(
        commit_message: &str,
        diff: &str,
        files_changed: &[String],
        previous_context: Option<&str>,
    ) -> String {
        let prev_section = match previous_context {
            Some(ctx) => format!(
                "\nPrevious Context (from the last processed commit):\n{}\n\nUse this to understand the evolving codebase and build incremental knowledge.\n",
                ctx
            ),
            None => String::new(),
        };

        format!(r#"You are a code context analyzer. Given a git commit diff, extract structured information about what was changed.
{}
Commit Message: {}

Files Changed: {}

Diff:
{}

Respond ONLY with valid JSON (no other text):
{{
  "summary": "1-2 sentence description of what this commit does",
  "files_changed": ["list of key files that were modified"],
  "key_details": ["2-4 important technical details about this change"],
  "technologies": ["technologies/libraries used"],
  "impact": "high|medium|low - how significant is this change"
}}"#, prev_section, commit_message, files_changed.join(", "), diff)
    }

    fn parse_response(response: &str) -> anyhow::Result<ExtractedContext> {
        if response.is_empty() {
            return Ok(ExtractedContext {
                summary: "Empty response from LLM".to_string(),
                files_changed: vec![],
                key_details: vec![],
                technologies: vec![],
                impact: "low".to_string(),
            });
        }
        
        let json_start = response.find('{');
        let json_end = response.rfind('}');
        
        if let (Some(start), Some(end)) = (json_start, json_end) {
            let json_str = &response[start..=end];
            
            #[derive(Deserialize)]
            struct RawContext {
                summary: String,
                #[serde(default)]
                files_changed: Vec<String>,
                #[serde(default)]
                key_details: Vec<String>,
                #[serde(default)]
                technologies: Vec<String>,
                #[serde(default)]
                impact: String,
            }
            
            if let Ok(raw) = serde_json::from_str::<RawContext>(json_str) {
                return Ok(ExtractedContext {
                    summary: raw.summary,
                    files_changed: raw.files_changed,
                    key_details: raw.key_details,
                    technologies: raw.technologies,
                    impact: if raw.impact.is_empty() { "medium".to_string() } else { raw.impact },
                });
            }
        }
        
        Ok(ExtractedContext {
            summary: format!("Raw LLM response: {}", &response[..response.len().min(200)]),
            files_changed: vec![],
            key_details: vec![],
            technologies: vec![],
            impact: "low".to_string(),
        })
    }

    #[allow(dead_code)]
    pub fn set_model(&mut self, model: String) {
        self.config.model = model;
    }

    #[allow(dead_code)]
    pub fn set_endpoint(&mut self, endpoint: String) {
        self.config.endpoint = endpoint;
    }

    #[allow(dead_code)]
    pub fn get_models(&self) -> Vec<String> {
        vec![
            "llama3.2".to_string(),
            "llama3.1".to_string(),
            "mistral".to_string(),
            "codellama".to_string(),
            "phi3".to_string(),
        ]
    }
}

pub fn check_ollama_installation() -> bool {
    std::process::Command::new("which")
        .arg("ollama")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
