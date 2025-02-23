use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, CreateChatCompletionRequest, CreateChatCompletionRequestArgs,
    },
    Client,
};
use dotenv::dotenv;
use futures_util::StreamExt;
use std::env;

#[tokio::main]
async fn main() {
    let mut chat = Chat::new();
    chat.chat().await;
}

fn get_client() -> Client<OpenAIConfig> {
    dotenv().ok();
    let openai_api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let org_id = env::var("ORGANIZATION_ID").expect("ORGANIZATION_ID must be set");

    let config = OpenAIConfig::new()
        .with_api_key(openai_api_key)
        .with_org_id(org_id);

    Client::with_config(config)
}

struct Chat {
    client: Client<OpenAIConfig>,
    chat_history: Vec<ChatCompletionRequestMessage>,
}
impl Chat {
    fn new() -> Self {
        let client = get_client();
        let chat_history = vec![ChatCompletionRequestMessage::System(
            "You are a helpful assistant. Create response against your request".into(),
        )];
        Self {
            client,
            chat_history,
        }
    }
    async fn get_user_input(&mut self) -> ChatCompletionRequestMessage {
        let mut user_input = String::new();
        std::io::stdin().read_line(&mut user_input).unwrap();
        let user_input = user_input.trim();

        let user_prompt = ChatCompletionRequestMessage::User(user_input.into());
        self.chat_history.push(user_prompt.clone());
        user_prompt
    }
    async fn create_chat_completion_request(&self) -> CreateChatCompletionRequest {
        CreateChatCompletionRequestArgs::default()
            .model("gpt-4o-mini")
            .messages(self.chat_history.clone())
            .build()
            .unwrap()
    }
    async fn get_llm_response(&mut self, request: CreateChatCompletionRequest) {
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
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            }
        }
        let llm_response = ChatCompletionRequestMessage::Assistant(full_response.as_str().into());
        self.chat_history.push(llm_response.clone());
        println!("{}", full_response);
    }
    async fn chat(&mut self) {
        let mut cnt = 5;
        while cnt < 5 {
            self.get_user_input().await;
            let request = self.create_chat_completion_request().await;
            self.get_llm_response(request).await;
            println!("{}", "Chat Completed");
            cnt -= 1;
        }
    }
}
