use common_errors::errors::CommonError;

pub trait ResultExt<T>
where
    Self: Sized,
{
    fn with_common_error(self) -> Result<T, CommonError>;
}

impl<T> ResultExt<T> for restix::Result<T> {
    fn with_common_error(self) -> Result<T, CommonError> {
        self.map_err(|err| {
            let reqwest_error: &reqwest::Error = err.as_ref();
            if reqwest_error.is_decode() {
                CommonError::internal(reqwest_error)
            } else {
                CommonError::gateway(reqwest_error)
            }
        })
    }
}
