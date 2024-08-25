use std::io::{Error, Write};

use log::info;

use crate::actors::{
    actor::ActorTrait,
    messages::{InternalMessage, ResponseMessage},
};

// Generic blocking actor that will be used to collect data via an http call.
pub struct Collector;

impl ActorTrait for Collector
where
    Collector: 'static,
{
    fn receive(&mut self, message: InternalMessage) {
        // do nothing
        info!("Received message in blocking actor via ActorTrait");
        if let InternalMessage::GetUrl {
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
