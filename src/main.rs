use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest, FunctionCallType};
use openai_api_rs::v1::common::{GPT3_5_TURBO, GPT4};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::{env, vec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let git_diff = get_git_diff()?;

    let prompt = format!(
        r#"Can you help me explain to my team what changes were made in this repo?
        Git Diff: {}"#,
        git_diff
    );

    let api_token = env::var("OPENAI_API_KEY")?;
    let client = Client::new(api_token);
    let model = get_model();

    let req = ChatCompletionRequest::new(model, vec![user_message(prompt)])
        .functions(build_openai_functions())
        .function_call(FunctionCallType::Auto);

    // INFO: debug what the request looks like
    // let serialized = serde_json::to_string(&req).unwrap();
    // println!("{}", serialized);

    let result = client.chat_completion(req)?;

    process_chat_completion_response(result)?;

    Ok(())
}

fn get_git_diff() -> Result<String, Box<dyn std::error::Error>> {
    let mut stdin = std::io::stdin().lock();

    let mut line = String::new();

    let mut diff = String::new();

    while let Ok(n_bytes) = stdin.read_to_string(&mut line) {
        if n_bytes == 0 {
            break;
        }
        diff += &line;
        line.clear();
    }

    if diff.is_empty() {
        return Err("No diff provided, pipe your diff to me, plz".into());
    }

    Ok(diff)
}

fn get_model() -> String {
    let args: Vec<String> = env::args().collect();
    let mut model = GPT3_5_TURBO.to_string();
    if args.len() > 1 && args[1] == "--gpt4" {
        model = GPT4.to_string();
    }
    model
}

fn user_message(content: String) -> chat_completion::ChatCompletionMessage {
    chat_completion::ChatCompletionMessage {
        role: chat_completion::MessageRole::user,
        content,
        function_call: None,
        name: None,
    }
}

fn build_openai_functions() -> Vec<chat_completion::Function> {
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
    let functions = vec![chat_completion::Function {
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
    }];
    functions
}

fn process_chat_completion_response(
    result: chat_completion::ChatCompletionResponse,
) -> Result<(), Box<dyn std::error::Error>> {
    match result.choices[0].finish_reason {
        None => Err("OpenAI returned no finish reason".into()),
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
            Ok(())
        }
        Some(_) => Err("OpenAI returned an unexpected finish reason".into()),
    }
}

fn explain_diff(explanation: &str, suggested_commit: &str) {
    println!("Concise explanation: {}", explanation);
    println!("Suggested commit message: {}", suggested_commit);
}
