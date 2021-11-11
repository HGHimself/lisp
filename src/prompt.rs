use ansi_term::Color;
use argh::FromArgs;
use linefeed::{Interface, ReadResult};
use std::io;

#[derive(FromArgs)]
/// Run sample code
#[argh(subcommand, name = "prompt")]
pub struct Prompt {}

impl Prompt {
    pub fn run(self) -> io::Result<()> {
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

        let mut env = crate::init_env();

        while let ReadResult::Input(line) = interface.read_line()? {
            if line == "exit" {
                return Ok(());
            }

            if line == "env" {
                println!("{:#?}", env);
            } else {
                let ast = match crate::parser::parse(&line) {
                    Ok(tup) => tup.1,
                    Err(e) => {
                        println!("{}", e);
                        crate::Lval::Num(0_f64)
                    }
                };
                // println!("{:?}", ast);
                println!("{:?}", crate::eval::eval(&mut env, ast));
            }

            interface.add_history_unique(line);
        }

        Ok(())
    }
}
