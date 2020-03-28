#[derive(Debug, Default)]
pub struct TestFn;

impl TestFn {
    fn run(&self) {
        println!("I just got called");
    }
}

funck::export!(TestFn, TestFn::run, "hello_fn");
