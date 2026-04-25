//! `Skyjoust` application entry point.

// TODO: Remove this stub and implement actual application functionality.
const GREETING: &str = "Hello from Skyjoust!";

/// Application entry point.
#[expect(clippy::print_stdout, reason = "CLI output is the intended behaviour")]
fn main() {
    println!("{GREETING}");
}

#[cfg(test)]
mod tests {
    //! Tests for the application entry point stub.

    use super::GREETING;

    #[test]
    fn greeting_names_skyjoust() {
        assert_eq!(GREETING, "Hello from Skyjoust!");
    }
}
