use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub trait GameState<C, S>
where
    S: Copy + Eq + Hash + Debug,
{
    fn on_enter(&mut self, _ctx: &mut C) {}
    fn on_exit(&mut self, _ctx: &mut C) {}
    fn update(&mut self, ctx: &mut C) -> Option<S>;
    fn draw(&self, ctx: &C);
}

pub struct StateMachine<C, S>
where
    S: Copy + Eq + Hash + Debug,
{
    states: HashMap<S, Box<dyn GameState<C, S>>>,
    current: Option<S>,
}

impl<C, S> StateMachine<C, S>
where
    S: Copy + Eq + Hash + Debug,
{
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            current: None,
        }
    }

    pub fn add_state<T: GameState<C, S> + 'static>(&mut self, id: S, state: T) {
        self.states.insert(id, Box::new(state));
    }

    pub fn set_initial(&mut self, id: S, ctx: &mut C) -> Result<(), String> {
        if !self.states.contains_key(&id) {
            return Err(format!("State '{id:?}' is not registered"));
        }

        self.current = Some(id);
        if let Some(state) = self.states.get_mut(&id) {
            state.on_enter(ctx);
        }

        Ok(())
    }

    pub fn current(&self) -> Option<S> {
        self.current
    }

    pub fn update(&mut self, ctx: &mut C) {
        let Some(current) = self.current else {
            return;
        };

        let next = self
            .states
            .get_mut(&current)
            .and_then(|state| state.update(ctx));
        if let Some(next_state) = next {
            self.transition(next_state, ctx);
        }
    }

    pub fn draw(&self, ctx: &C) {
        if let Some(current) = self.current {
            if let Some(state) = self.states.get(&current) {
                state.draw(ctx);
            }
        }
    }

    fn transition(&mut self, next: S, ctx: &mut C) {
        if self.current == Some(next) || !self.states.contains_key(&next) {
            return;
        }

        if let Some(current) = self.current {
            if let Some(state) = self.states.get_mut(&current) {
                state.on_exit(ctx);
            }
        }

        if let Some(state) = self.states.get_mut(&next) {
            state.on_enter(ctx);
        }

        self.current = Some(next);
    }
}

impl<C, S> Default for StateMachine<C, S>
where
    S: Copy + Eq + Hash + Debug,
{
    fn default() -> Self {
        Self::new()
    }
}
