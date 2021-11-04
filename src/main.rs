use ansi_term::Color;
use linefeed::{Interface, ReadResult};
use std::io;

// #[global_allocator]
// pub static ALLOCATOR: lisp::alloc::Tracing = lisp::alloc::Tracing::new();

fn main() -> io::Result<()> {
    let interface = Interface::new("color-demo")?;

    let style = Color::Red.bold();
    let text = "lisp> ";

    // The character values '\x01' and '\x02' are used to indicate the beginning
    // and end of an escape sequence. This informs linefeed, which cannot itself
    // interpret the meaning of escape sequences, that these characters are not
    // visible when the prompt is drawn and should not factor into calculating
    // the visible length of the prompt string.
    interface.set_prompt(&format!(
        "\x01{prefix}\x02{text}\x01{suffix}\x02",
        prefix = style.prefix(),
        text = text,
        suffix = style.suffix()
    ))?;

    while let ReadResult::Input(line) = interface.read_line()? {
        if line == "exit" {
            return Ok(());
        }
        let ast = match lisp::parser::parse(&line) {
            Ok(tup) => tup.1,
            Err(e) => {
                println!("Could not parse expression! {:?}", e);
                lisp::Expression::Num(0_f64)
            }
        };

        println!("{:?}", ast);
        println!("{:?}", lisp::eval::eval(&ast));

        interface.add_history_unique(line);
    }

    Ok(())
}
