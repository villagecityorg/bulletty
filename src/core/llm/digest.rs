/// Daily Digest — LLM-powered article summarization.
use crate::core::llm::{LlmRuntime, chat_completion};

pub fn generate_digest(
    runtime: &LlmRuntime,
    articles: &[(&str, &str, &str)],
    kid_safe: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let list: String = articles
        .iter()
        .map(|(t, c, p)| format!("[{}] {} — {}", c, t, p))
        .collect::<Vec<_>>()
        .join("\n");

    let safety = if kid_safe { "\nOmit mature content." } else { "" };

    let system_prompt = concat!(
        "You are a family briefing assistant. Summarize articles concisely. ",
        "Group by category. Keep it brief."
    );

    let user_prompt = format!(
        "Articles:\n{}\n\nProduce: 1) overview, 2) key items by category, 3) top 3 to read.{}",
        list, safety
    );

    chat_completion(runtime, system_prompt, &user_prompt)
}
