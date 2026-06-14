use std::sync::Arc;

mod handlers;
mod manager;
mod phase;

pub use phase::*;

pub fn init() -> crate::error::Result<()> {
    manager::init();
    handlers::install()?;
    Ok(())
}

pub fn on<E, F>(f: F)
where
    E: Event,
    F: Fn(&E) + Send + Sync + 'static,
{
    manager::on(f);
}

pub fn component<T>(component: Arc<T>) -> manager::Component<T>
where
    T: Send + Sync + 'static,
{
    manager::component(component)
}

pub fn emit<E>(event: E)
where
    E: Event,
{
    manager::emit(event);
}

pub fn observe_jass_function(name: &str) {
    manager::observe_jass_function(name);
}

pub fn observe_native_registration() {
    manager::observe_native_registration();
}
