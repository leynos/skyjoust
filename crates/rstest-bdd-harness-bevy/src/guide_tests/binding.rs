//! Checks that the users' guide's scenario example binds its feature scenario.
//!
//! Rustdoc compiles the guide's `#[scenario]` fence but strips the generated
//! `#[test]` item, so nothing at runtime proves that the fence's step patterns
//! still match the feature scenario it selects. This module supplies that
//! check from the same loader that reads the published guide: it extracts the
//! fence's `path` and `index`, the scenario's steps from the feature text, and
//! the published `#[given]`/`#[when]`/`#[then]` patterns, then requires every
//! scenario step to be matched by a published pattern of the same keyword.
//! A drifted pattern therefore fails the crate's suite even though the
//! fence's hidden driver still calls the step functions directly.

use rstest::rstest;

/// Feature text the guide's scenario example binds.
const FEATURE: &str = include_str!("../../tests/features/bevy_scenario.feature");

/// Marker identifying the guide's scenario fence.
const MARKER: &str = "bevy-harness-scenario";

#[test]
fn every_published_pattern_binds_a_step_of_the_selected_scenario() {
    let fence = fence_body(super::GUIDE, MARKER).expect("the guide carries the scenario fence");
    let (path, index) = scenario_target(fence).expect("the fence names its scenario");
    assert_eq!(
        path, "tests/features/bevy_scenario.feature",
        "the fence must bind the feature this crate ships"
    );
    let steps = feature_scenario_steps(FEATURE, index);
    let patterns = published_patterns(fence);

    assert_eq!(
        steps.len(),
        5,
        "scenario index {index} must have the five steps this test checks, got {steps:?}"
    );
    for (keyword, text) in &steps {
        let is_bound = patterns.iter().any(|(pattern_keyword, pattern)| {
            *pattern_keyword == *keyword && pattern_matches(pattern, text)
        });
        assert!(
            is_bound,
            "the {keyword} step {text:?} has no published pattern of that keyword; published: \
             {patterns:?}"
        );
    }
}

/// Returns the body of the fence introduced by `marker` in `guide`.
fn fence_body<'a>(guide: &'a str, marker: &str) -> Option<&'a str> {
    let marker_line = format!("<!-- tested-example: {marker} -->");
    let after_marker = guide.get(guide.find(&marker_line)?..)?;
    let content = after_marker.get(after_marker.find("```")? + 3..)?;
    let body = content.get(content.find('\n')? + 1..)?;
    body.get(..body.find("```")?)
}

/// Returns the fence's `path` and `index` attribute values.
fn scenario_target(fence: &str) -> Option<(&str, usize)> {
    let path = attribute_value(fence, "path")?;
    let index_text = fence.split_once("index = ")?.1;
    let digits: String = index_text
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    Some((path, digits.parse().ok()?))
}

/// Returns the quoted value of the `name = "value"` attribute in `fence`.
fn attribute_value<'a>(fence: &'a str, name: &str) -> Option<&'a str> {
    let start = fence.find(&format!("{name} = \""))? + name.len() + 4;
    let rest = fence.get(start..)?;
    rest.get(..rest.find('"')?)
}

/// Returns the `(keyword, pattern)` pairs the fence publishes as steps.
fn published_patterns(fence: &str) -> Vec<(&str, &str)> {
    let mut patterns = Vec::new();
    for raw_line in fence.lines() {
        let line = raw_line.trim_start();
        for (prefix, keyword) in [
            ("#[given(\"", "Given"),
            ("#[when(\"", "When"),
            ("#[then(\"", "Then"),
        ] {
            let Some(rest) = line.strip_prefix(prefix) else {
                continue;
            };
            if let Some(pattern) = rest.strip_suffix("\")]") {
                patterns.push((keyword, pattern));
            }
        }
    }
    patterns
}

/// Returns the `(keyword, text)` steps of the scenario at zero-based `index`.
///
/// The scenario macro's `index` attribute is zero-based, so `index = 1`
/// selects the file's second scenario.
fn feature_scenario_steps(feature: &str, index: usize) -> Vec<(&str, &str)> {
    let mut current = 0;
    let mut previous = "Given";
    let mut steps = Vec::new();
    for line in feature.lines().map(str::trim) {
        if line.starts_with("Scenario:") {
            current += 1;
            previous = "Given";
            continue;
        }
        if current != index + 1 {
            continue;
        }
        if let Some(step) = classified_step(line, &mut previous) {
            steps.push(step);
        }
    }
    steps
}

/// Returns the step `line` describes, resolving `And`/`But` to `previous`.
fn classified_step<'a>(
    line: &'a str,
    previous: &mut &'static str,
) -> Option<(&'static str, &'a str)> {
    for (prefix, keyword) in [("Given ", "Given"), ("When ", "When"), ("Then ", "Then")] {
        if let Some(text) = line.strip_prefix(prefix) {
            *previous = keyword;
            return Some((keyword, text));
        }
    }
    for prefix in ["And ", "But "] {
        if let Some(text) = line.strip_prefix(prefix) {
            return Some((*previous, text));
        }
    }
    None
}

/// Whether `pattern`, with `{name:type}` placeholders, matches `text`.
fn pattern_matches(pattern: &str, text: &str) -> bool {
    let mut remaining_text = text;
    let mut remaining_pattern = pattern;
    loop {
        let Some(open) = remaining_pattern.find('{') else {
            return remaining_text == remaining_pattern;
        };
        let (literal, after_open) = remaining_pattern.split_at(open);
        let Some(text_rest) = remaining_text.strip_prefix(literal) else {
            return false;
        };
        let Some(close) = after_open.find('}') else {
            return false;
        };
        let Some(placeholder) = after_open.get(1..close) else {
            return false;
        };
        let Some(tail) = after_open.get(close + 1..) else {
            return false;
        };
        let next_literal = tail.split('{').next().unwrap_or_default();
        if next_literal.is_empty() {
            return !text_rest.is_empty() && placeholder_binds(placeholder, text_rest);
        }
        let Some(position) = text_rest.find(next_literal) else {
            return false;
        };
        let Some(captured) = text_rest.get(..position) else {
            return false;
        };
        if !placeholder_binds(placeholder, captured) {
            return false;
        }
        let Some(remaining) = text_rest.get(position..) else {
            return false;
        };
        remaining_text = remaining;
        remaining_pattern = tail;
    }
}

/// Whether `captured` satisfies the placeholder `spec`'s type.
fn placeholder_binds(spec: &str, captured: &str) -> bool {
    if captured.is_empty() {
        return false;
    }
    match spec.split_once(':').map_or("", |(_, ty)| ty) {
        "u32" => captured.chars().all(|c| c.is_ascii_digit()),
        _ => true,
    }
}

#[rstest]
#[case("the counter reads {value:u32}", "the counter reads 3", true)]
#[case("the counter reads {value:u32}", "the counter reads three", false)]
#[case("advances {count:u32} times", "advances 2 times", true)]
#[case("advances {count:u32} times", "advances 2 ticks", false)]
#[case("a plain pattern", "a plain pattern", true)]
#[case("a plain pattern", "another pattern", false)]
fn placeholder_matching_is_decided_by_the_pattern(
    #[case] pattern: &str,
    #[case] text: &str,
    #[case] expected: bool,
) {
    assert_eq!(
        pattern_matches(pattern, text),
        expected,
        "pattern {pattern:?} against {text:?}"
    );
}
