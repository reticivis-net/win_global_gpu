// behavior like anyhow but without dependencies
pub type Error = Box<dyn std::error::Error>;
pub type Result<T, E = Error> = core::result::Result<T, E>;
// to convert a str(ing) into an Error, use .into()
