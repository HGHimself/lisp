#[global_allocator]
pub static ALLOCATOR: crate::alloc::Tracing = crate::alloc::Tracing::new();

use argh::FromArgs;

#[derive(FromArgs)]
/// Run sample code
#[argh(subcommand, name = "sample")]
pub struct Sample {}

impl Sample {
    pub fn run(self) {
        let line = "
            + 2
            (* 2 2)
            (* 2 3)
            (* 2 4)
            (* 2 5)
            (* 2 6)
            (* 2 7)
            (+
                (* 3 3)
                (* 3 3)
                (* 3 4)
                (* 3 5)
                (* 3 6)
                (+ 3
                    (* 3 3)
                    (* 3 3)
                    (* 3 4)
                    (* 3 5)
                    (* 3 6)
                    (* 3 7)
                (+
                    (* 4 4)
                    (* 4 3)
                    (* 4 4)
                    (* 4 5)
                    (* 4 6)
                    (+ 4
                    (* 4 4)
                    (* 4 3)
                    (* 4 4)
                    (* 4 5)
                    (* 4 6)
                    (* 4 7)
                    (+
                        (* 5 5)
                        (* 5 3)
                        (* 5 4)
                        (* 5 5)
                        (* 5 6)
                        (+ 5
                        (* 5 5)
                        (* 5 3)
                        (* 5 4)
                        (* 5 5)
                        (* 5 6)
                        (* 5 7)
                        (+
                            (* 6 6)
                            (* 6 3)
                            (* 6 4)
                            (* 6 5)
                            (* 6 6)
                            ( + 6
                            (* 6 6)
                            (* 6 3)
                            (* 6 4)
                            (* 6 5)
                            (* 6 6)
                            (* 6 7)
                            (+
                                (* 7 7)
                                (* 7 3)
                                (* 7 4)
                                (* 7 5)
                                (* 7 6)
                                (* 7 7))))))))))
        ";
        // let line = "* 9 9";
        ALLOCATOR.set_active(true);
        {
            let ast = match crate::parser::parse(&line) {
                Ok(tup) => tup.1,
                Err(e) => {
                    println!("Could not parse expression! {}", e);
                    crate::Expression::Num(0_f64)
                }
            };
            // println!("{:?}", ast);
            crate::eval::eval(&ast);
        }
        ALLOCATOR.set_active(false);
    }
}
