use std::sync::Mutex;

use funck::{CallError, CallResult, Request, Response};

use rood::sys::notify;

use serde::{Deserialize, Serialize};

fn default_notification_title() -> String {
    String::from("Funck Notification")
}

#[derive(Deserialize)]
pub struct NotificationRequest {
    #[serde(default = "default_notification_title")]
    title: String,
    body: String,
}

#[derive(Serialize)]
pub struct NotificationResponse {
    pub message: String,
}

impl NotificationResponse {
    pub fn new(body: &str) -> NotificationResponse {
        NotificationResponse {
            message: String::from(body),
        }
    }
}

#[derive(Debug, Default)]
pub struct NotifFn {
    call_count: Mutex<u16>,
}

impl NotifFn {
    fn run(&self, request: Request) -> CallResult<Response> {
        // Read the payload sent by the user to the funck server.
        let mut notif_req: NotificationRequest =
            serde_json::from_slice(request.body()).map_err(|e| CallError::new(e.to_string()))?;

        // Get & Increment call count
        let call_count = {
            let mut count_lock = self.call_count.lock().unwrap(); // Disregard locking errors for our example.
            *count_lock = *count_lock + 1;
            *count_lock
        };

        notif_req.body += &format!(" (called {} times)", call_count);

        // Send a system notification & handle the possible errors.
        let resp = match notify::send(&notif_req.title, &notif_req.body) {
            Ok(()) => NotificationResponse::new("OK"),
            Err(e) => NotificationResponse::new(&format!("Failed to notify: {}", e.to_string())),
        };

        // Serialize the response & send back to the funck server.
        Ok(Response::new()
            .with_meta("Content-Type", "application/json")
            .with_bytes(serde_json::to_vec(&resp).unwrap()))
    }
}

/// Generate the FFI wrapper so the server can load our function once compiled to a .so file.
funck::export!(NotifFn, NotifFn::run, "notify");
