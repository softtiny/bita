use tokio::task::JoinSet;
use std::time::Duration;
use tokio::{join, try_join, runtime,};

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