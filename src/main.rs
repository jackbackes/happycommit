use async_openai::{types::CreateChatCompletionRequestArgs, Client};
use futures::StreamExt;
use lazy_static::lazy_static;
use std::{
    io::{stdout, Write},
    ops::{Deref, DerefMut},
    process::{Command, Stdio},
};
use tiktoken_rs::cl100k_base;
use tokio::sync::Mutex;

lazy_static! {
    static ref STDOUT_LOCK: Mutex<()> = Mutex::new(());
}

#[derive(Clone)]
pub struct ChatCompletionRequestMessage {
    pub name: Option<String>,
    pub content: String,
    pub role: String,
}

#[allow(clippy::from_over_into)] // TODO: fix this
impl Into<tiktoken_rs::ChatCompletionRequestMessage> for ChatCompletionRequestMessage {
    fn into(self) -> tiktoken_rs::ChatCompletionRequestMessage {
        tiktoken_rs::ChatCompletionRequestMessage {
            name: self.name,
            content: self.content,
            role: self.role,
        }
    }
}

impl From<tiktoken_rs::ChatCompletionRequestMessage> for ChatCompletionRequestMessage {
    fn from(message: tiktoken_rs::ChatCompletionRequestMessage) -> Self {
        ChatCompletionRequestMessage {
            name: message.name,
            content: message.content,
            role: message.role,
        }
    }
}

#[allow(clippy::from_over_into)] // TODO: fix this
impl Into<async_openai::types::ChatCompletionRequestMessage> for ChatCompletionRequestMessage {
    fn into(self) -> async_openai::types::ChatCompletionRequestMessage {
        async_openai::types::ChatCompletionRequestMessage {
            name: self.name,
            content: self.content,
            role: match self.role.as_str() {
                "system" => async_openai::types::Role::System,
                "user" => async_openai::types::Role::User,
                "assistant" => async_openai::types::Role::Assistant,
                _ => panic!("Invalid role"),
            },
        }
    }
}

impl From<async_openai::types::ChatCompletionRequestMessage> for ChatCompletionRequestMessage {
    fn from(message: async_openai::types::ChatCompletionRequestMessage) -> Self {
        ChatCompletionRequestMessage {
            name: message.name,
            content: message.content,
            role: message.role.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct ChatCompletionRequestMessages {
    pub messages: Vec<ChatCompletionRequestMessage>,
}

#[allow(clippy::from_over_into)] // TODO: fix this
impl Into<Vec<tiktoken_rs::ChatCompletionRequestMessage>> for ChatCompletionRequestMessages {
    fn into(self) -> Vec<tiktoken_rs::ChatCompletionRequestMessage> {
        self.messages
            .into_iter()
            .map(|message| message.into())
            .collect()
    }
}

impl From<Vec<ChatCompletionRequestMessage>> for ChatCompletionRequestMessages {
    fn from(messages: Vec<ChatCompletionRequestMessage>) -> Self {
        ChatCompletionRequestMessages { messages }
    }
}

impl Deref for ChatCompletionRequestMessages {
    type Target = Vec<ChatCompletionRequestMessage>;

    fn deref(&self) -> &Self::Target {
        &self.messages
    }
}

impl DerefMut for ChatCompletionRequestMessages {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.messages
    }
}

fn get_code_changes() -> Result<String, Box<dyn std::error::Error>> {
    // todo: handle references
    let output = Command::new("git")
        .arg("diff")
        .arg("--cached")
        .arg("--")
        .arg(".")
        // use the user's current directory, from where the command was executed
        .current_dir(std::env::current_dir()?)
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout)
}

const SYSTEM_MESSAGE: &str = "You are helping the user write a great commit message.
======
FORMAT
======
The commit message should be in the following format:
==========
Branch: <suggested branch name>
~~~~~~~~~~
Subject: <subject (50 chars or less)>
~~~~~~~~~~
Body: <body>
==========
END FORMAT
==========

Instructions:

You can use the following guidelines to help them write a good commit message:

Write a concise and informative subject line: The subject line should summarize the change in 50 characters or less. It should be written in the imperative mood (e.g., \"Add feature X\" rather than \"Added feature X\").

Separate subject from body with a blank line: If you need to provide more context, include a message body after a blank line. This helps separate the summary from the details.

Explain the \"what\" and \"why\" in the message body: The message body should provide context on why the change was made and any potential implications. Avoid focusing on the \"how\" since that can be deduced from the code itself.

Use proper grammar, spelling, and punctuation: Good commit messages are well-written and easy to understand. Proper language usage helps convey the meaning effectively.

Keep line lengths reasonable: Aim for a maximum of 72 characters per line in the message body to ensure readability across different devices and tools.

Use bullet points or lists for multiple changes: If the commit contains several changes, organize them using bullet points or numbered lists for better readability.

Avoid generic or ambiguous messages: Commit messages like \"bug fix\" or \"updates\" don't provide enough context. Be specific about the changes you've made.

Don't include code in the message: The commit message should describe the change, not include the code itself. If the change is too complex to describe succinctly, consider breaking it into smaller commits.

Proofread before committing: Double-check your commit message for clarity, accuracy, and completeness before submitting it.

Organize the commit message to include any testing done: If you have tested the code, include the results in the commit message. If you have not tested the code, include a note that you have not tested the code.

Humor is ok, but don't overdo it. :wink:

All of the provided code is from a single staged workspace.";

fn create_message(message: ChatMessage) -> async_openai::types::ChatCompletionRequestMessage {
    async_openai::types::ChatCompletionRequestMessage {
        name: Some(message.0),
        content: message.2,
        role: message.1,
    }
}

async fn send_to_openai(
    client: &Client,
    messages: Vec<ChatMessage>,
) -> Result<String, anyhow::Error> {
    let mut stdout = stdout().lock();
    let result = client
        .chat()
        .create_stream(
            CreateChatCompletionRequestArgs::default()
                .messages(messages.into_iter().map(create_message).collect::<Vec<_>>())
                .model("gpt-3.5-turbo")
                .build()?,
        )
        .await;

    let mut collector = String::new();

    match result {
        Err(e) => panic!("Failed to connect to OpenAI API: {}", e),

        Ok(mut response_stream) => {
            while let Some(partial_response) = response_stream.next().await {
                match partial_response {
                    Err(e) => {
                        println!("Error in Chat Completion: {}", e);
                    }
                    Ok(response) => {
                        let _ = stdout.write(
                            response.choices[0]
                                .delta
                                .content
                                .clone()
                                .unwrap_or_default()
                                .as_bytes(),
                        )?;
                        // add to collector
                        collector.push_str(
                            &response.choices[0]
                                .delta
                                .content
                                .clone()
                                .unwrap_or_default(),
                        );
                        stdout.flush()?;
                    }
                }
            }
        }
    }
    let _ = stdout.write(b"\n")?;

    Ok(collector)
}

type ChatMessage = (String, async_openai::types::Role, String);

fn load_api_key() -> Result<String, Box<dyn std::error::Error>> {
    // first check in ~/.happycommit/config.toml
    let happycommitconfig_checker = || -> Result<String, Box<dyn std::error::Error>> {
        let config_path = dirs::home_dir().unwrap().join(".happycommit/config.toml");
        let config = std::fs::read_to_string(config_path)?;
        let config: toml::Value = toml::from_str(&config)?;
        let openai_api_key = config.get("OPENAI_API_KEY");
        if openai_api_key.is_none() {
            return Err("OPENAI_API_KEY not set in ~/.happycommit/config.toml".into());
        }
        let result = openai_api_key.unwrap().to_string();
        // strip quotes
        let result = result.replace("\"", "");
        Ok(result)
    };
    let dotenv_checker = || -> Result<String, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();
        let openai_api_key =
            dotenv::var("OPENAI_API_KEY")?;
        Ok(openai_api_key)
    };

    // first check happycommit config, then check dotenv - if in no places, throw
    let result = happycommitconfig_checker();
    if result.is_ok() {
        return result;
    }
    let result = dotenv_checker();
    if result.is_ok() {
        return result;
    }
    // throw an error

    Err("OPENAI_API_KEY must be set in .env file or ~/.happycommit/config.toml".into())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let openai_api_key = load_api_key()
        .map_err(|e| {
            panic!("Error loading OpenAI API key: {}", e);
        })
        .unwrap();

    // by default, read in all the code changes since origin/master
    // TODO: allow user to specify a different origin branch or commit
    let code_changes = get_code_changes();

    let code_changes = match code_changes {
        Ok(code_changes) => code_changes,
        Err(e) => {
            println!("Error getting code changes: {}", e);
            return;
        }
    };

    let client = Client::new().with_api_key(openai_api_key);

    // test the client connection
    println!("Testing connection to OpenAI API...");
    let _ = send_to_openai(
        &client,
        vec![
            (
                "Bot".to_string(),
                async_openai::types::Role::System,
                "You are a bot that writes commit messages.".to_string(),
            ),
            (
                "Test".to_string(),
                async_openai::types::Role::User,
                "Hello world!".to_string(),
            ),
        ],
    )
    .await
    .map_err(|e| panic!("Failed to connect to OpenAI API: {}", e));
    println!("Connection successful!");

    let result = stream_multipart_commit_message(&client, SYSTEM_MESSAGE, code_changes.as_str())
        .await
        .map_err(|e| panic!("Failed to connect to OpenAI API: {}", e))
        .unwrap();

    let final_message = result.last().unwrap();

    println!("Final Commit message:\n{}\n\n", final_message.as_str());

    let create_messages = |query: String| -> Vec<ChatMessage> {
        // assert that query must be one of "Branch", "Subject", or "Body"
        assert!(query == "Branch" || query == "Subject" || query == "Body");
        vec![
            ("CommitQueryBot".to_string(), async_openai::types::Role::System, "The CommitMesssageProvider will provide text using the following format, then request either \"Branch\", \"Subject\", or \"Body\"
            You will provide back the user the exact contents of what is requested. This will be done without any additiona prose or formatting.
            Here is the format that the user will provide (note, the user may not follow this format exactly, you will need to interpolate the text into the correct place):
            ```
            Branch: <suggested branch name>
            ~~~~~~~~~~
            Subject: <subject>
            ~~~~~~~~~~
            Body: <body>
            ```

            Example 1:
            ```
            Provide Branch for:

            Branch: feature/1234
            ~~~~~~~~~~
            Subject: Add a new feature
            ~~~~~~~~~~
            Body: This is a new feature that does something really cool.
            - cool thing 1
            - cool thing 2

            Output: feature/1234
            ```

            Example 2:
            ```
            Provide Subject for:

            Branch: feature/1234
            ~~~~~~~~~~
            Subject: Add a new feature
            ~~~~~~~~~~
            Body: This is a new feature that does something really cool.
            - cool thing 1
            - cool thing 2

            Output: \"Add a new feature\"
            ```

            Example 3:
            ```
            Provide Body for:

            Branch: feature/1234
            ~~~~~~~~~~
            Subject: Add a new feature
            ~~~~~~~~~~
            Body: This is a new feature that does something really cool.
            - cool thing 1
            - cool thing 2

            Output: \"This is a new feature that does something really cool.
            - cool thing 1
            - cool thing 2\"
            ```".to_string()),
            ("CommitMessageProvider".to_string(), async_openai::types::Role::Assistant, final_message.to_string()),
            ("User".to_string(), async_openai::types::Role::User, format!("Provide {} for commit message ^", query)),
        ]
    };

    // switch branch using git switch -c <branch>
    let branch = send_to_openai(&client, create_messages("Branch".to_string()))
        .await
        .unwrap();
    let branch = branch.trim();
    println!("Setting Branch: {}", branch);
    let _ = Command::new("git")
        .arg("switch")
        .arg("-c")
        .arg(branch)
        .output()
        .expect("Failed to set branch");

    // open the commit message in the editor, with the subject and body filled in
    let subject = send_to_openai(&client, create_messages("Subject".to_string()))
        .await
        .unwrap();
    let body = send_to_openai(&client, create_messages("Body".to_string()))
        .await
        .unwrap();

    let commit_message = format!("{}


    {}

    ~~~~~~~~~~
    This commit message was generated by HappyCommit. Try it in your project today!
    Check it out at https://github.com/jackbackes/happycommit
    ", subject, body);

    let mut commit_file = tempfile::NamedTempFile::new().expect("Failed to create temporary file");
    commit_message.split('\n').for_each(|line| {
        let _ = writeln!(commit_file, "{}", line);
    });

    // flush and close the file
    let _ = commit_file.flush();

    let status = Command::new("git")
        .arg("commit")
        .arg("--file")
        .arg(commit_file.path())
        .arg("--edit")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to commit");

    if status.success() {
        println!("Commit successful! Thanks for using happycommit!");
    } else {
        eprintln!("Commit failed. Please check the error message and try again.");
    }
}

use anyhow::Result;

#[allow(clippy::needless_borrow)]
async fn stream_multipart_commit_message(
    client: &Client,
    initial_prompt: &str,
    code_changes: &str,
) -> Result<Vec<String>, anyhow::Error> {
    let tokenizer = cl100k_base().unwrap();
    let max_tokens = 3500;
    let tokenized_iter = tokenizer.split_by_token_with_special_tokens(code_changes);

    let overlap = 200;
    let mut split_code_changes = Vec::new();

    let initial_prompt_tokens = tokenizer
        .split_by_token_with_special_tokens(initial_prompt)
        .collect::<Result<Vec<String>, _>>()?
        .len();

    let max_code_tokens = max_tokens - initial_prompt_tokens;

    let mut tokenized_iter = tokenized_iter.enumerate().peekable();

    while tokenized_iter.peek().is_some() {
        let mut current_split = Vec::new();

        // Loop until current_split_len reaches max_code_tokens or all tokens are processed
        while let Some((i, token_res)) = tokenized_iter.peek_mut() {
            // If the token is an error, then return an error
            if token_res.is_err() {
                return Err(anyhow::anyhow!(
                    "Error tokenizing code changes at index {}: {:?}",
                    i,
                    split_code_changes
                ));
            }

            current_split.push(token_res.as_ref().unwrap().clone());

            // Check if we have reached max_code_tokens or if we are at the last token
            if current_split.len() >= max_code_tokens {
                // We've reached max_code_tokens, let's go back by "overlap" tokens if possible
                let tokens_to_skip = current_split.len() - overlap;

                println!(
                    "Finished current split. Going from {} to {} tokens",
                    current_split.len(),
                    tokens_to_skip
                );

                // Skip tokens_to_skip tokens in tokenized_iter for the next iteration
                tokenized_iter.nth(tokens_to_skip - 1);

                // Break out of the loop
                break;
            } else if tokenized_iter.peek().is_none() {
                // Consume the last token and exit the loop
                tokenized_iter.next();
                break;
            } else {
                // Consume the token and continue the loop
                tokenized_iter.next();
            }
        }

        // Add current_split to split_code_changes
        split_code_changes.push(current_split);
    }

    let mut iteration_count = 0;
    let split_code_changes_len = split_code_changes.len();

    // Process each split_code_changes using the provided client and model, and concatenate the results
    let mut commit_messages: Vec<String> = Vec::new();
    for split_code_change_slice in split_code_changes {
        iteration_count += 1;
        println!("Iteration {}", iteration_count);
        // Join the tokens back into a string. This is the slice of code changes we will send to the api.
        let code_change_slice = split_code_change_slice.join("");

        // Join the prompt, the code change slice, and the previous commit message if it exists.
        let mut messages = vec![
            (
                "GitCommitBot".to_string(),
                async_openai::types::Role::System,
                initial_prompt.to_string(),
            ),
            (
                "User".to_string(),
                async_openai::types::Role::User,
                code_change_slice.to_string(),
            ),
        ];
        // add the most recent commit message if it exists
        if let Some(commit_message) = commit_messages.last() {
            let message = "Previous commit message is as follows.
            Take this commit message and modify it to include the additional code snippet (above) provided by the user.
            ======
            FORMAT
            ======
            The commit message should be in the following format:
            ==========
            Branch: <suggested branch name>
            ~~~~~~~~~~
            Subject: <subject (50 chars or less)>
            ~~~~~~~~~~
            Body: <body>
            ==========
            END FORMAT
            ==========
            \n".to_string() + commit_message;
            messages.push((
                "PreviousCommitProvider".to_string(),
                async_openai::types::Role::User,
                message,
            ));
        }

        println!(
            "Sending slice #{} of {} to OpenAI API...",
            iteration_count, split_code_changes_len
        );
        let result = send_to_openai(&client, messages).await?;
        println!("Received response from OpenAI API");

        commit_messages.push(result);
    }

    Ok(commit_messages)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_tokenizer() {
        let tokenizer = tiktoken_rs::cl100k_base().unwrap();
        let tokens = tokenizer.encode_with_special_tokens("hello world");
        let foo = tokenizer.decode(tokens).unwrap();
        assert_eq!(foo, "hello world");
    }
}
