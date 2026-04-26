//! `Skyjoust` application entry point.

use std::io::{self, Write};

// TODO: Remove this stub and implement actual application functionality.
const GREETING: &str = "Hello from Skyjoust!";

/// Application entry point.
///
/// Example output:
///
/// ```text
/// $ cargo run
/// Hello from Skyjoust!
/// ```
fn main() -> io::Result<()> { print_greeting(&mut io::stdout()) }

/// Write the Skyjoust greeting to `output`.
///
/// ```
/// # use std::io::{self, Write};
/// const GREETING: &str = "Hello from Skyjoust!";
///
/// fn print_greeting(output: &mut impl Write) -> io::Result<()> { writeln!(output, "{GREETING}") }
///
/// let mut buffer = Vec::new();
/// print_greeting(&mut buffer)?;
///
/// assert_eq!(buffer, b"Hello from Skyjoust!\n");
/// # Ok::<(), io::Error>(())
/// ```
fn print_greeting(output: &mut impl Write) -> io::Result<()> { writeln!(output, "{GREETING}") }

#[cfg(test)]
mod tests {
    //! Tests for the application entry point stub.

    use super::{GREETING, print_greeting};

    #[test]
    fn greeting_names_skyjoust() {
        let mut output = Vec::new();

        print_greeting(&mut output).expect("writing greeting to memory should succeed");

        assert_eq!(GREETING, "Hello from Skyjoust!");
        assert_eq!(output, b"Hello from Skyjoust!\n");
    }
}
