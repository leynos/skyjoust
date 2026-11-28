//! Unit tests for the guide's `CommonMark` fence scanner.
//!
//! Each fixture exercises one recognition rule: tilde and backtick openers,
//! up-to-three-space indentation, closing-delimiter matching, block-quote
//! nesting depth, list-item content indentation, tab padding after a quote
//! marker, the rejection of a tab exposed as indentation by a container
//! prefix, marker attachment, the blank-line allowance in its raw, quoted, and
//! whitespace-only quoted forms, the rejection of a marker in a sibling
//! container, and the end of a contained block. The guide-wide assertions live
//! in the parent module.

use rstest::rstest;

use super::{Fence, parse_fences};

#[path = "tests/conformance.rs"]
mod conformance;
#[path = "tests/html_blocks.rs"]
mod html_blocks;
#[path = "tests/html_containers.rs"]
mod html_containers;

/// Parses `text` and returns the single fence it must contain.
fn single_fence<'a>(text: &'a str, description: &str) -> Fence<'a> {
    let mut fences = parse_fences(text).expect("the fixture must parse");

    assert_eq!(
        fences.len(),
        1,
        "{description} must be recorded exactly once"
    );
    fences.pop().expect("the fixture records exactly one fence")
}

#[rstest]
#[case("~~~rust\nlet value = 1;\n~~~\n", "a tilde fence")]
#[case(
    "   ```rust\n   let value = 1;\n   ```\n",
    "an indented backtick fence"
)]
#[case("> ~~~rust\n> let value = 1;\n> ~~~\n", "a block-quoted fence")]
#[case(
    ">\t~~~rust\n>\tlet value = 1;\n>\t~~~\n",
    "a tab-padded block-quoted fence"
)]
#[case(
    "- Item\n\n    ```rust\n    let value = 1;\n    ```\n",
    "a four-space fence inside a bullet item"
)]
#[case(
    "1. Item\n\n    ```rust\n    let value = 1;\n    ```\n",
    "a four-space fence inside an ordered item"
)]
#[case(
    "- Item\n\n  > ```rust\n  > let value = 1;\n  > ```\n",
    "a quoted fence inside a bullet item"
)]
#[case(
    "> - Item\n>\n>     ```rust\n>     let value = 1;\n>     ```\n",
    "a fence in a list inside a block quote"
)]
fn unmarked_fence_is_detected(#[case] text: &str, #[case] description: &str) {
    let fence = single_fence(text, description);

    assert_eq!(
        fence.language, "rust",
        "{description} must keep the language after its delimiter"
    );
    assert!(
        fence.marker.is_none(),
        "{description} must stay unmarked so the marker check can reject it"
    );
}

#[rstest]
#[case(
    "- Item\n\n  <!-- tested-example: list-fence -->\n  ```rust\n  let value = 1;\n  ```\n",
    "list-fence",
    "a marker above a list fence"
)]
#[case(
    "<!-- tested-example: quoted-fence -->\n\n> ```rust\n> let value = 1;\n> ```\n",
    "quoted-fence",
    "a marker above a block-quoted fence"
)]
fn marker_above_a_contained_fence_is_attached(
    #[case] text: &str,
    #[case] expected: &str,
    #[case] description: &str,
) {
    let fence = single_fence(text, description);

    assert_eq!(
        fence.marker,
        Some(expected),
        "{description} must be attached to its fence"
    );
}

#[rstest]
#[case(
    "> <!-- tested-example: quoted-gap -->\n>\n> ```rust\n> let value = 1;\n> ```\n",
    "a quoted blank line"
)]
#[case(
    "> <!-- tested-example: quoted-gap -->\n>     \n> ```rust\n> let value = 1;\n> ```\n",
    "a whitespace-only quoted blank line"
)]
fn a_quoted_blank_line_keeps_the_marker_attached(#[case] text: &str, #[case] description: &str) {
    let fence = single_fence(text, description);

    assert_eq!(
        fence.marker,
        Some("quoted-gap"),
        "{description} must not detach the marker"
    );
}

#[test]
fn an_indented_code_line_does_not_excuse_a_dangling_marker() {
    let error = parse_fences("<!-- tested-example: dangling -->\n    let value = 1;\n")
        .expect_err("an indented code line must not stand in for the fence");

    assert!(
        error.contains("not followed by a fence"),
        "the error must report the dangling marker, got {error:?}"
    );
}

#[test]
fn a_marker_in_a_sibling_list_item_is_rejected() {
    let error =
        parse_fences("- <!-- tested-example: misplaced -->\n- ```rust\n  let value = 1;\n  ```\n")
            .expect_err("a marker in a sibling item must not label the next item's fence");

    assert!(
        error.contains("enclosing container"),
        "the error must explain the container mismatch, got {error:?}"
    );
}

#[test]
fn a_top_level_delimiter_does_not_close_a_list_fence() {
    let error = parse_fences("- Item\n\n    ```rust\n    let value = 1;\n```\n")
        .expect_err("a top-level delimiter must not close a list fence");

    assert!(
        error.contains("list item"),
        "the error must name the ended list item, got {error:?}"
    );
}

#[test]
fn tab_indented_list_content_is_rejected() {
    let error = parse_fences("- Item\n\n\t```rust\n\tlet value = 1;\n\t```\n")
        .expect_err("tab-indented list content must be rejected");

    assert!(
        error.contains("tab-indented"),
        "the error must explain the tab rejection, got {error:?}"
    );
}

#[test]
fn tab_indented_content_after_a_quote_prefix_is_rejected() {
    let error = parse_fences("> - Item\n>\n> \t```rust\n> \tlet value = 1;\n> \t```\n")
        .expect_err("a tab exposed after a block-quote prefix must be rejected");

    assert!(
        error.contains("tab-indented"),
        "the error must explain the tab rejection, got {error:?}"
    );
}

#[test]
fn closer_with_a_different_character_does_not_close() {
    let error =
        parse_fences("~~~rust\n```\n").expect_err("a backtick line must not close a tilde fence");

    assert!(
        error.contains("not terminated"),
        "the error must report the unterminated fence, got {error:?}"
    );
}

#[test]
fn longer_closer_closes_the_fence() {
    let fence = single_fence(
        "```rust\nlet value = 1;\n`````\n",
        "a closer longer than its opener",
    );

    assert_eq!(
        fence.language, "rust",
        "the block keeps the opener's language"
    );
}

#[test]
fn quoted_delimiter_does_not_close_a_top_level_fence() {
    // The quoted candidate matches the opener's character and length, so only
    // the depth comparison can reject it as a closer.
    let fence = single_fence(
        "```text\n> ```\n```\n",
        "a quoted delimiter inside a top-level fence",
    );

    assert_eq!(
        fence.language, "text",
        "the fence keeps its opener's language"
    );
}

#[test]
fn unquoted_delimiter_ends_a_quoted_fence() {
    let error = parse_fences("<!-- tested-example: example -->\n> ~~~rust\n```\n> ~~~\n")
        .expect_err("an unquoted delimiter must end the block quote");

    assert!(
        error.contains("block quote"),
        "the error must name the ended block quote, got {error:?}"
    );
}

#[test]
fn unquoted_blank_line_ends_a_quoted_fence() {
    let error =
        parse_fences("<!-- tested-example: first -->\n> ~~~rust\n> let value = 1;\n\n> ~~~\n")
            .expect_err("an unquoted blank line must end the block quote");

    assert!(
        error.contains("block quote"),
        "the error must name the ended block quote, got {error:?}"
    );
}

#[test]
fn a_marker_shaped_line_inside_a_fence_is_content() {
    let fence = single_fence(
        "<!-- tested-example: outer -->\n```markdown\n<!-- tested-example: inner -->\n```\n",
        "a fence whose content shows the marker syntax",
    );

    assert_eq!(
        fence.marker,
        Some("outer"),
        "the fence keeps the marker that precedes it"
    );
}

#[test]
fn a_non_breaking_space_after_a_closer_is_content() {
    // `CommonMark` allows only spaces and tabs after a closing delimiter, but
    // `str::trim` also strips U+00A0, so a naive suffix test would close the
    // fence here and then read the apparent Rust block as a second fence.
    let error = parse_fences(concat!(
        "<!-- tested-example: outer -->\n",
        "~~~markdown\n",
        "~~~\u{a0}\n",
        "<!-- tested-example: inner -->\n",
        "```rust\n",
        "let value = 1;\n",
        "```\n",
    ))
    .expect_err("a non-breaking space must not close the fence");

    assert!(
        error.contains("not terminated"),
        "the error must report the unterminated plaintext fence, got {error:?}"
    );
}

#[test]
fn a_tab_after_a_closer_is_accepted() {
    let fence = single_fence(
        concat!(
            "<!-- tested-example: tabbed -->\n",
            "```rust\n",
            "let value = 1;\n",
            "```\t\n",
        ),
        "a closer with a trailing tab",
    );

    assert_eq!(
        fence.marker,
        Some("tabbed"),
        "a tab after the closing delimiter is allowed by CommonMark"
    );
}

#[test]
fn a_fence_inside_an_html_block_is_rejected() {
    // Without a blank line these lines form one HTML block, so rustdoc never
    // sees a Rust example; recording one would overstate the guide's coverage.
    let error = parse_fences(concat!(
        "<details>\n",
        "<summary>Example</summary>\n",
        "<!-- tested-example: hidden -->\n",
        "```rust\n",
        "let value = 1;\n",
        "```\n",
        "</details>\n",
    ))
    .expect_err("a fence inside an HTML block must be rejected");

    assert!(
        error.contains("HTML block"),
        "the error must explain the HTML block, got {error:?}"
    );
}

#[test]
fn recognition_resumes_after_a_blank_line_ends_an_html_block() {
    let fence = single_fence(
        concat!(
            "<details>\n",
            "<summary>Example</summary>\n",
            "\n",
            "<!-- tested-example: visible -->\n",
            "```rust\n",
            "let value = 1;\n",
            "```\n",
            "</details>\n",
        ),
        "a fence after a blank line ending an HTML block",
    );

    assert_eq!(
        fence.marker,
        Some("visible"),
        "the blank line ends the HTML block, so the fence is Markdown again"
    );
}

#[test]
fn a_lazy_paragraph_continuation_is_rejected() {
    // CommonMark keeps the list item open across the unindented continuation,
    // so the four-space fence below belongs to the item. Dropping the item
    // instead would make the fence disappear from the scanner's output.
    let error = parse_fences("- Item\ncontinuation\n\n    ```rust\n    let value = 1;\n    ```\n")
        .expect_err("a lazy continuation must not silently drop its fence");

    assert!(
        error.contains("without its indentation"),
        "the error must explain the unsupported continuation, got {error:?}"
    );
}

#[test]
fn an_indented_continuation_keeps_the_list_item() {
    let fence = single_fence(
        "- Item\n  continuation\n\n    ```rust\n    let value = 1;\n    ```\n",
        "a list item continued at its content column",
    );

    assert!(
        fence.marker.is_none(),
        "the fence stays visible so the marker check can reject it"
    );
}

#[test]
fn a_backtick_in_the_info_string_is_not_an_opener() {
    // The middle line is an inline code span, not a fence. Read as an opener it
    // would consume the marker and swallow the real fence as its content; read
    // correctly it is prose, which may not separate a marker from its fence.
    let error =
        parse_fences("<!-- tested-example: real -->\n```rust```\n```rust\nlet value = 1;\n```\n")
            .expect_err("an inline code span must not consume the marker");

    assert!(
        error.contains("not followed by a fence"),
        "the error must report the dangling marker rather than a stray fence, got {error:?}"
    );
}
