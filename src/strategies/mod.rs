pub mod evaluation;
/// Strategy for choosing an action.
pub trait Strategy <StateType, ActionType> {
    /// Returns an action based on the referenced boardstate.
    fn decide(&mut self, state: &StateType) -> ActionType;
}

impl<F,S,A> Strategy<S, A> for F where F: FnMut(&S) -> A{
    fn decide(&mut self, state: &S) -> A {
        self(state)
    }
}