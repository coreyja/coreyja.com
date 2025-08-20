use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentActivityType {
    Thought,
    Action,
    Response,
    Error,
    Elicitation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AgentActivityContent {
    Thought {
        body: String,
    },
    Action {
        action: String,
        parameter: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<serde_json::Value>,
    },
    Response {
        body: String,
    },
    Error {
        body: String,
    },
    Elicitation {
        body: String,
    },
}

impl AgentActivityContent {
    pub fn thought(body: impl Into<String>) -> Self {
        Self::Thought { body: body.into() }
    }

    pub fn action(action: impl Into<String>, parameter: serde_json::Value) -> Self {
        Self::Action {
            action: action.into(),
            parameter,
            result: None,
        }
    }

    pub fn action_with_result(
        action: impl Into<String>,
        parameter: serde_json::Value,
        result: serde_json::Value,
    ) -> Self {
        Self::Action {
            action: action.into(),
            parameter,
            result: Some(result),
        }
    }

    pub fn response(body: impl Into<String>) -> Self {
        Self::Response { body: body.into() }
    }

    pub fn error(body: impl Into<String>) -> Self {
        Self::Error { body: body.into() }
    }

    pub fn elicitation(body: impl Into<String>) -> Self {
        Self::Elicitation { body: body.into() }
    }
}
