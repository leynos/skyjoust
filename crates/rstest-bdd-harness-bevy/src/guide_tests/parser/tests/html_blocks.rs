//! Regression tests for the `CommonMark` HTML-block findings.
//!
//! Each test pins one layout where HTML and Markdown interact: blocks that
//! separate or contain fences, block boundaries at containers, autolinks,
//! raw-text elements, inline tags that cannot interrupt a paragraph, and
//! standalone closing tags.

use super::{parse_fences, single_fence};

#[test]
fn an_html_block_separating_a_marker_from_its_fence_is_rejected() {
    // The HTML line is content, so it may not stand between a marker and the
    // fence the marker claims; only blank lines may.
    let error = parse_fences(concat!(
        "<!-- tested-example: detached -->\n",
        "<div>text</div>\n",
        "\n",
        "```rust\n",
        "let value = 1;\n",
        "```\n",
    ))
    .expect_err("HTML between a marker and its fence must be rejected");

    assert!(
        error.contains("not followed by a fence"),
        "the error must report the separated marker, got {error:?}"
    );
}

#[test]
fn a_fence_inside_an_html_block_is_rejected_without_a_marker() {
    // Without a marker the fence alone must still be refused, so this fails
    // even if only marker detection inside HTML were implemented.
    let error = parse_fences(concat!(
        "<details>\n",
        "```rust\n",
        "let value = 1;\n",
        "```\n",
        "</details>\n",
    ))
    .expect_err("a bare fence inside an HTML block must be rejected");

    assert!(
        error.contains("HTML block") && error.contains("line 2"),
        "the error must name the HTML block and the opening fence on line 2, got {error:?}"
    );
}

#[test]
fn an_html_block_ends_with_its_container() {
    // The block quote ends before the marker, so the HTML it opened ends too
    // and the marker must be accepted at the top level.
    let fence = single_fence(
        concat!(
            "> <details>\n",
            ">\n",
            "<!-- tested-example: after-quote -->\n",
            "```rust\n",
            "let value = 1;\n",
            "```\n",
        ),
        "a top-level fence after a quoted HTML block",
    );

    assert_eq!(
        fence.marker,
        Some("after-quote"),
        "the container boundary must end the HTML block"
    );
}

#[test]
fn an_autolink_is_not_an_html_block() {
    let fence = single_fence(
        concat!(
            "<https://example.org>\n",
            "<!-- tested-example: after-autolink -->\n",
            "```rust\n",
            "let value = 1;\n",
            "```\n",
        ),
        "a fence after an autolink",
    );

    assert_eq!(
        fence.marker,
        Some("after-autolink"),
        "an autolink is paragraph content, not an HTML block"
    );
}

#[test]
fn mixed_punctuation_inside_html_is_content() {
    // No run of three matching characters, so this is literal HTML text.
    let fence = single_fence(
        concat!(
            "<details>\n",
            "`~`\n",
            "</details>\n",
            "\n",
            "<!-- tested-example: after-html -->\n",
            "```rust\n",
            "let value = 1;\n",
            "```\n",
        ),
        "mixed punctuation inside an HTML block",
    );

    assert_eq!(
        fence.marker,
        Some("after-html"),
        "mixed punctuation must not be read as a fence delimiter"
    );
}

#[test]
fn a_raw_text_html_block_ends_only_at_its_closing_tag() {
    // `<script>` ignores blank lines, so a marked fence inside it is literal
    // text and must be rejected rather than counted as an example.
    let error = parse_fences(concat!(
        "<script>\n",
        "\n",
        "<!-- tested-example: in-script -->\n",
        "```rust\n",
        "let value = 1;\n",
        "```\n",
        "\n",
        "</script>\n",
    ))
    .expect_err("a fence inside a script element must be rejected");

    assert!(
        error.contains("HTML block"),
        "the error must name the HTML block, got {error:?}"
    );
}

#[test]
fn a_container_prefix_inside_html_is_content() {
    // A `>` inside `<details>` is literal HTML text. Opening a quote there
    // would change the container identity and end the block, exposing the
    // marker and fence that follow.
    let error = parse_fences(concat!(
        "<details>\n",
        "> quoted\n",
        "<!-- tested-example: hidden-in-html -->\n",
        "```rust\n",
        "let value = 1;\n",
        "```\n",
        "</details>\n",
    ))
    .expect_err("HTML content must not end the block by opening a container");

    assert!(
        error.contains("HTML block"),
        "the error must still report the HTML block, got {error:?}"
    );
}

#[test]
fn a_bang_line_does_not_start_a_block() {
    // `!continuation` is paragraph text, so CommonMark keeps the list item
    // open and the fence below belongs to it; dropping the item would hide
    // the fence.
    let error = parse_fences(concat!(
        "- Item\n",
        "!continuation\n",
        "\n",
        "    ```rust\n",
        "    let value = 1;\n",
        "    ```\n",
    ))
    .expect_err("an exclamation continuation must not drop the item");

    assert!(
        error.contains("without its indentation"),
        "the error must explain the unsupported continuation, got {error:?}"
    );
}

#[test]
fn a_digit_led_angle_bracket_is_not_an_html_block() {
    // `<1>` has no tag name, so it is paragraph content and the marker after
    // it must be accepted.
    let fence = single_fence(
        concat!(
            "<1>\n",
            "<!-- tested-example: numeric-text -->\n",
            "```rust\n",
            "let value = 1;\n",
            "```\n",
        ),
        "a fence after digit-led angle brackets",
    );

    assert_eq!(
        fence.marker,
        Some("numeric-text"),
        "a digit-led angle bracket is not an HTML opener"
    );
}

#[test]
fn a_marker_on_an_html_terminator_line_stays_content() {
    // The second line both carries the comment's `-->` and looks like a
    // marker, but it is still inside the comment, so the fence below is
    // unmarked and must be reported as such.
    let fence = single_fence(
        concat!(
            "<!--\n",
            "<!-- tested-example: hidden -->\n",
            "```rust\n",
            "let value = 1;\n",
            "```\n",
        ),
        "a fence whose marker line lies inside an HTML comment",
    );

    assert!(
        fence.marker.is_none(),
        "the marker stays inside the comment, so the fence is unmarked"
    );
}

#[test]
fn a_digit_led_angle_bracket_does_not_start_a_block() {
    // `<1>` is paragraph text, so the list item survives the continuation and
    // its fence stays visible to the checker.
    let error = parse_fences(concat!(
        "- Item\n",
        "<1>\n",
        "\n",
        "    ```rust\n",
        "    let value = 1;\n",
        "    ```\n",
    ))
    .expect_err("a digit-led angle bracket must not drop the list item");

    assert!(
        error.contains("without its indentation"),
        "the error must explain the unsupported continuation, got {error:?}"
    );
}

#[test]
fn a_raw_text_block_closed_on_its_opening_line_is_not_a_block() {
    let fence = single_fence(
        concat!(
            "<script>hello</script>\n",
            "<!-- tested-example: after-inline-script -->\n",
            "```rust\n",
            "let value = 1;\n",
            "```\n",
        ),
        "a fence after a script closed on its opening line",
    );

    assert_eq!(
        fence.marker,
        Some("after-inline-script"),
        "a script closed on its own line opens no block"
    );
}

#[test]
fn a_raw_text_block_closes_case_insensitively() {
    let fence = single_fence(
        concat!(
            "<script>\n",
            "\n",
            "</SCRIPT>\n",
            "\n",
            "<!-- tested-example: after-upper-script -->\n",
            "```rust\n",
            "let value = 1;\n",
            "```\n",
        ),
        "a fence after an upper-case closing tag",
    );

    assert_eq!(
        fence.marker,
        Some("after-upper-script"),
        "an upper-case closing tag must end the block"
    );
}

#[test]
fn an_inline_tag_does_not_interrupt_a_list_item() {
    // `<em>` is a complete tag but an inline element, so `CommonMark` keeps the
    // list item open across it and the fence stays inside the item.
    let text = concat!(
        "- Item\n",
        "<em>\n",
        "\n",
        "    ```rust\n",
        "    // example\n",
        "    ```\n",
    );
    let error = parse_fences(text).expect_err("a lazy continuation is rejected");
    assert!(
        error.contains("\"<em>\"") && error.contains("continues a list item"),
        "the inline tag must be reported as a lazy continuation, got {error:?}"
    );
}

#[test]
fn a_standalone_closing_tag_opens_a_blank_terminated_block() {
    // A closing tag alone on its line is HTML form 7, which a blank line ends.
    // The marker and fence after it are then inside an HTML block, not examples.
    let text = concat!(
        "</script>\n",
        "<!-- tested-example: closing -->\n",
        "```rust\n",
        "// example\n",
        "```\n",
    );
    // The block runs to the next blank line, so the marker itself is already
    // inside it and is the first thing the scanner rejects.
    let error = parse_fences(text).expect_err("a marker inside HTML is rejected");
    assert!(
        error.contains("line 2") && error.contains("HTML block"),
        "the marker on line 2 must be rejected as HTML content, got {error:?}"
    );
}

#[test]
fn a_cdata_block_hides_the_fence_inside_it() {
    // `CommonMark` HTML form 5 opens at `<![CDATA[` and ends at `]]>`, so the
    // marker and fence between them are raw HTML rather than an example.
    let text = concat!(
        "<![CDATA[\n",
        "<!-- tested-example: hidden -->\n",
        "```rust\n",
        "let value = 1;\n",
        "```\n",
        "]]>\n",
    );
    let error = parse_fences(text).expect_err("a marker inside CDATA is rejected");
    assert!(
        error.contains("line 2") && error.contains("HTML block"),
        "the marker on line 2 must be rejected as HTML content, got {error:?}"
    );
}

#[test]
fn an_indented_html_terminator_still_ends_the_block() {
    // `CommonMark` starts no indented code block inside an HTML block, so the
    // four-space terminator still closes CDATA and the marker below is an
    // ordinary Markdown marker again.
    let text = concat!(
        "<![CDATA[\n",
        "    ]]>\n",
        "<!-- tested-example: after-cdata -->\n",
        "```rust\n",
        "let value = 1;\n",
        "```\n",
    );
    let fences = parse_fences(text).expect("the terminator ends the CDATA block");
    assert_eq!(
        fences.len(),
        1,
        "the marked fence after the terminator must be recorded, got {fences:?}"
    );
}

#[test]
fn an_html_block_ending_with_its_quote_releases_the_next_container() {
    // The block opened inside the quote cannot outlive it. The list item below
    // therefore opens normally, and its own fence is recorded as unmarked
    // rather than disappearing into indented code behind the control.
    let text = concat!(
        "<!-- tested-example: control -->\n",
        "```rust\n",
        "let control = 1;\n",
        "```\n",
        "\n",
        "> <details>\n",
        "- Item\n",
        "\n",
        "    ```rust\n",
        "    let value = 1;\n",
        "    ```\n",
    );
    let fences = parse_fences(text).expect("the quote's block ends with the quote");
    assert_eq!(
        fences.len(),
        2,
        "the item's unmarked fence must be recorded, got {fences:?}"
    );
    assert!(
        fences.last().is_some_and(|fence| fence.marker.is_none()),
        "the second fence must have no marker, got {fences:?}"
    );
}
