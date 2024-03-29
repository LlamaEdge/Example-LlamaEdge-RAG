use clap::{crate_version, Arg, Command};
use endpoints::{
    chat::{
        ChatCompletionChunk, ChatCompletionRequestBuilder, ChatCompletionRequestMessage,
        ChatCompletionUserMessageContent,
    },
    common::FinishReason,
    embeddings::EmbeddingRequest,
};
use futures::StreamExt;
use std::fs::File;
use std::io::prelude::*;
use std::io::stdout;
use std::io::Write;
use std::path::Path;
use text_splitter::TextSplitter;
use tiktoken_rs::cl100k_base;

#[allow(unreachable_code)]
#[tokio::main]
async fn main() -> Result<(), String> {
    let matches = Command::new("llama-chat")
        .version(crate_version!())
        .arg(
            Arg::new("file")
                .long("file")
                .value_name("FILE")
                .help("File with the *.txt extension"),
        )
        .after_help("Example:\n  wasmedge --dir .:. llama-rag.wasm --file paris.txt\n")
        .get_matches();

    // parse the command line arguments
    let file = matches.get_one::<String>("file").unwrap();
    let file_path = std::path::PathBuf::from(file);
    if !file_path.exists() {
        return Err(format!("{} does not exist", file));
    }
    println!("[INFO] Document: {}\n", file);

    println!("[+] Chunking the document ...");

    // * load and chunk the text file
    let chunks = chunk_document(file)?;

    // * print chunks
    // println!("    Chunks: \n");
    // for chunk in chunks.iter() {
    //     println!("    {}\n", chunk);
    // }
    // println!("\n");

    println!("[+] Computing the embeddings for the document ...");

    // * use LlamaEdge API for RAG to compute and persist embeddings for the chunks
    upload_chunks(&chunks).await?;

    loop {
        println!("\n[You]: ");
        let user_input = read_input();

        // * answer a user query based on the document
        let mut stream = query(&user_input).await?.bytes_stream();

        // * print result
        println!("\n[Bot]: ");
        let mut first = true;
        while let Some(item) = stream.next().await {
            match item {
                Ok(bytes) => {
                    let s = String::from_utf8_lossy(&bytes).to_string();

                    let stream =
                        serde_json::Deserializer::from_str(&s).into_iter::<serde_json::Value>();

                    for value in stream {
                        match value {
                            Ok(v) => {
                                let chat_completion_chunk: ChatCompletionChunk =
                                    serde_json::from_value(v).unwrap();

                                let choice = &chat_completion_chunk.choices[0];

                                match choice.finish_reason {
                                    None => {
                                        let token = choice.delta.content.as_ref().unwrap();

                                        if token.is_empty() {
                                            continue;
                                        }

                                        if first {
                                            first = false;
                                            print_text(token.trim_start());
                                        } else {
                                            print_text(token);
                                        };
                                    }
                                    Some(FinishReason::stop) => {
                                        break;
                                    }
                                    Some(FinishReason::length) => {
                                        if let Some(token) = choice.delta.content.as_ref() {
                                            print_text(token);
                                        }

                                        break;
                                    }
                                    Some(_) => panic!("Unexpected finish reason"),
                                };
                            }
                            Err(err) => eprintln!("Error: {}", err),
                        }
                    }
                }
                Err(err) => eprintln!("Error: {}", err),
            }
        }
        println!("\n");
    }

    Ok(())
}

fn chunk_document(file: &str) -> Result<Vec<String>, String> {
    let file_path = Path::new(file);
    if !file_path.exists() {
        return Err("File does not exist".to_string());
    }
    if file_path.extension().is_none() || file_path.extension().unwrap() != "txt" {
        return Err("File is not a text file".to_string());
    }

    // read contents from a text file
    let mut file = File::open(file_path).expect("failed to open file");
    let mut text = String::new();
    file.read_to_string(&mut text).expect("failed to read file");

    let tokenizer = cl100k_base().unwrap();
    let max_tokens = 100;
    let splitter = TextSplitter::new(tokenizer).with_trim_chunks(true);

    let chunks = splitter
        .chunks(&text, max_tokens)
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    Ok(chunks)
}

async fn upload_chunks(chunks: &[String]) -> Result<(), String> {
    // create embedding request
    let embedding_request = EmbeddingRequest {
        model: "dummy-embedding-model".to_string(),
        input: chunks.to_vec(),
        encoding_format: None,
        user: None,
    };

    // * print the serialized embedding_request
    // println!("Serialized embedding_request:\n{}\n\n", serde_json::to_string_pretty(&embedding_request).unwrap());

    // create a client
    let client = reqwest::Client::new();

    let request_body = serde_json::to_value(&embedding_request).unwrap();
    match client
        .post("http://localhost:8080/v1/embeddings")
        .header("accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
    {
        Ok(_) => Ok(()),
        Err(err) => {
            println!("Error: {}", err);
            Err(err.to_string())
        }
    }
}

async fn query(input: &str) -> Result<reqwest::Response, String> {
    // create user message
    let user_message = ChatCompletionRequestMessage::new_user_message(
        ChatCompletionUserMessageContent::Text(input.to_string()),
        None,
    );

    // create a chat completion request
    let chat_request =
        ChatCompletionRequestBuilder::new("dummy-chat-completion-model", vec![user_message])
            .with_stream(true)
            .build();

    // * print the serialized chat_request
    // println!(
    //     "\n\nSerialized chat_request:\n{}\n\n",
    //     serde_json::to_string_pretty(&chat_request).unwrap()
    // );

    // create a client
    let client = reqwest::Client::new();
    let request_body = serde_json::to_value(&chat_request).unwrap();

    client
        .post("http://localhost:8080/v1/chat/completions")
        .header("accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .map_err(|e| e.to_string())
}

fn print_text(text: &str) {
    print!("{}", text);
    stdout().flush().unwrap();
}

// For single line input, just press [Return] to end the input.
// For multi-line input, end your input with '\\' and press [Return].
//
// For example:
//  [You]:
//  what is the capital of France?[Return]
//
//  [You]:
//  Count the words in the following sentence: \[Return]
//  \[Return]
//  You can use Git to save new files and any changes to already existing files as a bundle of changes called a commit, which can be thought of as a “revision” to your project.[Return]
//
fn read_input() -> String {
    let mut answer = String::new();
    loop {
        let mut temp = String::new();
        std::io::stdin()
            .read_line(&mut temp)
            .expect("The read bytes are not valid UTF-8");

        if temp.ends_with("\\\n") {
            temp.pop();
            temp.pop();
            temp.push('\n');
            answer.push_str(&temp);
            continue;
        } else if temp.ends_with("\n") {
            answer.push_str(&temp);
            return answer;
        } else {
            return answer;
        }
    }
}
