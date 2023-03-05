use anyhow::anyhow;
use common_errors::errors::CommonError;
use common_rust::env;
use reqwest::redirect::Policy;

pub trait ResultExt<T>
where
    Self: Sized,
{
    fn with_common_error(self) -> anyhow::Result<T>;
}

impl<T> ResultExt<T> for restix::Result<T> {
    fn with_common_error(self) -> anyhow::Result<T> {
        self.map_err(|err| {
            let reqwest_error: &reqwest::Error = err.as_ref();
            if reqwest_error.is_decode() {
                anyhow!(CommonError::internal(reqwest_error))
            } else {
                anyhow!(CommonError::gateway(reqwest_error))
            }
        })
    }
}

pub fn create_restix_client() -> restix::Restix {
    let connect_timeout = env::get_parsed_or("GATEWAY_CONNECT_TIMEOUT", 1500);

    restix::RestixBuilder::new()
        .client(
            reqwest::ClientBuilder::new()
                .gzip(true)
                .deflate(true)
                .redirect(Policy::none())
                .timeout(std::time::Duration::from_secs(15))
                .connect_timeout(std::time::Duration::from_millis(connect_timeout))
                .pool_max_idle_per_host(3)
                .build()
                .expect("Something went wrong when building HttClient"),
        )
        .build()
}
