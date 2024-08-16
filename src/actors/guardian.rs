use std::io::Error;
use std::{io::ErrorKind, result::Result};
use tokio::sync::mpsc;

use super::actor::{Actor, ActorTrait};
use super::actor_ref::{self, ActorRef};
use super::{
    messages,
    officer::Officer,
};



pub trait OfficerFactory {
    fn create_officer<T: ActorTrait + 'static>(&mut self, officer_type: T) -> Result<(), Error>;
    fn remove_officer(&mut self, officer_id: u32) -> Result<(), Error>;
    fn add_courrier<T: ActorTrait + 'static>(&mut self, officer_id: u32, courrier_type: T) -> Result<(), Error>;
    fn remove_courrier(&mut self, officer_id: u32, courrier_id: u32) -> Result<(), Error>;
}

pub(super) struct Guardian {
    pub (super) receiver: mpsc::Receiver<messages::Message>,
    pub (super) officers: Vec<Box<Officer>>,
}


impl Guardian {
    pub(super) fn new(receiver: tokio::sync::mpsc::Receiver::<messages::Message>) -> Guardian {
        Guardian {
            receiver,
            officers: Vec::new(),
        }
    }

    pub (super) async fn receive(&mut self, message: messages::Message) {
        // need to examine what to do with messages sent to the guardian
        // can we pass an actor_ref to the officer?
        
    }
    

}

impl OfficerFactory for Guardian {
    fn create_officer<T: ActorTrait + 'static>(&mut self, officer_type: T) -> Result<(), Error> {
        // How to pass the actor ref to the officer?
        let officer_id = self.officers.len() as u32;
        let (snd, rec) = tokio::sync::mpsc::channel::<messages::InternalMessage>(
            super::SIDS_DEFAULT_BUFFER_SIZE,
        );
        let actor = Actor::new(officer_type, rec);
        let actor_ref = ActorRef::new(actor, snd);
        let officer = Officer::new(actor_ref);
        self.officers.push(Box::new(officer)); 
        Ok(())
    }

    fn remove_officer(&mut self, officer_id: u32) -> Result<(), Error> {
        let _ = match self.officers.get_mut(officer_id as usize) {
            Some(officer) => officer,
            None => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("No Officer found with id {}.", officer_id),
                ));
            }
        };
        // do we terminate the officer here?
        //officer_to_remove.send(messages::Message::Terminate).await;
        self.officers.remove(officer_id as usize);
        Ok(())
    }

    fn add_courrier<T: ActorTrait + Sized + 'static>(&mut self, officer_id: u32, courrier: T) -> Result<(), Error> {
        let officer = match self.officers.get_mut(officer_id as usize) {
            Some(officer) => officer,
            None => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("No Officer found with id {}.", officer_id),
                ));
            }
        };
        let (snd, rec) = tokio::sync::mpsc::channel::<messages::InternalMessage>(
            super::SIDS_DEFAULT_BUFFER_SIZE,
        );
        let actor = Actor::new(courrier, rec);
        let actor_ref = actor_ref::ActorRef::new(actor, snd);         
        officer.subscribe(actor_ref);
        Ok(())
    }

    fn remove_courrier(&mut self, officer_id: u32, courrier_id: u32) -> Result<(), Error> {
        let officer = match self.officers.get_mut(officer_id as usize) {
            Some(officer) => officer,
            None => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("No Officer found with id {}.", officer_id),
                ));
            }
        };
        match officer.courriers.get(courrier_id as usize) {
            Some(_) => (),
            None => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!("No Courrier found with id {}.", courrier_id),
                ));
            }
        };
        
        officer.unsubscribe(courrier_id);
        
        Ok(())
    }
}

// grcov-excl-start
#[cfg(test)]
mod tests {
    use super::*;
    use messages::ResponseMessage;
    use tokio::sync::oneshot;



    #[tokio::test]
    async fn test_guardian_actor() {
        let (_tx, rx) = mpsc::channel(1);
        let mut guardian = Guardian::new(rx);


    }

    

    #[tokio::test]
    async fn test_guardian_actor_get_next_id() {
        let (_tx, rx) = mpsc::channel(1);
        let mut guardian = Guardian::new(rx);
        let (tx, rx) = oneshot::channel::<ResponseMessage>();
    
    }

    #[tokio::test]
    async fn test_guardian_send_features() {
        let (_tx, rx) = mpsc::channel(1);
        let mut guardian = Guardian::new(rx);
        let (tx, rx) = oneshot::channel::<ResponseMessage>();
        
           
        
    }

    #[tokio::test]
    async fn test_guardian_actor_creates_courriers() {
        let (_tx, rx) = mpsc::channel(1);
        let mut guardian = Guardian::new(rx);

        
    }
}

// grcov-excl-stop
