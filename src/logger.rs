use {
    actix_web::{
        dev::{Payload, Service, ServiceRequest, ServiceResponse, Transform},
        Error, FromRequest, HttpMessage, HttpRequest,
    },
    futures::{
        future::{ok, ready, Ready},
        task::{Context, Poll},
    },
    std::{future::Future, pin::Pin},
    tracing::Span,
    tracing_futures::Instrument,
    uuid::Uuid,
};

pub struct TracingLogger;

impl<S, B> Transform<S, ServiceRequest> for TracingLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = TracingLoggerMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(TracingLoggerMiddleware { service })
    }
}

#[doc(hidden)]
pub struct TracingLoggerMiddleware<S> {
    service: S,
}

#[derive(Clone, Copy)]
pub struct RequestId(Uuid);

impl std::ops::Deref for RequestId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::convert::Into<Uuid> for RequestId {
    fn into(self) -> Uuid {
        self.0
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<S, B> Service<ServiceRequest> for TracingLoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let user_agent = req
            .headers()
            .get("User-Agent")
            .map(|h| h.to_str().unwrap_or(""))
            .unwrap_or("");

        let request_id = RequestId(Uuid::new_v4());

        let span = tracing::info_span!(
            "Request",
            request_path = %req.path(),
            user_agent = %user_agent,
            client_ip_address = %req.connection_info().realip_remote_addr().unwrap_or(""),
            request_id = %request_id.0,
            status_code = tracing::field::Empty,
        );

        req.extensions_mut().insert(request_id);

        let fut = self.service.call(req);

        Box::pin(
            async move {
                let outcome = fut.await;

                let status_code = match &outcome {
                    Ok(response) => response.response().status(),
                    Err(error) => error.as_response_error().status_code(),
                };

                Span::current().record("status_code", &status_code.as_u16());

                outcome
            }
            .instrument(span),
        )
    }
}

impl FromRequest for RequestId {
    type Error = ();
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(req.extensions().get::<RequestId>().copied().ok_or(()))
    }
}
