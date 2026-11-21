//! Unit tests for the guide's `CommonMark` fence scanner.
//!
//! Each fixture exercises one recognition rule: tilde and backtick openers,
//! up-to-three-space indentation, closing-delimiter matching, block-quote
//! nesting depth, list-item content indentation, tab padding after a quote
//! marker, marker attachment, and the end of a contained block. The guide-wide
//! assertions live in the parent module.

use rstest::rstest;

use super::{Fence, parse_fences};

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
