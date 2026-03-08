pub trait Backend: Send {
    fn apply_mutations(
        &mut self,
        mutations: &[native_schema::Mutation],
    ) -> Result<(), BackendError>;
    fn apply_layout(
        &mut self,
        frames: &[native_schema::LayoutFrame],
    ) -> Result<(), BackendError>;
    fn flush(&mut self) -> Result<(), BackendError>;

    fn drain_events(&mut self) -> Vec<native_schema::UiEvent> {
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendError {
    BatchRejected(String),
}

pub fn debug_mutations(mutations: &[native_schema::Mutation]) -> String {
    format!("{mutations:#?}")
}

pub fn debug_layout(frames: &[native_schema::LayoutFrame]) -> String {
    format!("{frames:#?}")
}
