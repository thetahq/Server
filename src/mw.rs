use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use futures::future::{ok, FutureResult};
use futures::{Future, Poll};

pub struct Logging;

impl<S,B> Transform<S> for Logging where S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>, S::Future: 'static, B: 'static {
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = LoggingMiddleware<S>;
    type InitError = ();
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(LoggingMiddleware{
            service,
        })
    }
}

pub struct LoggingMiddleware<S> {
    service: S,
}

impl<S, B> Service for LoggingMiddleware<S> where S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>, S::Future: 'static, B: 'static {
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        super::utils::log(format!("{} -> {}", req.method().to_string(), req.path()).as_str());

        Box::new(self.service.call(req).and_then(|res| {
            Ok(res)
        }))
    }
}