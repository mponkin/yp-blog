//! Blog-client library

#![deny(unreachable_pub)]
#![warn(missing_docs)]

pub mod blog_client;
pub mod error;
mod grpc_client;
mod http_client;

/// Available trqnsports for blog clients
pub enum Transport {
    /// Http client with server address
    Http(String),
    /// Grpc client with server address
    Grpc(String),
}
