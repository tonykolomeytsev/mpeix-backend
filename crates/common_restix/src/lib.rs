use anyhow::anyhow;
use common_errors::errors::CommonError;
use common_rust::env;

pub trait ResultExt<T>
where
    Self: Sized,
{
    fn with_common_error(self) -> anyhow::Result<T>;
}

impl<T> ResultExt<T> for reqwest::Result<T> {
    fn with_common_error(self) -> anyhow::Result<T> {
        self.map_err(|err| {
            if err.is_decode() {
                anyhow!(CommonError::internal(err))
            } else {
                anyhow!(CommonError::gateway(err))
            }
        })
    }
}

pub fn create_reqwest_client() -> reqwest::Client {
    let connect_timeout = env::get_parsed_or("GATEWAY_CONNECT_TIMEOUT", 1500);
    reqwest::ClientBuilder::new()
        .gzip(true)
        .deflate(true)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(15))
        .connect_timeout(std::time::Duration::from_millis(connect_timeout))
        .pool_max_idle_per_host(3)
        .build()
        .expect("Error while building reqwest::Client")
}
