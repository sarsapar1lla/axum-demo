use std::collections::HashMap;

use crate::model::{Answer, Event, Notification};

pub fn apply(event: &Event, notification: &Notification) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert(String::from("id"), String::from(event.response().id()));
    map.insert(String::from("created"), notification.created().to_rfc3339());
    map.insert(
        String::from("s3_uri"),
        format!("s3://{}/{}", notification.bucket(), notification.key()),
    );

    let answers: HashMap<String, String> = event
        .request()
        .answers()
        .iter()
        .flat_map(|(key, answer)| transform_answer(key, answer))
        .collect();

    map.extend(answers);

    map
}

fn transform_answer(key: &str, answer: &Answer) -> Vec<(String, String)> {
    match answer {
        Answer::Simple(value) => vec![(String::from(key), String::from(value))],
        Answer::Collection(collection) => collection
            .iter()
            .enumerate()
            .flat_map(|(idx, value)| transform_collection_answer(key, idx, value))
            .collect(),
    }
}

fn transform_collection_answer(
    parent_key: &str,
    index: usize,
    value: &HashMap<String, String>,
) -> Vec<(String, String)> {
    value
        .iter()
        .map(|(child_key, child_value)| {
            (
                format!("{}{}_{}", parent_key, index + 1, child_key),
                child_value.to_owned(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::DateTime;

    use crate::model::{Request, Response};

    use super::*;

    #[test]
    fn transforms_event() {
        let actual = apply(&event(), &notification());
        assert_eq!(actual, expected())
    }

    fn expected() -> HashMap<String, String> {
        HashMap::from([
            (String::from("id"), String::from("1234")),
            (
                String::from("created"),
                String::from("2024-08-10T11:00:00+00:00"),
            ),
            (
                String::from("s3_uri"),
                String::from("s3://test-bucket/1234.json"),
            ),
            (String::from("first_name"), String::from("Tim")),
            (String::from("pets1_type"), String::from("Cat")),
            (String::from("pets1_name"), String::from("Tiffin")),
            (String::from("pets2_type"), String::from("Dog")),
            (String::from("pets2_name"), String::from("Waldo")),
        ])
    }

    fn notification() -> Notification {
        Notification::builder()
            .message_id("some-message")
            .receipt_handle("receipt")
            .created(DateTime::from_str("2024-08-10T11:00:00Z").unwrap())
            .bucket("test-bucket")
            .key("1234.json")
            .build()
    }

    fn event() -> Event {
        Event::new(request(), response())
    }

    fn request() -> Request {
        Request::new(
            "somewhere",
            HashMap::from([
                (
                    String::from("first_name"),
                    Answer::Simple(String::from("Tim")),
                ),
                (
                    String::from("pets"),
                    Answer::Collection(vec![
                        HashMap::from([
                            (String::from("type"), String::from("Cat")),
                            (String::from("name"), String::from("Tiffin")),
                        ]),
                        HashMap::from([
                            (String::from("type"), String::from("Dog")),
                            (String::from("name"), String::from("Waldo")),
                        ]),
                    ]),
                ),
            ]),
        )
    }

    fn response() -> Response {
        Response::new("1234")
    }
}
