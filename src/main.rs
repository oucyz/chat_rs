use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
    },
    Client,
};
use dotenv::dotenv;
use futures_util::StreamExt;
use std::{env, io};

#[tokio::main]
async fn main() {
    let mut chat = Chat::new();
    chat.run().await;
}

fn get_client() -> Client<OpenAIConfig> {
    dotenv().ok();
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let org_id = env::var("ORGANIZATION_ID").expect("ORGANIZATION_ID must be set");
    let config = OpenAIConfig::new()
        .with_api_key(api_key)
        .with_org_id(org_id);
    Client::with_config(config)
}

struct Chat {
    client: Client<OpenAIConfig>,
    chat_history: Vec<ChatCompletionRequestMessage>,
}

impl Chat {
    fn new() -> Self {
        Self {
            client: get_client(),
            chat_history: vec![ChatCompletionRequestMessage::System(
                "You are a helpful assistant. Create response against your request".into(),
            )],
        }
    }

    async fn read_user_input(&mut self) -> ChatCompletionRequestMessage {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let trimmed = input.trim();
        println!("User: {}", trimmed);
        let message = ChatCompletionRequestMessage::User(trimmed.into());
        self.chat_history.push(message.clone());
        message
    }

    async fn create_chat_completion_request(&self) -> CreateChatCompletionRequest {
        CreateChatCompletionRequestArgs::default()
            .model("gpt-4o-mini")
            .messages(self.chat_history.as_slice())
            .build()
            .unwrap()
    }

    async fn process_response(&mut self, request: CreateChatCompletionRequest) {
        let mut stream = self.client.chat().create_stream(request).await.unwrap();
        let mut full_response = String::new();
        while let Some(item) = stream.next().await {
            match item {
                Ok(response_part) => {
                    if let Some(choice) = response_part.choices.first() {
                        if let Some(content) = &choice.delta.content {
                            full_response.push_str(content);
                        }
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        let assistant_message =
            ChatCompletionRequestMessage::Assistant(full_response.clone().into());
        self.chat_history.push(assistant_message);
        println!("Assistant: {}", full_response);
    }

    async fn run(&mut self) {
        for _ in 0..5 {
            self.read_user_input().await;
            let request = self.create_chat_completion_request().await;
            self.process_response(request).await;
        }
        println!("Chat Completed");
    }
}
