use parking_lot::RwLock;
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::WichtelEvent;

pub struct AppState {
    pub events: RwLock<HashMap<Uuid, WichtelEvent>>,
    pub invite_codes: RwLock<HashMap<String, Uuid>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(HashMap::new()),
            invite_codes: RwLock::new(HashMap::new()),
        }
    }

    pub fn create_event(&self, name: String) -> WichtelEvent {
        let event = WichtelEvent::new(name);
        let mut events = self.events.write();
        let mut codes = self.invite_codes.write();
        
        codes.insert(event.invite_code.clone(), event.id);
        events.insert(event.id, event.clone());
        event
    }

    pub fn get_event(&self, id: &Uuid) -> Option<WichtelEvent> {
        self.events.read().get(id).cloned()
    }

    pub fn get_event_by_invite_code(&self, code: &str) -> Option<WichtelEvent> {
        let codes = self.invite_codes.read();
        let event_id = codes.get(code)?;
        self.get_event(event_id)
    }

    pub fn add_participant(&self, event_id: &Uuid, name: String) -> Option<Uuid> {
        let mut events = self.events.write();
        let event = events.get_mut(event_id)?;
        Some(event.add_participant(name))
    }

    pub fn close_event(&self, event_id: &Uuid, organizer_token: &Uuid) -> Result<(), &'static str> {
        let mut events = self.events.write();
        let event = events.get_mut(event_id).ok_or("Event not found")?;
        
        if &event.organizer_token != organizer_token {
            return Err("Invalid organizer token");
        }
        
        event.close_and_assign()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
