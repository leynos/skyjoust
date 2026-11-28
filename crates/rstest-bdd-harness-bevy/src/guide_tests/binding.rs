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
const MARKER_LINE: &str = "<!-- tested-example: bevy-harness-scenario -->";

/// The attribute prefix naming the feature a fence binds.
const PATH_PREFIX: &str = "path = \"";

/// The attribute prefix selecting one scenario of that feature.
const INDEX_PREFIX: &str = "index = ";

/// The step attributes the guide may publish, with their keywords.
const ATTRIBUTES: [(&str, Keyword); 3] = [
    ("#[given(\"", Keyword::Given),
    ("#[when(\"", Keyword::When),
    ("#[then(\"", Keyword::Then),
];

/// The Gherkin keywords a step may carry, with their line prefixes.
const KEYWORDS: [(&str, Keyword); 3] = [
    ("Given ", Keyword::Given),
    ("When ", Keyword::When),
    ("Then ", Keyword::Then),
];

#[test]
fn every_published_pattern_binds_a_step_of_the_selected_scenario() {
    let fence = Fence::in_guide(super::GUIDE).expect("the guide carries the scenario fence");
    assert_eq!(
        fence.path(),
        Some("tests/features/bevy_scenario.feature"),
        "the fence must bind the feature this crate ships"
    );
    let index = fence.index().expect("the fence selects a scenario");
    let steps = Feature::steps(index);
    let patterns = fence.patterns();

    assert_eq!(
        steps.len(),
        5,
        "scenario index {index} must have the five steps this test checks, got {steps:?}"
    );
    for step in &steps {
        assert!(
            patterns.iter().any(|pattern| pattern.matches(step)),
            "the {} step {:?} has no published pattern of that keyword; published: {patterns:?}",
            step.keyword,
            step.text
        );
    }
}

/// A step keyword: `Given`, `When`, or `Then`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Keyword {
    Given,
    When,
    Then,
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Given => "Given",
            Self::When => "When",
            Self::Then => "Then",
        };
        formatter.write_str(name)
    }
}

/// One step of a scenario: its keyword and the text after it.
#[derive(Debug)]
struct Step<'a> {
    keyword: Keyword,
    text: &'a str,
}

/// One published step pattern: its keyword and the `{name:type}` text.
#[derive(Debug)]
struct Pattern<'a> {
    keyword: Keyword,
    text: &'a str,
}

impl Pattern<'_> {
    /// Whether this published pattern binds `step`.
    fn matches(&self, step: &Step<'_>) -> bool {
        self.keyword == step.keyword && matches_text(self.text, step.text)
    }
}

/// The guide's scenario fence, already located in the published text.
struct Fence<'a> {
    body: &'a str,
}

impl<'a> Fence<'a> {
    /// Returns the fence body that `marker` introduces in `guide`.
    fn in_guide(guide: &'a str) -> Option<Self> {
        let after_marker = guide.get(guide.find(MARKER_LINE)?..)?;
        let content = after_marker.get(after_marker.find("```")? + 3..)?;
        let body = content.get(content.find('\n')? + 1..)?;
        Some(Self {
            body: body.get(..body.find("```")?)?,
        })
    }

    /// Returns the feature path the fence binds.
    fn path(&self) -> Option<&'a str> { quoted_after(self.body, PATH_PREFIX) }

    /// Returns the zero-based scenario index the fence selects.
    fn index(&self) -> Option<usize> {
        let start = self.body.find(INDEX_PREFIX)? + INDEX_PREFIX.len();
        let digits: String = self
            .body
            .get(start..)?
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        digits.parse().ok()
    }

    /// Returns the step patterns the fence publishes.
    fn patterns(&self) -> Vec<Pattern<'a>> { self.body.lines().filter_map(published).collect() }
}

/// Returns the quoted text following `prefix` in `text`.
fn quoted_after<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = text.get(text.find(prefix)? + prefix.len()..)?;
    rest.get(..rest.find('"')?)
}

/// Returns the step attribute `line` declares, if it declares one.
fn published(line: &str) -> Option<Pattern<'_>> {
    let trimmed = line.trim_start();
    ATTRIBUTES.iter().find_map(|(prefix, keyword)| {
        trimmed
            .strip_prefix(prefix)?
            .strip_suffix("\")]")
            .map(|text| Pattern {
                keyword: *keyword,
                text,
            })
    })
}

/// The scenarios of the guide's feature file.
struct Feature;

impl Feature {
    /// Returns the steps of the scenario at zero-based `index`.
    ///
    /// The scenario macro's `index` attribute is zero-based, so `index = 1`
    /// selects the file's second scenario.
    fn steps(index: usize) -> Vec<Step<'static>> {
        let mut current = 0;
        let mut previous = Keyword::Given;
        let mut steps = Vec::new();
        for line in FEATURE.lines().map(str::trim) {
            if line.starts_with("Scenario:") {
                current += 1;
                previous = Keyword::Given;
                continue;
            }
            if current == index + 1
                && let Some(step) = classified_step(line, &mut previous)
            {
                steps.push(step);
            }
        }
        steps
    }
}

/// Returns the step `line` describes, resolving `And`/`But` to `previous`.
fn classified_step<'a>(line: &'a str, previous: &mut Keyword) -> Option<Step<'a>> {
    if let Some((prefix, keyword)) = KEYWORDS.iter().find(|(prefix, _)| line.starts_with(prefix)) {
        *previous = *keyword;
        return line.strip_prefix(prefix).map(|text| Step {
            keyword: *keyword,
            text,
        });
    }
    let text = line
        .strip_prefix("And ")
        .or_else(|| line.strip_prefix("But "))?;
    Some(Step {
        keyword: *previous,
        text,
    })
}

/// A pattern split at its first placeholder.
struct Placeholder<'a> {
    /// Text the captured value must follow.
    literal: &'a str,
    /// The `name:type` spec between the braces.
    spec: &'a str,
    /// Everything after the closing brace.
    tail: &'a str,
}

impl<'a> Placeholder<'a> {
    /// Splits `pattern` at its first placeholder, if it has one.
    fn split(pattern: &'a str) -> Option<Self> {
        let open = pattern.find('{')?;
        let (literal, after_open) = pattern.split_at(open);
        let close = after_open.find('}')?;
        Some(Self {
            literal,
            spec: after_open.get(1..close)?,
            tail: after_open.get(close + 1..)?,
        })
    }

    /// Whether `captured` satisfies this placeholder's type.
    fn binds(&self, captured: &str) -> bool {
        !captured.is_empty()
            && match self.spec.split_once(':').map_or("", |(_, ty)| ty) {
                "u32" => captured.chars().all(|character| character.is_ascii_digit()),
                _ => true,
            }
    }
}

/// Whether `pattern`, with `{name:type}` placeholders, matches `text`.
fn matches_text(pattern: &str, text: &str) -> bool {
    let Some(placeholder) = Placeholder::split(pattern) else {
        return text == pattern;
    };
    let Some(rest) = text.strip_prefix(placeholder.literal) else {
        return false;
    };
    let next_literal = placeholder.tail.split('{').next().unwrap_or_default();
    if next_literal.is_empty() {
        return placeholder.binds(rest);
    }
    let Some(position) = rest.find(next_literal) else {
        return false;
    };
    let Some((captured, remaining)) = rest.split_at_checked(position) else {
        return false;
    };
    placeholder.binds(captured) && matches_text(placeholder.tail, remaining)
}

#[rstest]
#[case(("the counter reads {value:u32}", "the counter reads 3", true))]
#[case(("the counter reads {value:u32}", "the counter reads three", false))]
#[case(("advances {count:u32} times", "advances 2 times", true))]
#[case(("advances {count:u32} times", "advances 2 ticks", false))]
#[case(("a plain pattern", "a plain pattern", true))]
#[case(("a plain pattern", "another pattern", false))]
fn placeholder_matching_is_decided_by_the_pattern(#[case] case: (&str, &str, bool)) {
    let (pattern, text, expected) = case;

    assert_eq!(
        matches_text(pattern, text),
        expected,
        "pattern {pattern:?} against {text:?}"
    );
}
