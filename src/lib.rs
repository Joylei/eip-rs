pub mod client;
mod codec;
pub mod consts;
pub mod error;
pub mod frame;
pub mod objects;

pub type Result<T> = std::result::Result<T, error::Error>;

#[cfg(test)]
mod test {
    use std::future::Future;

    pub(crate) fn block_on<F>(f: F)
    where
        F: Future<Output = anyhow::Result<()>>,
    {
        let mut builder = tokio::runtime::Builder::new_current_thread();
        builder.enable_all();
        let rt = builder.build().unwrap();
        rt.block_on(f).unwrap();
    }
}
