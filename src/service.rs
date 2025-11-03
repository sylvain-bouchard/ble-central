use std::sync::Arc;

pub trait Service: Send + Sync {}

pub struct Logger;
impl Service for Logger {}

impl Logger {
    pub fn log(&self, msg: &str) {
        println!("[LOG] {}", msg);
    }
}

pub struct Database;
impl Service for Database {}

impl Database {
    pub fn query(&self, sql: &str) {
        println!("Executing SQL: {}", sql);
    }
}

pub struct ServiceProvider {
    pub logger: Arc<Logger>,
    pub database: Arc<Database>,
}

impl ServiceProvider {
    pub fn new() -> Self {
        Self {
            logger: Arc::new(Logger),
            database: Arc::new(Database),
        }
    }
}
