use axum::body::Body as AxumBody;
use core::fmt::{self, Display};
use hyper::body::{Body as HyperBody, Bytes, Frame, Incoming};
use std::{error::Error, pin::Pin};

pub enum Body {
    Axum(AxumBody),
    Hyper(Incoming),
}

impl HyperBody for Body {
    type Data = Bytes;
    type Error = BodyError;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match self.get_mut() {
            Body::Axum(body) => Pin::new(body)
                .poll_frame(cx)
                .map(|o| o.map(|r| r.map_err(BodyError::Axum))),
            Body::Hyper(incoming) => Pin::new(incoming)
                .poll_frame(cx)
                .map(|o| o.map(|r| r.map_err(BodyError::Hyper))),
        }
    }
}

#[derive(Debug)]
pub enum BodyError {
    Axum(axum::Error),
    Hyper(hyper::Error),
}

impl Display for BodyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BodyError::Axum(e) => format!("axum body error: {e:?}"),
                BodyError::Hyper(e) => format!("hyper body error: {e:?}"),
            }
        )
    }
}

impl Error for BodyError {}
