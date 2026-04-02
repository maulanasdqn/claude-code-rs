use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::agent::{AgentDefinition, AgentId, AgentState, AgentStatus};
use crate::domain::team::{Team, TeamId};

pub struct CoordinatorMode {
    pub agents: HashMap<AgentId, AgentState>,
    pub teams: HashMap<TeamId, Team>,
}

impl CoordinatorMode {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            teams: HashMap::new(),
        }
    }

    pub fn spawn_agent(&mut self, def: AgentDefinition) -> AgentId {
        let id = def.id.clone();
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let state = AgentState {
            definition: def,
            status: AgentStatus::Running,
            created_at,
        };
        self.agents.insert(id.clone(), state);
        id
    }

    pub fn stop_agent(&mut self, id: &AgentId) {
        if let Some(state) = self.agents.get_mut(id) {
            state.status = AgentStatus::Completed;
        }
    }

    pub fn list_agents(&self) -> Vec<AgentState> {
        self.agents.values().cloned().collect()
    }

    pub fn create_team(&mut self, name: impl Into<String>, members: Vec<AgentId>) -> TeamId {
        let name = name.into();
        let id = TeamId::new(format!("team-{}", uuid_simple()));
        let team = Team {
            id: id.clone(),
            name: name.clone(),
            members,
            description: name,
        };
        self.teams.insert(id.clone(), team);
        id
    }

    pub fn delete_team(&mut self, id: &TeamId) {
        self.teams.remove(id);
    }
}

impl Default for CoordinatorMode {
    fn default() -> Self {
        Self::new()
    }
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    format!("{:08x}", nanos)
}
