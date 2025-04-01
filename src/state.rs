use rocket::tokio::sync::Mutex;
use std::sync::Arc;

type LockedState = Arc<Mutex<State>>;

pub struct _State {
    state: LockedState,
}

impl Default for _State {
    fn default() -> Self {
        _State {
            state: Arc::new(Mutex::new(State::default())),
        }
    }
}

impl _State {
    pub async fn get(&self) -> rocket::tokio::sync::MutexGuard<'_, State> {
        self.state.lock().await
    }
}

#[derive(Debug, Clone, Default)]
pub struct State {}

impl State {}

pub fn initial_state() -> _State {
    _State::default()
}
