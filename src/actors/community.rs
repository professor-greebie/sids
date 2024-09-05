/// The community of preset actors that can be spawned in the actor system.
/// 
/// The community is a collection of actors that can be spawned in the actor system. The actors are defined in the `community` module.
/// 
/// 

#[cfg(feature = "streams")]
pub mod source;
pub mod collector;
pub mod generic;