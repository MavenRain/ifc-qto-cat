//! Hand-rolled error type for QTO operations.

/// Errors that can occur during quantity takeoff processing.
#[derive(Debug)]
pub enum Error {
    /// The IFC content could not be parsed.
    Core(ifc_lite_core_cat::Error),
    /// An entity referenced by a relation could not be found.
    MissingEntity(u32),
    /// A quantity set was malformed or missing expected data.
    MalformedQuantity(String),
    /// No matching elements were found for a metric.
    NoMatchingElements(String),
}

/// Convenience alias.
pub type Result<T> = std::result::Result<T, Error>;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Core(e) => write!(f, "core parser error: {e}"),
            Self::MissingEntity(id) => write!(f, "entity #{id} not found"),
            Self::MalformedQuantity(msg) => write!(f, "malformed quantity: {msg}"),
            Self::NoMatchingElements(metric) => {
                write!(f, "no matching elements for metric: {metric}")
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Core(e) => Some(e),
            Self::MissingEntity(_) | Self::MalformedQuantity(_) | Self::NoMatchingElements(_) => {
                None
            }
        }
    }
}

impl From<ifc_lite_core_cat::Error> for Error {
    fn from(e: ifc_lite_core_cat::Error) -> Self {
        Self::Core(e)
    }
}
