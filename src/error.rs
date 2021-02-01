

#[derive(Debug)]
pub enum Error {
    PbfError(osmpbfreader::error::Error),
    IoError(std::io::Error),
    MissingInfo(String),
    NotANode(osmpbfreader::objects::OsmObj),
    NotAWayId(osmpbfreader::objects::OsmId),
    ConversionError(String),
    #[cfg(feature="gdal")]
    GdalError(gdal::errors::GdalError),
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
            Error::NotAWayId(o) =>
                write!(f, "OsmId is not a way id: {:?}", o),
            Error::ConversionError(s) =>
                write!(f, "ConversionError: {}", s),
            #[cfg(feature="gdal")]
            Error::GdalError(e) =>
                write!(f, "GdalError: {}", e),
            Error::NodeCountError => 
                write!(f, "Error counting nodes"),
            Error::MakeGraphError =>
                write!(f, "Error making graph")
        }
    }
}
