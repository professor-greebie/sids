use std::any::Any;
use crate::actors::actor::Actor;

use super::actor::A;

pub struct Props {
    actor: Box<dyn A>,
    args : Vec<Box<dyn Any>>

}