//! Regression tests where HTML blocks meet block quotes and list items.
//!
//! An HTML block cannot outlive the container holding it, and a line that
//! leaves one container may open another block of its own. These tests pin
//! both boundaries, plus the rule that container-shaped text inside raw HTML
//! is content rather than a new container.

use super::{parse_fences, single_fence};

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

#[test]
fn a_line_that_abandons_a_block_may_open_another() {
    // The quoted `<details>` ends at the unquoted `<script>`, but that same
    // line opens a raw-text block of its own, so the marker and fence below it
    // are raw HTML rather than an example.
    let text = concat!(
        "> <details>\n",
        "<script>\n",
        "<!-- tested-example: hidden -->\n",
        "```rust\n",
        "let value = 1;\n",
        "```\n",
        "</script>\n",
    );
    let error = parse_fences(text).expect_err("a marker inside script is rejected");
    assert!(
        error.contains("line 3") && error.contains("HTML block"),
        "the marker on line 3 must be rejected as HTML content, got {error:?}"
    );
}
