use eva::{auto_impl, fut::Fut};

#[cfg(feature = "tokio")]
mod tokio_rt {
    use super::*;

    #[derive(Debug, Clone, Copy)]
    pub struct TokioRt;

    impl RtRef for TokioRt {}
    impl RtMut for TokioRt {
        fn spawn<F: Fut<Output: Send> + 'static>(&mut self, task: F) {
            tokio::spawn(task);
        }

        fn spawn_blocking<F: BlockingFn>(&mut self, task: F) -> impl Fut<Output = F::Ret> {
            async move {
                let ret = tokio::task::spawn_blocking(task).await;
                ret.expect("failed to join blocking task")
            }
        }
    }
}

#[cfg(feature = "tokio")]
pub use tokio_rt::*;

#[auto_impl(&, &mut)]
pub trait RtRef: Send + Sync {}

pub trait BlockingFn: FnOnce() -> Self::Ret + Send + 'static {
    type Ret: Send + 'static;
}

impl<F, R> BlockingFn for F
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    type Ret = R;
}

#[auto_impl(&mut)]
pub trait RtMut: RtRef {
    fn spawn_blocking<F: BlockingFn>(&mut self, task: F) -> impl Fut<Output = F::Ret>;
    fn spawn<F: Fut<Output: Send> + 'static>(&mut self, task: F);
}
