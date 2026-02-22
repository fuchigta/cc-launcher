use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    #[allow(dead_code)]
    Config(String),

    #[error("{0}")]
    Execution(String),

    #[error("{0}")]
    Plugin(String),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_error_display_not_found() {
        let err = AppError::NotFound("item missing".to_string());
        assert_eq!(err.to_string(), "item missing");
    }

    #[test]
    fn app_error_display_config() {
        let err = AppError::Config("invalid config".to_string());
        assert_eq!(err.to_string(), "invalid config");
    }

    #[test]
    fn app_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_err: AppError = io_err.into();
        assert!(matches!(app_err, AppError::Io(_)));
    }

    #[test]
    fn app_error_serialize() {
        let err = AppError::Execution("timeout".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert_eq!(json, "\"timeout\"");
    }
}
