use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Event {
    request: Request,
    response: Response,
}

impl Event {
    #[cfg(test)]
    pub fn new(request: Request, response: Response) -> Self {
        Self { request, response }
    }

    pub fn request(&self) -> &Request {
        &self.request
    }

    pub fn response(&self) -> &Response {
        &self.response
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Request {
    source: String,
    answers: HashMap<String, Answer>,
}

impl Request {
    #[cfg(test)]
    pub fn new(source: &str, answers: HashMap<String, Answer>) -> Self {
        Self {
            source: String::from(source),
            answers,
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn answers(&self) -> &HashMap<String, Answer> {
        &self.answers
    }
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum Answer {
    Simple(String),
    Collection(Vec<HashMap<String, String>>),
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Response {
    id: String,
}

impl Response {
    #[cfg(test)]
    pub fn new(id: &str) -> Self {
        Self {
            id: String::from(id),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_not_deserialise_invalid_event() {
        assert!(serde_json::from_str::<Event>("{}").is_err())
    }

    #[test]
    fn deserialises_event() {
        let actual: Event = serde_json::from_str(event()).unwrap();
        assert_eq!(actual, expected())
    }

    fn expected() -> Event {
        let mut answers = HashMap::new();
        answers.insert(
            String::from("first_name"),
            Answer::Simple(String::from("Tim")),
        );
        answers.insert(
            String::from("pets"),
            Answer::Collection(vec![HashMap::from([
                (String::from("type"), String::from("Cat")),
                (String::from("name"), String::from("Tiffin")),
            ])]),
        );
        Event {
            request: Request {
                source: String::from("somewhere"),
                answers,
            },
            response: Response {
                id: String::from("1234"),
            },
        }
    }

    fn event() -> &'static str {
        r#"{"request":{"source":"somewhere","answers":{"first_name":"Tim","pets":[{"type":"Cat","name":"Tiffin"}]}}, "response":{"id":"1234"}}"#
    }
}
