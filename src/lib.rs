use redacted_debug::RedactedDebug;

use failure::{Error, format_err};
use reqwest;
use serde::{Serialize, Serializer};

static MESSAGE_API_URL: &'static str = "https://api.pushover.net/1/messages.json";

#[derive(Debug)]
/// The notification with which the notification will be sent.
pub enum Priority {
    /// generate no notification/alert.
    NoNotification,
    /// always send as a quiet notification.
    QuietNotification,
    /// display as high-priority and bypass the user's quiet hours.
    HighPriority,
    /// to also require confirmation from the user.
    RequireConfirmation,
}

impl Serialize for Priority {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let pri = match self {
            Priority::NoNotification => -2,
            Priority::QuietNotification => -1,
            Priority::HighPriority => 1,
            Priority::RequireConfirmation => 2,
        };
        serializer.serialize_i8(pri)
    }
}

#[derive(Serialize, RedactedDebug)]
pub struct Notification<'a> {
    #[redacted]
    token: &'a str,
    user: &'a str,
    message: &'a str,
    // attachment - an image attachment to send with the message; see attachments for more information on how to upload files
    // device - your user's device name to send the message directly to that device, rather than all of the user's devices (multiple devices may be separated by a comma)
    title: Option<String>,
    // TODO(richo) why isn't this an actual url type
    url: Option<String>,
    url_title: Option<String>,
    priority: Option<Priority>,
    // sound - the name of one of the sounds supported by device clients to override the user's default sound choice
    // timestamp - a Unix timestamp of your message's date and time to display to the user, rather than the time your message is received by our API
}

macro_rules! setter {
    ($field:ident, $ty:ty, $doc:expr) => {
        #[doc = $doc]
        pub fn $field(mut self, $field: $ty) -> Notification<'a> {
            self.$field = Some($field);
            self
        }
    }
}

impl<'a> Notification<'a> {
    setter!(title, String, "your message's title, otherwise your app's name is used");
    setter!(url, String, "a supplementary URL to show with your message");
    setter!(url_title, String, "a title for your supplementary URL, otherwise just the URL is shown");
    setter!(priority, Priority, "The notification priority for this message");
}

#[derive(RedactedDebug)]
pub struct PushoverClient {
    #[redacted]
    token: String,
    client: reqwest::Client,
}

impl PushoverClient {
    pub fn new(token: String) -> PushoverClient {
        let client = reqwest::Client::new();

        PushoverClient {
            token,
            client,
        }
    }

    pub fn build_notification<'a>(&'a self, user: &'a str, message: &'a str) -> Notification<'a> {
        Notification {
            token: &self.token,
            user: user,
            message: message,
            title: None,
            url: None,
            url_title: None,
            priority: None,
        }
    }

    pub fn send<'a>(&'a self, notification: &'a Notification) -> Result<reqwest::Response, Error> {
        self.client
            .post(MESSAGE_API_URL)
            .form(&notification)
            .send()
            .map_err(|e| format_err!("HTTP error: {:?}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    use serde_json;

    #[test]
    fn test_serialized_priorities_dtrt() {
        let client = PushoverClient::new("".into());
        let req = client
            .build_notification("richo", "test")
            .priority(Priority::HighPriority);
        assert!(
            serde_json::to_string(&req)
                .unwrap()
                .contains("\"priority\":1"),
            "Serialization failed"
        );
    }

    #[test]
    fn test_setters_all_work() -> Result<(), Error> {
        let client = PushoverClient::new("".into());
        let notification = client.build_notification("richo", "this is a test_notification");
        let out = notification
            .title("test title".into())
            .url("http://butts.lol".into())
            .url_title("loool".into())
            .priority(Priority::HighPriority);
        // client.send(&out)?;
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_sends_notification() -> Result<(), Error> {
        let pushover = PushoverClient::new(
            env::var("ARCHIVER_TEST_PUSHOVER_KEY").expect("Didn't provide test key"),
            );
        let user_key: String = "redacted".into();
        let req = pushover.build_notification(&user_key, "hi there");
        pushover.send(&req)?;
        Ok(())
    }
}
