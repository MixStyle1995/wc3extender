use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::logging;

use super::phase::{
    ConfigFinished, ConfigRebuild, ConfigStarted, Event, JassFunctionCalled, MainStarted,
    NativeRegistration,
};

type ErasedHandler = Arc<dyn Fn(&dyn Any) + Send + Sync>;

#[derive(Clone)]
struct Subscription {
    sequence: usize,
    handler: ErasedHandler,
}

static SUBSCRIPTIONS: OnceLock<Mutex<HashMap<TypeId, Vec<Subscription>>>> = OnceLock::new();
static NEXT_SEQUENCE: AtomicUsize = AtomicUsize::new(0);
static SEEN_CONFIG: AtomicBool = AtomicBool::new(false);
static MAIN_DISPATCHED: AtomicBool = AtomicBool::new(false);

fn subscriptions() -> &'static Mutex<HashMap<TypeId, Vec<Subscription>>> {
    SUBSCRIPTIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn init() {
    let _ = subscriptions();
}

pub fn on<E, F>(f: F)
where
    E: Event,
    F: Fn(&E) + Send + Sync + 'static,
{
    let sequence = NEXT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let handler: ErasedHandler = Arc::new(move |event: &dyn Any| {
        let event = event
            .downcast_ref::<E>()
            .expect("lifecycle event type mismatch");
        f(event);
    });

    subscriptions()
        .lock()
        .unwrap()
        .entry(TypeId::of::<E>())
        .or_default()
        .push(Subscription { sequence, handler });
}

pub struct Component<T> {
    component: Arc<T>,
}

pub fn component<T>(component: Arc<T>) -> Component<T>
where
    T: Send + Sync + 'static,
{
    Component { component }
}

impl<T> Component<T>
where
    T: Send + Sync + 'static,
{
    pub fn on<E>(self, method: fn(&T, &E)) -> Self
    where
        E: Event,
    {
        let component = self.component.clone();
        on(move |event: &E| {
            method(component.as_ref(), event);
        });
        self
    }
}

pub fn emit<E>(event: E)
where
    E: Event,
{
    let mut snapshot = subscriptions()
        .lock()
        .unwrap()
        .get(&TypeId::of::<E>())
        .cloned()
        .unwrap_or_default();

    snapshot.sort_by_key(|sub| sub.sequence);

    for sub in snapshot {
        (sub.handler)(&event);
    }
}

pub fn observe_jass_function(name: &str) {
    match name {
        "config" => {
            let reload = SEEN_CONFIG.swap(true, Ordering::Relaxed);
            MAIN_DISPATCHED.store(false, Ordering::Relaxed);

            emit(ConfigStarted { reload });
            emit(ConfigRebuild { reload });
            emit(ConfigFinished { reload });
        }
        "main" => {
            if MAIN_DISPATCHED.swap(true, Ordering::Relaxed) {
                logging::info("lifecycle: main skipped; already dispatched for this config");
                return;
            }

            emit(MainStarted);
        }
        other => emit(JassFunctionCalled {
            name: other.into(),
        }),
    }
}

pub fn observe_native_registration() {
    emit(NativeRegistration);
}
