use super::*;

use futures::stream::StreamExt as _;

use viendesu_core::{
    requests::uploads::{abort, finish, list_pending, start},
    uploads::Chunk,
};

use crate::requests::uploads as requests;

struct FinishUpload<'a> {
    client: &'a mut HttpClient,
}

impl CallStep<finish::Args> for FinishUpload<'_> {
    type Ok = finish::Ok;
    type Err = finish::Err;

    async fn call(&mut self, args: finish::Args) -> Response<Self::Ok, Self::Err> {
        use reqwest::multipart;

        let finish::Args { id, stream } = args;

        // I don't know why anyone would re-use same stream, but let it be.
        let content_length = stream.left();
        let endpoint = self.client.endpoint(&c!("/uploads/{id}"));
        let mut request = self.client.client.post(endpoint);
        if let Some(session) = self.client.session {
            request = request.bearer_auth(session);
        }
        let body = reqwest::Body::wrap_stream(stream.map(|c| match c {
            Chunk::Aborted(error) => Err(error),
            Chunk::Data(data) => Ok(data),
        }));
        request = request.multipart(multipart::Form::new().part(
            "file",
            if let Some(size) = content_length {
                multipart::Part::stream_with_length(body, size as u64)
            } else {
                multipart::Part::stream(body)
            },
        ));

        let response = self
            .client
            .client
            .execute(request.build().expect("shit happens"))
            .await
            .map_err(|e| Aux::InternalError(format!("failed to make request: {e}")))?;
        let bytes = response
            .bytes()
            .await
            .map_err(|e| Aux::InternalError(format!("failed to read response: {e}")))?;

        self.client.load_response(&bytes)
    }
}

impl Uploads for HttpClient {
    fn list_pending(
        &mut self,
    ) -> impl CallStep<list_pending::Args, Ok = list_pending::Ok, Err = list_pending::Err> {
        self.do_call(Method::GET, |list_pending::Args {}| {
            (c!("/uploads"), requests::ListPending {})
        })
    }

    fn start(&mut self) -> impl CallStep<start::Args, Ok = start::Ok, Err = start::Err> {
        self.do_call(
            Method::POST,
            |start::Args {
                 name,
                 hash,
                 class,
                 size,
             }| {
                (
                    c!("/uploads"),
                    requests::Start {
                        file_name: name,
                        hash,
                        class,
                        size,
                    },
                )
            },
        )
    }

    fn abort(&mut self) -> impl CallStep<abort::Args, Ok = abort::Ok, Err = abort::Err> {
        self.do_call(Method::DELETE, |abort::Args { upload }| {
            (c!("/uploads/{upload}"), requests::Abort {})
        })
    }

    fn finish(&mut self) -> impl CallStep<finish::Args, Ok = finish::Ok, Err = finish::Err> {
        FinishUpload { client: self }
    }
}
