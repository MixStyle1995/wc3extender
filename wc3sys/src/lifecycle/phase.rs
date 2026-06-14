use std::any::Any;

pub trait Event: Any + Send + Sync + 'static {}

impl<T> Event for T where T: Any + Send + Sync + 'static {}

#[derive(Debug, Clone, Copy)]
pub struct ConfigStarted {
    pub reload: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ConfigRebuild {
    pub reload: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ConfigFinished {
    pub reload: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct MainStarted;

#[derive(Debug, Clone)]
pub struct JassFunctionCalled {
    pub name: Box<str>,
}

#[derive(Debug, Clone, Copy)]
pub struct NativeRegistration;

#[derive(Debug, Clone, Copy)]
pub struct War3MpqArchivesInitializing;

#[derive(Debug, Clone, Copy)]
pub struct War3MpqArchivesInitialized;

#[derive(Debug, Clone, Copy)]
pub struct War3MpqArchivesFailed;
