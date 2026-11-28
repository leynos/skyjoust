//! Regression tests for the container, blank-line, and marker findings.
//!
//! Each test pins one layout the scanner previously mis-read: thematic breaks
//! that look like list markers, Unicode blank lines, tab-separated headings and
//! list markers, unquoted lazy continuations, and quote-shaped fence content.

use super::{parse_fences, single_fence};

#[test]
fn a_thematic_break_is_not_a_list_marker() {
    // `- - -` is a thematic break, so the four-space lines after it are an
    // indented code block. Reading the break as three items would strip that
    // indentation and record a fence CommonMark sees as code.
    let fences = parse_fences(concat!(
        "- - -\n",
        "\n",
        "    <!-- tested-example: after-break -->\n",
        "    ```rust\n",
        "    let value = 1;\n",
        "    ```\n",
    ))
    .expect("the break and the code block must parse");

    assert!(
        fences.is_empty(),
        "the indented code must not be recorded as a fence, got {fences:?}"
    );
}

#[test]
fn a_non_breaking_space_line_is_not_blank() {
    // Only ASCII spaces and tabs make a line blank, so U+00A0 is content and
    // may not separate a marker from its fence.
    let error = parse_fences(concat!(
        "<!-- tested-example: nbsp-gap -->\n",
        "\u{a0}\n",
        "```rust\n",
        "let value = 1;\n",
        "```\n",
    ))
    .expect_err("a non-breaking-space line must not bridge the marker gap");

    assert!(
        error.contains("not followed by a fence"),
        "the error must report the dangling marker, got {error:?}"
    );
}

#[test]
fn an_atx_heading_may_be_followed_by_a_tab() {
    // The heading legitimately ends the list item, so the tab after the hashes
    // must not be mistaken for a lazy continuation.
    let fence = single_fence(
        concat!(
            "- Item\n",
            "#\tHeading\n\n```rust\n",
            "let value = 1;\n```\n"
        ),
        "a tab-separated heading after a list item",
    );

    assert!(
        fence.marker.is_none(),
        "the fence stays visible so the marker check can reject it"
    );
}

#[test]
fn an_unquoted_lazy_continuation_is_rejected() {
    // CommonMark keeps the quote and its item across the unquoted line, so the
    // later fence belongs to them; dropping both would hide it.
    let error = parse_fences(concat!(
        "> - Item\n",
        "continuation\n",
        ">\n",
        ">     ```rust\n",
        ">     let value = 1;\n",
        ">     ```\n",
    ))
    .expect_err("an unquoted continuation must not silently drop the quote");

    assert!(
        error.contains("without its marker"),
        "the error must explain the omitted quote prefix, got {error:?}"
    );
}

#[test]
fn a_tab_padded_list_marker_is_rejected() {
    let error = parse_fences(concat!(
        "  -\tItem\n",
        "\n    ```rust\n",
        "    let value = 1;\n    ```\n"
    ))
    .expect_err("a tab after a list marker must be rejected");

    assert!(
        error.contains("tab after a list marker"),
        "the error must explain the tab rejection, got {error:?}"
    );
}

#[test]
fn quote_shaped_lines_inside_a_fence_are_content() {
    // A `>` line inside a fence is literal text: opening a quote there would
    // end the fence's container and reject the following plain content.
    let text = concat!(
        "<!-- tested-example: quoted -->\n",
        "```text\n",
        "> quoted\n",
        "plain continuation\n",
        "```\n",
    );
    let fences = parse_fences(text).expect("quote-shaped fence content is literal");
    assert_eq!(
        fences.len(),
        1,
        "the fence must be recorded once, got {fences:?}"
    );
    let marker = fences.first().and_then(|fence| fence.marker);
    assert_eq!(
        marker,
        Some("quoted"),
        "the marker must label the fence, got {fences:?}"
    );
}
