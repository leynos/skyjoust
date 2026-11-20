//! Unit tests for the guide's CommonMark fence scanner.
//!
//! Each fixture exercises one recognition rule: tilde and backtick openers,
//! up-to-three-space indentation, closing-delimiter matching, block-quote
//! nesting depth, tab padding after a quote marker, marker attachment, and the
//! end of a quoted container. The guide-wide assertions live in the parent
//! module.

use super::parse_fences;

#[test]
fn unmarked_tilde_fence_is_detected() {
    let fences = parse_fences("~~~rust\nlet value = 1;\n~~~\n").expect("the tilde fence parses");

    assert_eq!(
        fences.len(),
        1,
        "a tilde fence must be recorded exactly once"
    );
    let fence = fences
        .first()
        .expect("the fixture records exactly one fence");
    assert_eq!(
        fence.language, "rust",
        "the text after the delimiter names the language"
    );
    assert!(
        fence.marker.is_none(),
        "the fixture leaves the fence unmarked so the marker check can reject it"
    );
}

#[test]
fn unmarked_indented_fence_is_detected() {
    let fences =
        parse_fences("   ```rust\n   let value = 1;\n   ```\n").expect("the indented fence parses");

    assert_eq!(
        fences.len(),
        1,
        "an indented backtick fence must be recorded exactly once"
    );
    let fence = fences
        .first()
        .expect("the fixture records exactly one fence");
    assert!(
        fence.marker.is_none(),
        "the fixture leaves the fence unmarked so the marker check can reject it"
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
    let fences =
        parse_fences("```rust\nlet value = 1;\n`````\n").expect("a longer closer parses the block");

    assert_eq!(
        fences.len(),
        1,
        "a closer longer than its opener must still close the block"
    );
    let fence = fences
        .first()
        .expect("the fixture records exactly one fence");
    assert_eq!(
        fence.language, "rust",
        "the block keeps the opener's language"
    );
}

#[test]
fn unmarked_block_quoted_fence_is_detected() {
    let fences = parse_fences("> ~~~rust\n> let value = 1;\n> ~~~\n")
        .expect("the block-quoted fence parses");

    assert_eq!(
        fences.len(),
        1,
        "a fence inside a block quote must be recorded exactly once"
    );
    let fence = fences
        .first()
        .expect("the fixture records exactly one fence");
    assert_eq!(
        fence.language, "rust",
        "the info string inside the quote names the language"
    );
    assert!(
        fence.marker.is_none(),
        "the fixture leaves the fence unmarked so the marker check can reject it"
    );
}

#[test]
fn marker_above_a_block_quoted_fence_is_attached() {
    let fences = parse_fences(
        "<!-- tested-example: quoted-fence -->\n\n> ```rust\n> let value = 1;\n> ```\n",
    )
    .expect("the marked block-quoted fence parses");

    let marker = fences.first().and_then(|fence| fence.marker);
    assert_eq!(
        marker,
        Some("quoted-fence"),
        "a marker above a block-quoted fence must be attached to it"
    );
}

#[test]
fn quoted_delimiter_does_not_close_a_top_level_fence() {
    // The quoted candidate matches the opener's character and length, so only
    // the depth comparison can reject it as a closer.
    let fences = parse_fences("```text\n> ```\n```\n").expect("the top-level fence parses");

    assert_eq!(
        fences.len(),
        1,
        "a quoted delimiter is content inside a top-level fence"
    );
    assert_eq!(
        fences
            .first()
            .map(|fence| fence.language)
            .expect("the fixture records exactly one fence"),
        "text",
        "the fence keeps its opener's language"
    );
}

#[test]
fn tab_after_the_quote_marker_is_accepted() {
    let fences = parse_fences(">\t~~~rust\n>\tlet value = 1;\n>\t~~~\n")
        .expect("the tab-padded block-quoted fence parses");

    assert_eq!(
        fences.len(),
        1,
        "a fence whose quote marker is padded with a tab must be recorded exactly once"
    );
    let fence = fences
        .first()
        .expect("the fixture records exactly one fence");
    assert!(
        fence.marker.is_none(),
        "the fixture leaves the fence unmarked so the marker check can reject it"
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
