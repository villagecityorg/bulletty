/// Smart Pick — LLM-powered feed recommendation.
use crate::core::llm::{LlmRuntime, chat_completion};

pub fn smart_pick(
    runtime: &LlmRuntime,
    kid_safe: bool,
    existing_feeds: &[(&str, &str)],
) -> Result<String, Box<dyn std::error::Error>> {
    let feed_list: String = existing_feeds
        .iter()
        .map(|(title, url)| format!("- {} ({})", title, url))
        .collect::<Vec<_>>()
        .join("\n");

    let safety = if kid_safe { " IMPORTANT: child-safe content only." } else { "" };

    let system_prompt = concat!(
        "You are a feed curator. Given existing subscriptions, suggest 3-5 new high-quality RSS feeds. ",
        "Return ONLY one feed per line: Title | url | Category"
    );

    let user_prompt = format!(
        "Current feeds:\n{}\n\nSuggest complementary feeds.{}",
        feed_list, safety
    );

    chat_completion(runtime, system_prompt, &user_prompt)
}
