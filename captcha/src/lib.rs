use eva::fut::Fut;

use viendesu_core::{
    bail,
    types::captcha::Token,
    world::{World, WorldMut},
};

use self::error::CaptchaResult;

pub mod error;

pub trait Service: Send + Sync {
    fn verify<W: WorldMut>(
        &self,
        w: World<W>,
        token: &Token,
    ) -> impl Fut<Output = CaptchaResult<()>>;
}

impl Service for bool {
    async fn verify<W: WorldMut>(&self, w: World<W>, token: &Token) -> CaptchaResult<()> {
        _ = w;
        _ = token;

        if *self {
            Ok(())
        } else {
            bail!("invalid captcha")
        }
    }
}
