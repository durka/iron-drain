//! Iron middleware that makes sure requests are read in full before reusing sockets
//!
//! Hyper keeps sockets alive to reuse them between requests, to speed things up. If a request
//! that isn't supposed to have a body is sent with one, or if the server does not read out the
//! full body of a request, then the next request will be corrupted due to data remaining in
//! the network buffer.
//!
//! The `Drain` adapter in this module defines an iron `AfterMiddleware` that makes sure to empty
//! the buffer before the next request, whether the current request succeeded or failed. It reads
//! up to a configurable limit, and if there is still more data remaining, it closes the socket.
//!
//! Usage:
//!
//! ```rust
//! extern crate iron;
//!
//! use iron::prelude::*;
//! use iron::status;
//!
//! # fn main() {
//! let mut srv = Chain::new(|_: &mut Request| {
//!     Ok(Response::with((status::Ok, "Hello world!")))
//! });
//! srv.link_after(Drain::new());
//! # || {
//! srv.http("localhost:3000").unwrap();
//! # };
//! # }
//! ```

extern crate iron;

use std::io::{self, Read};
use iron::prelude::*;
use iron::headers::Connection;
use iron::middleware::AfterMiddleware;

/// Iron middleware that makes sure requests are read in full before reusing sockets
pub struct Drain { limit: u64 }

impl Drain {
    /// Create a Drain with the default limit (1MB)
    pub fn new() -> Drain {
        Drain::with_limit(1024 * 1024)
    }

    /// Create a Drain with a custom limit
    pub fn with_limit(limit: u64) -> Drain {
        Drain {
            limit: limit
        }
    }

    fn drain(&self, req: &mut Request, resp: &mut Response) {
        // try reading up to the limit
        if io::copy(&mut req.body.by_ref().take(self.limit), &mut io::sink()).is_ok() {
            // see if there's anything left
            let mut buf = [0];
            if let Ok(n) = req.body.read(&mut buf) {
                if n == 0 {
                    return;
                }
            }
        }

        // there's too much data or an error occurred, so just close the connection
        resp.headers.set(Connection::close());
    }
}

impl AfterMiddleware for Drain {
    fn after(&self, req: &mut Request, mut resp: Response) -> IronResult<Response> {
        self.drain(req, &mut resp);
        Ok(resp)
    }

    fn catch(&self, req: &mut Request, mut err: IronError) -> IronResult<Response> {
        self.drain(req, &mut err.response);
        Err(err)
    }
}

