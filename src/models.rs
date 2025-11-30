use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: Uuid,
    pub name: String,
    pub joined_at: DateTime<Utc>,
    pub assigned_to: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventStatus {
    Open,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WichtelEvent {
    pub id: Uuid,
    pub name: String,
    pub organizer_token: Uuid,
    pub invite_code: String,
    pub status: EventStatus,
    pub participants: HashMap<Uuid, Participant>,
    pub created_at: DateTime<Utc>,
}

impl WichtelEvent {
    pub fn new(name: String) -> Self {
        let invite_code = generate_invite_code();
        Self {
            id: Uuid::new_v4(),
            name,
            organizer_token: Uuid::new_v4(),
            invite_code,
            status: EventStatus::Open,
            participants: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    pub fn add_participant(&mut self, name: String) -> Uuid {
        let participant = Participant {
            id: Uuid::new_v4(),
            name,
            joined_at: Utc::now(),
            assigned_to: None,
        };
        let id = participant.id;
        self.participants.insert(id, participant);
        id
    }

    pub fn close_and_assign(&mut self) -> Result<(), &'static str> {
        if self.status == EventStatus::Closed {
            return Err("Event is already closed");
        }
        if self.participants.len() < 2 {
            return Err("Need at least 2 participants");
        }

        let participant_ids: Vec<Uuid> = self.participants.keys().cloned().collect();
        let assignments = generate_assignments(&participant_ids)?;
        
        for (giver, receiver) in assignments {
            if let Some(participant) = self.participants.get_mut(&giver) {
                participant.assigned_to = Some(receiver);
            }
        }

        self.status = EventStatus::Closed;
        Ok(())
    }

    pub fn get_assignment(&self, participant_id: Uuid) -> Option<&Participant> {
        let participant = self.participants.get(&participant_id)?;
        let assigned_to_id = participant.assigned_to?;
        self.participants.get(&assigned_to_id)
    }
}

fn generate_invite_code() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::thread_rng();
    (0..6)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn generate_assignments(participant_ids: &[Uuid]) -> Result<Vec<(Uuid, Uuid)>, &'static str> {
    use rand::seq::SliceRandom;
    
    if participant_ids.len() < 2 {
        return Err("Need at least 2 participants");
    }

    let mut rng = rand::thread_rng();
    let mut shuffled = participant_ids.to_vec();
    
    // Use derangement algorithm - ensure no one gets themselves
    loop {
        shuffled.shuffle(&mut rng);
        let valid = participant_ids.iter()
            .zip(shuffled.iter())
            .all(|(a, b)| a != b);
        if valid {
            break;
        }
    }

    Ok(participant_ids.iter().cloned().zip(shuffled.into_iter()).collect())
}
