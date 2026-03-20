/// Build a prompt that asks the LLM whether two facts contradict each other.
pub fn contradiction_detection_prompt(fact_a: &str, fact_b: &str) -> String {
    format!(
        r#"You are a fact verification expert. Determine whether the following two facts contradict each other, confirm each other, or are unrelated.

Fact A: "{fact_a}"
Fact B: "{fact_b}"

Consider:
- Direct contradiction (one negates the other)
- Temporal contradiction (was true then, changed now)
- Partial contradiction (partially overlapping claims)
- Confirmation (both assert the same thing)
- Unrelated (different topics entirely)

Respond with a JSON object:
{{
  "relationship": "contradicts" | "confirms" | "unrelated",
  "confidence": 0.0 to 1.0,
  "reasoning": "brief explanation",
  "suggested_certainty_adjustment": -1.0 to 1.0
}}

The suggested_certainty_adjustment indicates how the certainty of Fact A should be modified:
- Positive values reinforce Fact A (confirmation)
- Negative values weaken Fact A (contradiction)
- Zero means no change (unrelated)

Respond ONLY with the JSON object, no additional text."#
    )
}

/// Build a system prompt for the contradiction detection task.
pub fn contradiction_detection_system() -> &'static str {
    "You are a precise fact verification system. You determine logical relationships between factual assertions. Always respond with valid JSON."
}
