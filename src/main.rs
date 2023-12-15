use openai_api_rs::v1::api::Client;
use openai_api_rs::v1::chat_completion::{self, ChatCompletionRequest};
use openai_api_rs::v1::common::GPT3_5_TURBO;
use std::env;

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

    let api_token = env::var("OPENAI_API_KEY")?;
    let client = Client::new(api_token);

    let prompt = format!(
        r#"You are a coding assistant that helps programmers 
        write descriptions about changes. You will be given 
        the output of git diff, and you will write a 
        description of the changes to be used as a summary
        for the rest of the team. 
        Your goal is to write a concise explanation in markdown
        about what actually changed and WHY, and finally suggest 
        a commit message in this format:

        Concise explanation: <explanation>
        Suggested commit message: <message>

        Git Diff: {}"#,
        git_diff
    );
    let req = ChatCompletionRequest::new(
        GPT3_5_TURBO.to_string(),
        vec![chat_completion::ChatCompletionMessage {
            role: chat_completion::MessageRole::user,
            content: prompt,
            function_call: None,
            name: None,
        }],
    );

    let result = client.chat_completion(req)?;
    if let Some(choice) = result.choices.get(0) {
        let content = choice.message.content.as_deref().unwrap_or_default();
        println!("{}", content);
    } else {
        eprintln!("No results fromp OpenAI");
    }

    Ok(())
}
