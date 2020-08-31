

#[derive(Debug)]
pub enum Error {
    PbfError(osmpbfreader::error::Error),
    IoError(std::io::Error),
    MissingInfo(String),
    NotANode(osmpbfreader::objects::OsmObj),
    ConversionError(String),
    NodeCountError,
    MakeGraphError
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::IoError(e) => 
                write!(f, "IoError: {}", e),
            Error::PbfError(e) =>
                write!(f, "PbfError: {}", e),
            Error::MissingInfo(s) => 
                write!(f, "MissingInfo: {}", s),
            Error::NotANode(o) =>
                write!(f, "Object is not a node: {:?}", o),
            Error::ConversionError(s) =>
                write!(f, "ConversionError: {}", s),
            Error::NodeCountError => 
                write!(f, "Error counting nodes"),
            Error::MakeGraphError =>
                write!(f, "Error making graph")
        }
    }
}
