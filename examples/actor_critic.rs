//! Actor-Critic Reinforcement Learning Example
//!
//! This example demonstrates an actor-critic system using SIDS actors:
//! - Environment: Multi-armed bandit with 3 arms
//! - Actor: Learns action policy (which arm to pull)
//! - Critic: Learns value function (expected rewards)
//! - Coordinator: Manages training loop
//!
//! The actor-critic pattern is a fundamental reinforcement learning architecture
//! where the actor learns what actions to take, and the critic evaluates how good
//! those actions are, providing feedback to improve the actor's policy.

use log::{info, warn};
use sids::actors::{self, actor::Actor, get_response_handler, messages::Message, spawn_actor};
use std::collections::HashMap;

/// Messages for the actor-critic system
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum ACMessage {
    /// Request action from actor (state -> action)
    GetAction { state: u32 },
    /// Update actor policy with reward
    UpdateActor {
        state: u32,
        action: u32,
        td_error: f64,
    },
    /// Get value estimate from critic
    GetValue { state: u32 },
    /// Update critic value estimate
    UpdateCritic { state: u32, td_error: f64 },
    /// Pull arm in environment
    PullArm { arm: u32 },
    /// Run training episode
    RunEpisode { episode: u32 },
    /// Get statistics
    GetStats,
}

/// Response types for actor-critic system
#[derive(Debug, Clone)]
enum ACResponse {
    Action(u32),
    Value(f64),
    Reward(f64),
    Stats(String),
    Success,
}

/// Multi-armed bandit environment
/// Three arms with different reward probabilities
struct BanditEnvironment {
    arm_probs: Vec<f64>,
    total_pulls: u32,
    total_reward: f64,
}

impl BanditEnvironment {
    fn new() -> Self {
        Self {
            arm_probs: vec![0.1, 0.5, 0.3], // Arm 1 (10%), Arm 2 (50% - best), Arm 3 (30%)
            total_pulls: 0,
            total_reward: 0.0,
        }
    }

    fn pull_arm(&mut self, arm: u32) -> f64 {
        self.total_pulls += 1;
        let prob = self.arm_probs[arm as usize];
        let reward = if rand::random::<f64>() < prob {
            1.0
        } else {
            0.0
        };
        self.total_reward += reward;
        reward
    }
}

impl Actor<ACMessage, ACResponse> for BanditEnvironment {
    async fn receive(&mut self, message: Message<ACMessage, ACResponse>) {
        if let Some(payload) = message.payload {
            match payload {
                ACMessage::PullArm { arm } => {
                    let reward = self.pull_arm(arm);
                    if let Some(responder) = message.responder {
                        responder.handle(ACResponse::Reward(reward)).await;
                    }
                }
                ACMessage::GetStats => {
                    let avg_reward = if self.total_pulls > 0 {
                        self.total_reward / self.total_pulls as f64
                    } else {
                        0.0
                    };
                    let stats = format!(
                        "Environment Stats:\n  Total Pulls: {}\n  Total Reward: {:.1}\n  Avg Reward: {:.3}",
                        self.total_pulls, self.total_reward, avg_reward
                    );
                    if let Some(responder) = message.responder {
                        responder.handle(ACResponse::Stats(stats)).await;
                    }
                }
                _ => {}
            }
        }
    }
}

/// Actor agent - learns action policy
/// Uses softmax policy over action values
struct ActorAgent {
    action_values: HashMap<u32, f64>, // State -> action value for each arm
    learning_rate: f64,
    num_actions: u32,
}

impl ActorAgent {
    fn new(num_actions: u32) -> Self {
        let mut action_values = HashMap::new();
        for i in 0..num_actions {
            action_values.insert(i, 0.0);
        }
        Self {
            action_values,
            learning_rate: 0.1,
            num_actions,
        }
    }

    fn select_action(&self) -> u32 {
        // Softmax action selection with temperature
        let temperature = 1.0;
        let exp_values: Vec<f64> = (0..self.num_actions)
            .map(|a| (self.action_values[&a] / temperature).exp())
            .collect();
        let sum: f64 = exp_values.iter().sum();
        let probs: Vec<f64> = exp_values.iter().map(|v| v / sum).collect();

        // Sample action based on probabilities
        let mut cumsum = 0.0;
        let rand_val = rand::random::<f64>();
        for (action, &prob) in probs.iter().enumerate() {
            cumsum += prob;
            if rand_val <= cumsum {
                return action as u32;
            }
        }
        self.num_actions - 1
    }

    fn update(&mut self, action: u32, td_error: f64) {
        // Update action value using TD error
        let current = self.action_values[&action];
        self.action_values
            .insert(action, current + self.learning_rate * td_error);
    }
}

impl Actor<ACMessage, ACResponse> for ActorAgent {
    async fn receive(&mut self, message: Message<ACMessage, ACResponse>) {
        if let Some(payload) = message.payload {
            match payload {
                ACMessage::GetAction { state: _ } => {
                    let action = self.select_action();
                    if let Some(responder) = message.responder {
                        responder.handle(ACResponse::Action(action)).await;
                    }
                }
                ACMessage::UpdateActor {
                    state: _,
                    action,
                    td_error,
                } => {
                    self.update(action, td_error);
                    if let Some(responder) = message.responder {
                        responder.handle(ACResponse::Success).await;
                    }
                }
                ACMessage::GetStats => {
                    let stats = format!(
                        "Actor Stats:\n  Action Values: {:?}\n  Learning Rate: {}",
                        self.action_values, self.learning_rate
                    );
                    if let Some(responder) = message.responder {
                        responder.handle(ACResponse::Stats(stats)).await;
                    }
                }
                _ => {}
            }
        }
    }
}

/// Critic agent - learns value function
struct CriticAgent {
    value_estimate: f64,
    learning_rate: f64,
}

impl CriticAgent {
    fn new() -> Self {
        Self {
            value_estimate: 0.0,
            learning_rate: 0.1,
        }
    }

    fn update(&mut self, td_error: f64) {
        self.value_estimate += self.learning_rate * td_error;
    }
}

impl Actor<ACMessage, ACResponse> for CriticAgent {
    async fn receive(&mut self, message: Message<ACMessage, ACResponse>) {
        if let Some(payload) = message.payload {
            match payload {
                ACMessage::GetValue { state: _ } => {
                    if let Some(responder) = message.responder {
                        responder
                            .handle(ACResponse::Value(self.value_estimate))
                            .await;
                    }
                }
                ACMessage::UpdateCritic { state: _, td_error } => {
                    self.update(td_error);
                    if let Some(responder) = message.responder {
                        responder.handle(ACResponse::Success).await;
                    }
                }
                ACMessage::GetStats => {
                    let stats = format!(
                        "Critic Stats:\n  Value Estimate: {:.3}\n  Learning Rate: {}",
                        self.value_estimate, self.learning_rate
                    );
                    if let Some(responder) = message.responder {
                        responder.handle(ACResponse::Stats(stats)).await;
                    }
                }
                _ => {}
            }
        }
    }
}

/// Coordinator - manages training loop
struct Coordinator {
    env_id: u32,
    actor_id: u32,
    critic_id: u32,
    #[allow(dead_code)]
    gamma: f64, // Discount factor
}

impl Coordinator {
    fn new(env_id: u32, actor_id: u32, critic_id: u32) -> Self {
        Self {
            env_id,
            actor_id,
            critic_id,
            gamma: 0.99,
        }
    }

    async fn run_episode(
        &self,
        system: &mut sids::actors::actor_system::ActorSystem<ACMessage, ACResponse>,
        episode: u32,
    ) -> Result<f64, String> {
        let state = 0u32; // In bandit, state doesn't change

        // 1. Actor selects action
        let (handler, rx) = get_response_handler::<ACResponse>();
        actors::send_message_by_id(
            system,
            self.actor_id,
            Message {
                payload: Some(ACMessage::GetAction { state }),
                stop: false,
                responder: Some(handler),
                blocking: None,
            },
        )
        .await
        .map_err(|e| format!("Failed to get action: {:?}", e))?;

        let action = match rx.await.map_err(|e| format!("No response: {}", e))? {
            ACResponse::Action(a) => a,
            _ => return Err("Invalid response type".to_string()),
        };

        // 2. Environment provides reward
        let (handler, rx) = get_response_handler::<ACResponse>();
        actors::send_message_by_id(
            system,
            self.env_id,
            Message {
                payload: Some(ACMessage::PullArm { arm: action }),
                stop: false,
                responder: Some(handler),
                blocking: None,
            },
        )
        .await
        .map_err(|e| format!("Failed to pull arm: {:?}", e))?;

        let reward = match rx.await.map_err(|e| format!("No response: {}", e))? {
            ACResponse::Reward(r) => r,
            _ => return Err("Invalid response type".to_string()),
        };

        // 3. Critic evaluates state
        let (handler, rx) = get_response_handler::<ACResponse>();
        actors::send_message_by_id(
            system,
            self.critic_id,
            Message {
                payload: Some(ACMessage::GetValue { state }),
                stop: false,
                responder: Some(handler),
                blocking: None,
            },
        )
        .await
        .map_err(|e| format!("Failed to get value: {:?}", e))?;

        let value = match rx.await.map_err(|e| format!("No response: {}", e))? {
            ACResponse::Value(v) => v,
            _ => return Err("Invalid response type".to_string()),
        };

        // 4. Calculate TD error
        // For episodic bandit: TD_error = reward - value (no next state)
        let td_error = reward - value;

        // 5. Update critic
        actors::send_message_by_id(
            system,
            self.critic_id,
            Message {
                payload: Some(ACMessage::UpdateCritic { state, td_error }),
                stop: false,
                responder: None,
                blocking: None,
            },
        )
        .await
        .map_err(|e| format!("Failed to update critic: {:?}", e))?;

        // 6. Update actor
        actors::send_message_by_id(
            system,
            self.actor_id,
            Message {
                payload: Some(ACMessage::UpdateActor {
                    state,
                    action,
                    td_error,
                }),
                stop: false,
                responder: None,
                blocking: None,
            },
        )
        .await
        .map_err(|e| format!("Failed to update actor: {:?}", e))?;

        if episode % 100 == 0 {
            info!(
                "Episode {}: Action={}, Reward={:.1}, Value={:.3}, TD_Error={:.3}",
                episode, action, reward, value, td_error
            );
        }

        Ok(reward)
    }
}

impl Actor<ACMessage, ACResponse> for Coordinator {
    async fn receive(&mut self, message: Message<ACMessage, ACResponse>) {
        if let Some(ACMessage::RunEpisode { episode }) = message.payload {
            // Note: In real implementation, we'd need access to actor system
            // This is a simplified example - in practice, coordinator would
            // be external to the actor system or have a reference to it
            info!("Coordinator received episode request: {}", episode);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("=== Actor-Critic Multi-Armed Bandit ===");
    info!("3 arms with reward probabilities: [0.1, 0.5, 0.3]");
    info!("Goal: Learn to prefer arm 1 (50% reward rate)\n");

    // Create actor system
    let mut system = actors::start_actor_system::<ACMessage, ACResponse>();

    // Spawn agents
    let env = BanditEnvironment::new();
    let actor_agent = ActorAgent::new(3);
    let critic = CriticAgent::new();

    spawn_actor(&mut system, env, Some("Environment".to_string())).await;
    spawn_actor(&mut system, actor_agent, Some("Actor".to_string())).await;
    spawn_actor(&mut system, critic, Some("Critic".to_string())).await;

    // Get actor IDs
    let env_id = actors::find_actor_by_name(&system, "Environment")?;
    let actor_id = actors::find_actor_by_name(&system, "Actor")?;
    let critic_id = actors::find_actor_by_name(&system, "Critic")?;

    info!(
        "Spawned actors - Env: {}, Actor: {}, Critic: {}\n",
        env_id, actor_id, critic_id
    );

    // Create coordinator
    let coordinator = Coordinator::new(env_id, actor_id, critic_id);

    // Training loop
    let num_episodes = 500;
    let mut total_reward = 0.0;

    for episode in 1..=num_episodes {
        match coordinator.run_episode(&mut system, episode).await {
            Ok(reward) => total_reward += reward,
            Err(e) => warn!("Episode {} failed: {}", episode, e),
        }
    }

    let avg_reward = total_reward / num_episodes as f64;
    info!("\n=== Training Complete ===");
    info!("Episodes: {}", num_episodes);
    info!("Average Reward: {:.3}", avg_reward);

    // Get final statistics
    info!("\n=== Final Statistics ===");

    // Actor stats
    let (handler, rx) = get_response_handler::<ACResponse>();
    actors::send_message_by_id(
        &mut system,
        actor_id,
        Message {
            payload: Some(ACMessage::GetStats),
            stop: false,
            responder: Some(handler),
            blocking: None,
        },
    )
    .await?;
    if let ACResponse::Stats(stats) = rx.await? {
        info!("\n{}", stats);
    }

    // Critic stats
    let (handler, rx) = get_response_handler::<ACResponse>();
    actors::send_message_by_id(
        &mut system,
        critic_id,
        Message {
            payload: Some(ACMessage::GetStats),
            stop: false,
            responder: Some(handler),
            blocking: None,
        },
    )
    .await?;
    if let ACResponse::Stats(stats) = rx.await? {
        info!("\n{}", stats);
    }

    // Environment stats
    let (handler, rx) = get_response_handler::<ACResponse>();
    actors::send_message_by_id(
        &mut system,
        env_id,
        Message {
            payload: Some(ACMessage::GetStats),
            stop: false,
            responder: Some(handler),
            blocking: None,
        },
    )
    .await?;
    if let ACResponse::Stats(stats) = rx.await? {
        info!("\n{}", stats);
    }

    info!("\n=== Analysis ===");
    info!("The actor should learn to prefer Arm 1 (50% reward rate)");
    info!("Action values show learned preferences for each arm");
    info!("Critic's value estimate approximates expected long-term reward");

    Ok(())
}
