use tokio::task::JoinSet;
use std::time::Duration;
use tokio::{join, try_join, runtime,};
use std::future::Future;

fn print_time(){
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap();
    let all = duration.as_secs();
    println!("all start is: {}",all);
    let mut diff = all%(3600*24);
    println!("diff start is: {}",diff);
    let hour = diff/3600 + 8;
    diff = diff %3600;
    let minus = diff / 60;
    diff = diff % 60;
    let seconds = diff;
    println!("time:{}:{}:{}", hour,minus,seconds);
}

async fn tokio_flow_join_set() {

    // all start is: 1740565436
    // diff start is: 37436
    // Task 1 completed,time:SystemTime { intervals: 133850390364833944 },18:23:56
    // Task output: 2
    // all start is: 1740565441
    // diff start is: 37441
    // Task 2 completed,time:SystemTime { intervals: 133850390414747589 },18:24:1
    // Task output: 4


    let mut set = JoinSet::new();

    for i in 1..3 {
        set.spawn(async move {
            tokio::time::sleep(Duration::from_secs(i*5)).await;

            println!("Task {} completed", i,);
            print_time();
            i * 2
        });
    }

    while let Some(res) = set.join_next().await {
        match res {
            Ok(output) => println!("Task output: {}", output),
            Err(e) => println!("Task panicked: {}", e),
        }
    }
}



async fn first_future() -> Result<i32, String> {
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    println!("First future completed");
    print_time();
    Ok(1)
}

async fn second_future() -> Result<i32, String> {
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    println!("Second future completed");
    print_time();
    Ok(2)
}


async fn tokio_flow_try_join() -> Result<(), String> {
    // Second future completed
    // all start is: 1740565666
    // diff start is: 37666
    // time:18:27:46
    // First future completed
    // all start is: 1740565667
    // diff start is: 37667
    // time:18:27:47
    // Result 1: 1, Result 2: 2
    //
    let (result1, result2) = try_join!(first_future(), second_future()).expect("fail to run try josin");
    println!("Result 1: {}, Result 2: {}", result1, result2);
    Ok(())
}

#[tokio::test]
async fn tokio_flow_join() -> Result<(), String> {
    // Second future completed
    // all start is: 1740566158
    // diff start is: 38158
    // time:18:35:58
    // First future completed
    // all start is: 1740566159
    // diff start is: 38159
    // time:18:35:59
    // Result 1: 1, Result 2: 2
    let (result1, result2) = join!(first_future(), second_future());
    println!("Result 1: {}, Result 2: {}", result1.expect("not rusult1"), result2.expect("not result2"));
    Ok(())
}

#[test]
fn go_other_thread() {

    runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            tokio::time::sleep(Duration::from_secs(55)).await;

        })
}

#[test]
fn current_thread() {
    runtime::Builder::new_current_thread()
    //A Tokio 1.x context was found, but timers are disabled. Call `enable_time` on the runtime builder to enable timers
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            tokio::time::sleep(Duration::from_secs(55)).await;
        })
}

#[test]
fn loop_find_run(){
    let mut count = 0;
    loop {
        count += 1;
        if count > 2{
            count = 0;
        }
    }
}

#[tokio::test]
async fn loop_find_async(){
    let mut count = 0;
    loop {
        count += 1;
        if count > 2{
            count = 0;
        }
    }
}


#[tokio::test]
async fn loop_async_sleep_await(){
    tokio::time::sleep(std::time::Duration::from_secs(13203)).await;
}

#[tokio::test]
async fn loop_asyncs_join(){
    //method cannot be called on `Pin<&mut MaybeDone<()>>` due to unsatisfied trait bounds
    //join!(tokio::time::sleep(std::time::Duration::from_secs(13203)).await);
    join!(
        tokio::time::sleep(std::time::Duration::from_secs(13203)),
        tokio::time::sleep(std::time::Duration::from_secs(13203)),
        tokio::time::sleep(std::time::Duration::from_secs(13203)),
        tokio::time::sleep(std::time::Duration::from_secs(13203)),
        tokio::time::sleep(std::time::Duration::from_secs(13203))
    );
}



#[test]
fn build_thread_w2(){
    runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            let mut count = 0;
            loop {
                count += 1;
                if count > 2{
                    count = 0;
                }
            }

        })
}


struct SimpleFuture {
    counter: u32,
}

impl SimpleFuture {
    fn new() -> Self {
        SimpleFuture { counter: 0 }
    }
}

impl std::future::Future for SimpleFuture {
    type Output = String;
    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        self.counter += 1;
        if self.counter > 3 {
            std::task::Poll::Ready(format!("Future resolved after {} polls", self.counter))
        } else {
            println!("Future is pending,poll count: {}", self.counter);
            cx.waker().wake_by_ref();
            std::task::Poll::Pending
        }
    }
}

#[tokio::test]
async fn test_simple_future(){
    println!("run simplefuture");
    let simple = SimpleFuture::new();
    let res = simple.await;
    println!("run simplefuture:{}",res);
}



// Define dummy functions for the vtable operations
unsafe fn dummy_clone(data: *const ()) -> std::task::RawWaker {
    std::task::RawWaker::new(data, &DUMMY_VTABLE)
}

unsafe fn dummy_wake(data: *const ()) {
    // Do nothing
}

unsafe fn dummy_wake_by_ref(data: *const ()) {
    // Do nothing
}

unsafe fn dummy_drop(data: *const ()) {
    // Do nothing
}

// Create the dummy RawWakerVTable
static DUMMY_VTABLE: std::task::RawWakerVTable = std::task::RawWakerVTable::new(
    dummy_clone,
    dummy_wake,
    dummy_wake_by_ref,
    dummy_drop,
);


fn minimal_waker() -> std::task::Waker {
    // Create a dummy raw waker vtable
    let raw_waker_vtable = &std::task::RawWakerVTable::new(
        |_| std::task::RawWaker::new(std::ptr::null(), &DUMMY_VTABLE), // clone
        |_| {},                                               // wake
        |_| {},                                               // wake_by_ref
        |_| {},                                               // drop
    );

    // Create a dummy raw waker
    let raw_waker = std::task::RawWaker::new(std::ptr::null(), raw_waker_vtable);

    // Create a Waker from the raw waker
    unsafe { std::task::Waker::from_raw(raw_waker) }
}

#[test]
fn run_future_simple() {
    let mut future  = SimpleFuture::new();
    let mut future = unsafe { std::pin::Pin::new_unchecked(&mut future) };
    
    let waker = minimal_waker();
    let mut context = std::task::Context::from_waker(&waker);

    loop {
        match future.as_mut().poll(&mut context){
            std::task::Poll::Ready(result) => {
                println!("Result: {}", result);
                break;
            }
            std::task::Poll::Pending => {
                println!("Future is pending");
                std::thread::sleep(std::time::Duration::from_millis(5000));
            }
        }
    }
}