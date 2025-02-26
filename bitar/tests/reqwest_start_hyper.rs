use std::convert::Infallible;
use std::net::{SocketAddr};
use http_body_util::Full;
use hyper::server::conn::http2;
use hyper::service::service_fn;
use hyper::body::Bytes;
use hyper::{Request, Response};
use hyper::rt::Executor;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;


async fn hello(request: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    println!("heloo...................");
    Ok(Response::new(Full::new(Bytes::from("hello world"))))
}

#[derive(Clone)]
pub struct TokioExecutor;

impl<F> Executor<F> for TokioExecutor
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    fn execute(&self, fut: F) {
        tokio::task::spawn(fut);
    }
}

#[tokio::test]
async fn hyper_server_start() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127,0,0,1], 3000));

    let listener = TcpListener::bind(addr).await.expect("faild to listen 3000 port");
    println!("run: `curl -v --http2-prior-knowledge http://127.0.0.1:3000` command to send http2 request");
    loop {
        let (stream,_) = listener.accept().await?;
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            // need TokioEXecutor struct
            println!("run: `curl -v --http2-prior-knowledge http://127.0.0.1:3000` command to send http2 request");
            if let Err(err) =http2::Builder::new(TokioExecutor)
                .serve_connection(io, service_fn(hello))
                .await
            {
                eprintln!("http2 server error {}", err);
            }
        });
    }


    Ok(())
}