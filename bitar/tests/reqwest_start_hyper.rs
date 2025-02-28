use std::convert::Infallible;
use std::net::{SocketAddr};
use http_body_util::Full;
use hyper::server::conn::http2;
use hyper::service::service_fn;
use hyper::body::{Body, Bytes};
use hyper::{Request, Response};
use hyper::rt::Executor;
use hyper_util::rt::TokioIo;
use tokio::join;
use tokio::net::TcpListener;



fn print_time(){
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
    let all = duration.as_secs();
    //println!("all start is: {}",all);
    let mut diff = all%(3600*24);
    //println!("diff start is: {}",diff);
    let hour = diff/3600 + 8;
    diff = diff %3600;
    let minus = diff / 60;
    diff = diff % 60;
    let seconds = diff;
    println!("time:{}:{}:{}", hour,minus,seconds);
}

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

async fn reqwest_request() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    let url = "http://127.0.0.1:3000/";
    let client = reqwest::Client::builder().http2_prior_knowledge().build().expect("failed to build client");

    let res = client.get(url).send().await.expect("error request");
    let data = res.text().await.expect("fail to text");
    println!("response from server:{}",data);
    Ok(())
}

// request ok
// server ok
// join! ok

async fn request_with_join() -> Result<(), Box<dyn std::error::Error>> {

    join!(hyper_server_start(),reqwest_request());
    Ok(())
}


// Example a  start ---- aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
async fn handle_request_wait_5(_req: Request<hyper::body::Incoming>) -> Result<Response<Full<hyper::body::Bytes>>, Infallible> {
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    Ok(Response::new(Full::from( Bytes::from("hello world"))))
}

async fn reqwest_request_2s(url:&str) -> Result<String, Box<dyn std::error::Error>>{

    let client = reqwest::Client::builder().http2_prior_knowledge().timeout(std::time::Duration::from_secs(2)).build().expect("failed to build client");

    let res = client.get(url).send().await?;
    let msg = res.text().await.expect("get text err");
    println!("msg is: {}",msg);
    Ok(msg)
}


async fn request_timeout_work() -> Result<(), Box<dyn std::error::Error>> {
    // Server setup
    let addr = SocketAddr::from(([127,0,0,1],3000));
    let listener = TcpListener::bind(addr).await.expect("bind port erro");
    let server_task = tokio::task::spawn(async move {
        loop{
            let (stream,_err) = listener.accept().await.expect("accpet listen err");
            let io = TokioIo::new(stream);
            tokio::task::spawn(async move {
                //TokioExecutor
                if let Err(err) = http2::Builder::new(TokioExecutor)
                    .serve_connection(io,service_fn(handle_request_wait_5))
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
    server_task.abort();
    Ok(())
}

// Example a  end ---- aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa

// Example b start ---- bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
//async fn handleaquest_wai5(req:Request<hyper::body::Incoming>) -> Result<Response<Full<hyper::body::Bytes>>, Infallible> {
async fn handle_request_path(req:Request<hyper::body::Incoming>) -> Result<Response<Full<hyper::body::Bytes>>, Infallible> {
    match req.uri().path() {
        "/slow" => {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            Ok(Response::new(Full::new(Bytes::from("after 5s response"))))
        },
        _ => Ok(Response::new(Full::new(Bytes::from("fast response"))))
    }
}

async fn request_and_path() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127,0,0,1],3000));
    let stream = TcpListener::bind(addr).await.expect("failed to bind port");
    let server_task = tokio::task::spawn(async move {
        loop {
            let (stream,_) = stream.accept().await.expect("accept err");
            let io = TokioIo::new(stream);
            if let Err(err) = http2::Builder::new(TokioExecutor)
                .serve_connection(io,service_fn(handle_request_path))
                .await {
                eprintln!("http2 server path err: {:?}",err);
            }
        }
    });
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    tokio::task::spawn(async {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        println!("request fast clock,print after 1s,after fast response,see path fast msg ^^^");
    });
    println!("req fast start timeis");
    print_time();
    if let Ok(msg) = reqwest_request_2s("http://127.0.0.1:3000/fast").await {
        println!("path fast msg:{}",msg);
    } else {
        eprintln!("err: path fast msg gone");
    }
    println!("req fast end timeis");
    print_time();
    println!("req slow start timeis");
    print_time();
    if let Err(err) = reqwest_request_2s("http://127.0.0.1:3000/slow").await {
        println!("slow reqeust timeout 2s err:{:?}",err);
    }
    println!("req slow end timeis");
    print_time();
    server_task.abort();
    Ok(())
}
// Example b end ---- bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb

// Example c start ---- ccccccccccccccccccccccc
#[tokio::test]
async fn request_no_server() -> Result<(), Box<dyn std::error::Error>> {
    println!("req slow start timeis");
    print_time();
    if let Err(err) = reqwest_request_2s("http://127.0.0.1:3000").await {
        println!("slow reqeust timeout 2s err:{:?}",err);
    }
    println!("req slow end timeis");
    print_time();
    Ok(())
}
// Example c end ---- ccccccccccccccccccccccc