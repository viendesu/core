use viendesu_core::errors::AuxResult;

use super::*;

impl Authentication for HttpClient {
    async fn authenticate(&mut self, session: session::Token) -> AuxResult<()> {
        self.session = Some(session);

        Ok(())
    }

    fn clear(&mut self) {
        self.session = None;
    }
}
