#[derive(Debug, Default)]
pub struct TestFn {}

impl funck::Funcktion for TestFn {
    fn name(&self) -> &'static str {
        "hello_fn"
    }

    fn call(&self) {
        println!("I just got called");
    }
}

funck::export!(TestFn, TestFn::default);
