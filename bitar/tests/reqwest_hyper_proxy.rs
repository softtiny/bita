use std::convert::Infallible;
use std::net::SocketAddr;
use std::thread::JoinHandle;
use bytes::Bytes;
use http_body_util::Full;
use hyper::{Request, Response};
use hyper::rt::Executor;
use hyper::server::conn::http2;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use merino::*;
use tokio::net::TcpListener;

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

async fn handle_request(_req: Request<hyper::body::Incoming>) -> Result<Response<Full<hyper::body::Bytes>>, Infallible> {
    Ok(Response::new(Full::from( Bytes::from("hello world"))))
}


async fn reqwest_request_2s(url:&str) -> Result<String, Box<dyn std::error::Error>>{

    let client = reqwest::Client::builder().http2_prior_knowledge().timeout(std::time::Duration::from_secs(2)).build().expect("failed to build client");

    let res = client.get(url).send().await?;
    let msg = res.text().await.expect("get text err");
    println!("msg is: {}",msg);
    Ok(msg)
}

async fn reqwest_request_proxy(url:&str) -> Result<String, Box<dyn std::error::Error>>{
    println!("reqwest_request_proxy runing");
    let proxy = reqwest::Proxy::http("socks5://127.0.0.1:3001")?;
    println!("reqwest_request_proxy proxy runing");
    let client = reqwest::Client::builder()
        .http2_prior_knowledge()
        .timeout(std::time::Duration::from_secs(2))
        .proxy(proxy)
        .build().expect("failed to build client");
    println!("reqwest_request_proxy client runing");
    let res = client.get(url).send().await?;
    println!("reqwest_request_proxy res runing");
    let msg = res.text().await.expect("get text err");
    println!("msg is: {}",msg);
    Ok(msg)
}

#[tokio::test]
async fn request_again() -> Result<(),Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127,0,0,1],3000));
    let listener = TcpListener::bind(addr).await.expect("bind port error");
    Ok(())
}



async fn request_server_proxy() -> Result<(),Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127,0,0,1],3000));
    let listener = TcpListener::bind(addr).await.expect("bind port erro");
    let server_task = tokio::task::spawn(async move {
        loop{
            let (stream,_err) = listener.accept().await.expect("accpet listen err");
            let io = TokioIo::new(stream);
            tokio::task::spawn(async move {
                //TokioExecutor
                if let Err(err) = http2::Builder::new(TokioExecutor)
                    .serve_connection(io,service_fn(handle_request))
                    .await {
                    eprintln!("http2 server error ..{:?}",err);
                }
            });
        }
    });
    println!("new on mian");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    tokio::task::spawn(async {
        tokio::time::sleep(std::time::Duration::from_secs(1));
        println!("timeout wait 1s,still wating");
    });
    if let Err(err)  = reqwest_request_2s("http://127.0.0.1:3000").await {
        println!("timout 2s err occur:{:?}",err);
    }
    println!("after client req s");

    let mut auth_methods: Vec<u8> = Vec::new();
    auth_methods.push(merino::AuthMethods::NoAuth as u8);
    println!("run proxy task start");
    let mut proxy_task = tokio::task::spawn(async {
        println!("go proxy task created");
        // let sbres = tokio::task::spawn_blocking( move ||{
        //     let mut merino = Merino::new(3001, "127.0.0.1", auth_methods, Vec::new(),None).expect("failed create proxy");
        //     merino.serve();
        //     println!("ok proxy task created");
        // });
        let mut merino = Merino::new(3001,"127.0.0.1",auth_methods,Vec::new(),None).await.expect("failed create proxy");
        merino.serve().await;
        println!("runing after proxy thread");

    });
    // let proxy_task= tokio::task::spawn_blocking(move ||{
    //     println!("go proxy task created");
    //
    //     let mut merino = Merino::new(3001, "127.0.0.1".to_string(), auth_methods, Vec::new()).expect("proxy server fail");
    //     merino.serve().expect("fail start proxy serve");
    //     println!("ok proxy task created");
    // });
    println!("run proxy task created");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    println!("run proxy request start");
    if let Err(err) = reqwest_request_proxy("http://127.0.0.1:3000").await {
        println!("request proxy err occur:{:?}",err);
    }
    println!("after client req pass proxy");
    server_task.abort();
    println!("after server task abort listen port gone");
    println!("wait 1s");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    println!("wait 1s done");

    proxy_task.abort();
    println!("after proxy task abort listen port is gone");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    println!("wait 1s done");


    //Calling abort() on a JoinHandle obtained from task::spawn_blocking will not terminate the blocking task
    //proxy_task.abort();
    //println!("after proxy task abort");
    Ok(())
}
