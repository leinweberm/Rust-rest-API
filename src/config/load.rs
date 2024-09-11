use std::{any::Any, env, io};
use dotenv::dotenv;
use lazy_static::lazy_static;
use tokio::sync::OnceCell;
use std::sync::Arc;

/// ConfigFields
/// ```
/// DatabaseUrl: String // postgres compatible connection string
/// DatabaseCertPath: String // absolute path to root CA certificate
/// StaticFilesDir: String // absolute path to serve static files from
/// CacheClientPages: u8 // (minutes) controls html pages browser cache
/// CacheClientStatic: u8 // (hours) controls static assets browser cache
/// CacheMemoryPages: u8 // (minutes) controls in-memory Rust cache for html pages
/// CacheMemoryGeneral: u8 // (seconds) controls in-memory Rust general purpose cache
/// ```
#[derive(Clone)]
pub enum ConfigField {
    TestVariable,
    DatabaseUrl,
    DatabaseCertPath,
    StaticFilesDir,
    CacheClientPages,
    CacheClientStatic,
    CacheMemoryPages,
    CacheMemoryGeneral
}

impl ConfigField {
    pub fn to_str(&self) -> &str {
        match self {
            ConfigField::TestVariable => "test_variable",
            ConfigField::DatabaseUrl => "database_url",
            ConfigField::DatabaseCertPath => "database_cert_path",
            ConfigField::StaticFilesDir => "static_files_dir",
            ConfigField::CacheClientPages => "cache_client_pages",
            ConfigField::CacheClientStatic => "cache_client_static",
            ConfigField::CacheMemoryPages => "cache_memory_pages",
            ConfigField::CacheMemoryGeneral => "cache_memory_general"
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub test_variable: String,
    pub database_url: String,
    pub database_cert_path: String,
    pub static_files_dir: String,
    pub cache_client_pages: u8,
    pub cache_client_static: u8,
    pub cache_memory_pages: u8,
    pub cache_memory_general: u8
}

impl Config {
    pub fn get_field<T: 'static>(&self, field: ConfigField) -> Result<T, std::io::Error> where T: Any + Clone {
        let value: Box<dyn Any + Send> = match field {
            ConfigField::TestVariable => Box::new(self.test_variable.clone()),
            ConfigField::DatabaseUrl => Box::new(self.database_url.clone()),
            ConfigField::DatabaseCertPath => Box::new(self.database_cert_path.clone()),
            ConfigField::StaticFilesDir => Box::new(self.static_files_dir.clone()),
            ConfigField::CacheClientPages => Box::new(self.cache_client_pages),
            ConfigField::CacheClientStatic => Box::new(self.cache_client_static),
            ConfigField::CacheMemoryPages => Box::new(self.cache_memory_pages),
            ConfigField::CacheMemoryGeneral => Box::new(self.cache_memory_general)
        };

        if let Some(result) = value.downcast_ref::<T>() {
            Ok(result.clone())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, format!("Failed to get config property {}", &field.to_str())))
        }
    }
}

lazy_static! {
    pub static ref CONFIG: OnceCell<Arc<Config>> = OnceCell::const_new();
}

pub async fn init () -> Result<(), std::io::Error> {
    dotenv().ok();
    debug!(target: "cfg", ".env file loaded");
    let missing_required_error = "is required config property";
    let invalid_type_error = "should be type of";

    let mut field = ConfigField::DatabaseUrl.to_str();
    let database_url = env::var(&field)
        .expect(&format!("{} {}", &field, &missing_required_error));

    field = ConfigField::DatabaseCertPath.to_str();
    let database_cert_path = env::var(&field)
        .expect(&format!("{} {}", &field, &missing_required_error));

    field = ConfigField::StaticFilesDir.to_str();
    let static_files_dir = env::var(&field)
        .expect(&format!("{} {}", &field, &missing_required_error));

    field = ConfigField::CacheClientPages.to_str();
    let temp_client_pages = env::var(&field)
        .expect(&format!("{} {}", &field, &missing_required_error));
    let cache_client_pages: u8 = temp_client_pages
        .parse()
        .expect(&format!("{} {} {}", &field, &invalid_type_error, "u8"));

    field = ConfigField::CacheClientStatic.to_str();
    let temp_client_static = env::var(&field)
        .expect(&format!("{} {}", &field, &missing_required_error));
    let cache_client_static: u8 = temp_client_static
        .parse()
        .expect(&format!("{} {} {}", &field, &invalid_type_error, "u8"));

    field = ConfigField::CacheMemoryPages.to_str();
    let temp_memory_pages = env::var(&field)
        .expect(&format!("{} {}", &field, &missing_required_error));
    let cache_memory_pages: u8 = temp_memory_pages
        .parse()
        .expect(&format!("{} {} {}", &field, &invalid_type_error, "u8"));

    field = ConfigField::CacheMemoryGeneral.to_str();
    let temp_memory_general = env::var(&field)
        .expect(&format!("{} {}", &field, &missing_required_error));
    let cache_memory_general = temp_memory_general
        .parse()
        .expect(&format!("{} {} {}", &field, &invalid_type_error, "u8"));

    let config = Arc::new(Config {
        test_variable: "test".to_string(),
        database_url,
        database_cert_path,
        static_files_dir,
        cache_client_pages,
        cache_client_static,
        cache_memory_pages,
        cache_memory_general
    });
    debug!(target: "cfg", "config instance created");

    CONFIG.set(config)
        .expect("Failed to set config as static reference");

    Ok(())
}

pub async fn get<T>(field: ConfigField) -> Result<T, io::Error> where T: 'static + Any + Clone {
    let config_ref = CONFIG
        .get()
        .ok_or_else(||io::Error::new(io::ErrorKind::InvalidData, "Config does not exist"))?;

    let config = config_ref.as_ref();

    let field_clone = field.clone();
    let result = config.get_field::<T>(field);

    match &result {
        Ok(_) => {
            debug!(target: "cfg", "Successfully retrieved config value for field {}", field_clone.to_str());
        }
        Err(e) => {
            error!(target: "cfg", "Failed to retrieve config value: {:?}", e);
        }
    };

    result
}

pub async fn test () -> Result<(), io::Error> {
    let value = get::<String>(ConfigField::TestVariable).await;

    match value {
        Ok(result) => {
            let value_string: String = String::from(result);
            assert_eq!(&value_string, "test");
            Ok(())
        },
        Err(e) => {
            eprintln!("Error retrieving config: {}", e);
            Err(io::Error::new(io::ErrorKind::InvalidData, format!("{}", e)))
        }
    }
}