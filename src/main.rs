use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest, FunctionCallType};
use openai_api_rs::v1::common::GPT4;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{env, vec};

fn explain_diff(explanation: &str, suggested_commit: &str) {
    println!("Concise explanation: {}", explanation);
    println!("Suggested commit message: {}", suggested_commit);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if !std::path::Path::new(".git").exists() {
        eprintln!("Not in a git repo");
        return Ok(());
    }

    let output = std::process::Command::new("git").arg("diff").output()?;

    if !output.status.success() {
        eprintln!("Failed to execute git diff");
        return Ok(());
    }

    let git_diff = String::from_utf8_lossy(&output.stdout);

    // check if git_diff is empty
    if git_diff.trim().is_empty() {
        eprintln!("No changes were made");
        return Ok(());
    }

    let api_token = env::var("OPENAI_API_KEY")?;
    let client = Client::new(api_token);

    let mut properties = HashMap::new();
    properties.insert(
        "explanation".to_string(),
        Box::new(chat_completion::JSONSchemaDefine {
            schema_type: Some(chat_completion::JSONSchemaType::String),
            description: Some(
                "A concise description about the changes and why they were made".to_string(),
            ),
            enum_values: None,
            properties: None,
            required: None,
            items: None,
        }),
    );
    properties.insert(
        "suggested_commit".to_string(),
        Box::new(chat_completion::JSONSchemaDefine {
            schema_type: Some(chat_completion::JSONSchemaType::String),
            description: Some("A commit message to suggest".to_string()),
            enum_values: None,
            properties: None,
            required: None,
            items: None,
        }),
    );

    let prompt = format!(
        r#"What changes were made in this diff?
        Git Diff: {}"#,
        git_diff
    );
    let req = ChatCompletionRequest::new(
        GPT4.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: prompt,
            function_call: None,
            name: None,
        }],
    )
    .functions(vec![chat_completion::Function {
        name: String::from("explain_diff"),
        description: Some(String::from("Explains the diff to the user")),
        parameters: chat_completion::FunctionParameters {
            schema_type: chat_completion::JSONSchemaType::Object,
            properties: Some(properties),
            required: Some(vec![
                String::from("explanation"),
                String::from("suggested_commit"),
            ]),
        },
    }])
    .function_call(FunctionCallType::Auto);

    // INFO: debug what the request looks like
    // let serialized = serde_json::to_string(&req).unwrap();
    // println!("{}", serialized);

    let result = client.chat_completion(req)?;

    match result.choices[0].finish_reason {
        None => {
            eprintln!("No finish_reason");
            eprintln!("{:?}", result.choices[0].message.content);
        }
        Some(chat_completion::FinishReason::stop) => {
            eprintln!("Stop");
            eprintln!("{:?}", result.choices[0].message.content);
        }
        Some(chat_completion::FinishReason::length) => {
            eprintln!("Length");
        }
        Some(chat_completion::FinishReason::function_call) => {
            #[derive(Serialize, Deserialize)]
            struct DiffExplained {
                explanation: String,
                suggested_commit: String,
            }
            let function_call = result.choices[0].message.function_call.as_ref().unwrap();
            let name = function_call.name.clone().unwrap();
            let arguments = function_call.arguments.clone().unwrap();
            let c: DiffExplained = serde_json::from_str(&arguments)?;
            let explanation = c.explanation;
            let suggested_commit = c.suggested_commit;
            if name == "explain_diff" {
                explain_diff(&explanation, &suggested_commit);
            }
        }
        Some(chat_completion::FinishReason::content_filter) => {
            eprintln!("ContentFilter");
        }
        Some(chat_completion::FinishReason::null) => {
            eprintln!("Null");
        }
    }

    Ok(())
}
