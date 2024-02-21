use leptos::*;

use super::state::AppState;

pub fn extract_state() -> Result<AppState, ServerFnError> {
    use_context::<AppState>().ok_or_else(|| ServerFnError::ServerError("App state missing.".into()))
}
