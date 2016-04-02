This is a tiny Rust crate containing an Iron middleware. The middleware serves to work around a [bug in hyper](https://github.com/hyperium/hyper/issues/309) that can cause sockets to be reused without the previous request being fully read out, which always causes the next request to fail in parsing.

For usage, see the documentation.

