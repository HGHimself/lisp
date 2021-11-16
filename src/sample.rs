#[global_allocator]
pub static ALLOCATOR: crate::alloc::Tracing = crate::alloc::Tracing::new();

use argh::FromArgs;

#[derive(FromArgs)]
/// Run sample code
#[argh(subcommand, name = "sample")]
pub struct Sample {}

impl Sample {
    pub fn run(self) {
        let line = "* 9 9";
        ALLOCATOR.set_active(true);
        {
            let ast = match crate::parser::parse(&line) {
                Ok(tup) => tup.1,
                Err(e) => {
                    println!("Could not parse Lval! {}", e);
                    crate::Lval::Num(0_f64)
                }
            };
            // println!("{:?}", ast);
            let mut env = crate::init_env();
            crate::eval::eval(&mut env, ast);
        }
        ALLOCATOR.set_active(false);
    }
}
