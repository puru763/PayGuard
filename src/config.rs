use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub environment: String,
}

impl Config {
    /// Loads configuration from environment variables, failing fast if any are missing.
    pub fn from_env() -> Result<Self, String> {
        // Load .env file if present (useful for local development)
        let _ = dotenvy::dotenv();

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "Environment variable 'DATABASE_URL' is required but not set.")?;

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .map_err(|_| "Environment variable 'SERVER_PORT' must be a valid u16 port number.")?;

        let environment = env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string());

        Ok(Self {
            database_url,
            server_port,
            environment,
        })
    }
}