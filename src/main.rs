// in `src/main.rs`

use argh::FromArgs;

#[derive(FromArgs)]
/// Small string demo
struct Args {
    #[argh(subcommand)]
    subcommand: Subcommand,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Subcommand {
    Sample(lisp::sample::Sample),
    Report(lisp::report::Report),
    Prompt(lisp::prompt::Prompt),
}

impl Subcommand {
    fn run(self) {
        match self {
            Subcommand::Sample(x) => x.run(),
            Subcommand::Report(x) => x.run(),
            Subcommand::Prompt(x) => x.run().unwrap(),
        }
    }
}

fn main() {
    // see https://turbo.fish/
    argh::from_env::<Args>().subcommand.run();
}
