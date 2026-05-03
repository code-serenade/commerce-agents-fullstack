use model_gateway_rs::{
    llm::{Llm, chat_completions::ChatCompletionsLlm},
    model::llm::{ChatMessage, LlmInput},
};

const TEST_IMAGE_DATA_URL: &str = concat!(
    "data:image/png;base64,",
    "iVBORw0KGgoAAAANSUhEUgAAAKAAAACgCAIAAAAErfB6AAABzElEQVR42u3dQXEDUQxEQSExf1DhohDIKZV4NeN+ZQTT1/XXrKobEwAWYAEWYAEWYAEGLMACLMACLMACDFiABViABViABViAAQuwAAuwAAuwAAMWYAEWYAEW4J/6er1+9wNciNqNPVy7pQdtN/Og7WYetN3Mg7abeeh2Gw/dbuNB2808dLuNh2638dDtNh663cZDt9t46HYbD91uY8CA6SYbD91u46HbbQwYMN1kY8CA6SYbAwZMN9kYMGC6ycaAAQMGTPesMWDAgAEDBkz3GWPAgAEDBgwYMGDAgAEDBgwYMGDAgAEDBgwYMGDAgAEDBgwYMGDAHw3M2Ed3gAEDBgz4MWDGb14bMGDAgBn7hz/gTwJer+wABpwNvF66Aww4G3i9NgsYcDbwevG9HnjdbKgHXldXAAPOBl6Xz+qB1+3CeuB1fbQeeN0PrgdeF8DrgRONb854FzjL+OyGp4EjmI+vFwB82fj+dBnAN40jdosBPsUctFgY8OPMcVtFAj/CHLpSMPDbmKP3iQf+P+mOWXqA/wS7b4pOYAEGLMACLMACLMACDFiABViABViABViAAQuwAAuwAAuwAAMWYAEWYAEWYAEGLMACLMB6a9/jQ+ZVZVfJ1wAAAABJRU5ErkJggg==",
);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url =
        std::env::var("LLM_BASE_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".to_owned());
    let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "gemma4:26b".to_owned());
    let api_key = std::env::var("LLM_API_KEY").ok();

    let llm = ChatCompletionsLlm::new(&base_url, &model, api_key.as_deref())?
        .with_temperature(Some(0.0))
        .with_max_tokens(Some(1024));

    let output = llm
        .chat_once(LlmInput {
            messages: vec![ChatMessage::user_with_image(
                "Describe the image briefly. Mention the dominant color and shape. Answer in one \
                 short sentence.",
                TEST_IMAGE_DATA_URL,
            )],
        })
        .await?;

    println!("{}", output.get_content());
    Ok(())
}
