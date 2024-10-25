use std::io::{Error, Write};

use log::info;

use crate::actors::{
    actor::Actor,
    messages::{Message, ResponseMessage},
};

// Generic blocking actor that will be used to collect data via an http call.
pub struct Collector;

impl Actor for Collector
where
    Collector: 'static,
{
    fn receive(&mut self, message: Message) {
        // do nothing
        info!("Received message in blocking actor via ActorTrait");
        if let Message::GetUrl {
            url,
            output,
            responder,
        } = message
        {
            info!(
                "Blocking actor received string message to get url at: {}",
                url
            );
            match self.get_uri(url, output) {
                Ok(_) => {
                    info!("Successfully got the URI");
                    let _ = responder.send(ResponseMessage::Success);
                }
                Err(e) => {
                    info!("Failed to get the URI due to {:?}", e);
                    let _ = responder.send(ResponseMessage::Failure);
                }
            }
        }
    }
}

// Collector requires spawn blocking in order to get the response from the reqwest::blocking::get method.
impl Collector {
    fn get_uri(&self, uri: String, location: String) -> Result<(), Error> {
        if cfg!(test) {
            return Ok(());
        }
        info!("Getting URI {}", uri);
        info!("Writing to location {}", location);
        let res = reqwest::blocking::get(uri).unwrap().text().unwrap();
        self.write_to_file(res, location).unwrap();
        Ok(())
    }

    fn write_to_file(&self, body: String, location: String) -> Result<(), Error> {
        if cfg!(test) {
            return Ok(());
        }
        info!("Writing to file - {}", location);
        let mut file = std::fs::File::create(location).unwrap();
        file.write_all(body.as_bytes()).unwrap();
        Ok(())
    }
}

/// An actor that can serialize and deserialize messages between generic types and json objects.
struct SerdeActor<T> where T: Serialize + Deserialize + Send + Clone + 'static {

}

impl<T> Actor for SerdeActor<T> where T: Serialize + Deserialize + Send + Clone + 'static {
    fn receive(&mut self, message: MessageImpl<T>) {

        match message {
            Message::Serialize {
                object,
                responder,
            } => {
                let serialized = serde_json::to_string(object).unwrap();
                let _ = responder.send(ResponseMessage::Response(serialized));
            }
            Message::Deserialize {
                message,
                path,
                responder,
            } => {
                let deserialized: T = serde_json::from_str(&message).unwrap();
                self.content = deserialized;
                let _ = responder.send(ResponseMessageImpl<T>::ResponseDeserialized("Ok".to_string(), deserialized));
            }
            _ => {
                // do nothing
            }
        }
        // do nothing
    }
}


