use funck::{CallError, CallResult, Request, Response};

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct PersonRequest {
    pub first_name: String,
    pub last_name: String,
}

#[derive(Serialize)]
pub struct PersonResponse {
    pub greeting: String,
}

#[derive(Debug, Default)]
pub struct TestFn;

impl TestFn {
    fn run(&self, request: Request) -> CallResult<Response> {
        let person: PersonRequest =
            serde_json::from_slice(request.body()).map_err(|e| CallError::new(e.to_string()))?;
        let greeting = format!(
            "Hello there,  Mr. {} {}",
            person.first_name, person.last_name
        );

        let resp = Response::new()
            .with_meta("Content-Type", "application/json")
            .with_bytes(
                serde_json::to_vec(&PersonResponse { greeting })
                    .map_err(|e| CallError::new(e.to_string()))?,
            );

        Ok(resp)
    }
}

funck::export!(TestFn, TestFn::run, "testfn");
