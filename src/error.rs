use thiserror::Error;

#[derive(Error, Debug)]
pub enum InterfaceListError {
    #[error("An io error occured:")]
    Io {
        #[from] io: std::io::Error,
    }
}

#[derive(Error, Debug)]
pub enum EssidFetchError {
    #[error("An io error occured:")]
    Io {
        #[from] io: std::io::Error,
    },
    #[error("A nul string came in from the Linux kernel:")]
    NulError {
        #[from] nul: std::ffi::NulError,
    },

}
