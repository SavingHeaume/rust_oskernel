#![no_std]
#![no_main]
#[macro_use]
extern crate user_lib;
extern crate alloc;
use alloc::vec;
use user_lib::exit;
use user_lib::{
    mutex_blocking_create, mutex_lock, mutex_unlock, semaphore_create, semaphore_down, semaphore_up,
};
use user_lib::{sleep, thread_create, waittid};

// 缓冲区相关常量
const BUFFER_SIZE: usize = 5;
const PRODUCER_COUNT: usize = 2;
const CONSUMER_COUNT: usize = 2;
const ITEMS_PER_PRODUCER: usize = 3;

// 同步原语ID
const MUTEX_ID: usize = 0;
const EMPTY_SLOTS_SEM: usize = 0; // 空槽位信号量
const FILLED_SLOTS_SEM: usize = 1; // 已填充槽位信号量

// 共享缓冲区
static mut BUFFER: [usize; BUFFER_SIZE] = [0; BUFFER_SIZE];
static mut IN: usize = 0; // 生产者写入位置
static mut OUT: usize = 0; // 消费者读取位置

fn producer(id: *const usize) -> ! {
    let id = unsafe { *id };

    for i in 0..ITEMS_PER_PRODUCER {
        let item = id * 100 + i; // 生产的项目

        // 等待空槽位
        semaphore_down(EMPTY_SLOTS_SEM);

        // 获取互斥锁保护缓冲区
        mutex_lock(MUTEX_ID);

        // 生产项目到缓冲区
        unsafe {
            let current_in = IN;
            BUFFER[current_in] = item;
            println!(
                "Producer {} produced item {} at position {}",
                id, item, current_in
            );
            IN = (current_in + 1) % BUFFER_SIZE;
        }

        // 释放互斥锁
        mutex_unlock(MUTEX_ID);

        // 增加已填充槽位计数
        semaphore_up(FILLED_SLOTS_SEM);

        // 模拟生产时间
        sleep(50);
    }

    println!("Producer {} finished", id);
    exit(0)
}

fn consumer(id: *const usize) -> ! {
    let id = unsafe { *id };
    let mut consumed_count = 0;

    loop {
        // 等待已填充的槽位
        semaphore_down(FILLED_SLOTS_SEM);

        // 获取互斥锁保护缓冲区
        mutex_lock(MUTEX_ID);

        // 从缓冲区消费项目
        let item = unsafe {
            let current_out = OUT;
            let item = BUFFER[current_out];
            println!(
                "Consumer {} consumed item {} from position {}",
                id, item, current_out
            );
            OUT = (current_out + 1) % BUFFER_SIZE;
            item
        };

        // 释放互斥锁
        mutex_unlock(MUTEX_ID);

        // 增加空槽位计数
        semaphore_up(EMPTY_SLOTS_SEM);

        consumed_count += 1;

        // 模拟处理消费的项目
        println!("Consumer {} processing item {}", id, item);

        // 如果消费了足够的项目就退出
        if consumed_count >= (PRODUCER_COUNT * ITEMS_PER_PRODUCER) / CONSUMER_COUNT {
            break;
        }

        // 模拟消费时间
        sleep(80);
    }

    println!(
        "Consumer {} finished, consumed {} items",
        id, consumed_count
    );
    exit(0)
}

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("Producer-Consumer with Semaphores");
    println!(
        "Buffer size: {}, Producers: {}, Consumers: {}",
        BUFFER_SIZE, PRODUCER_COUNT, CONSUMER_COUNT
    );

    // 创建同步原语
    assert_eq!(mutex_blocking_create() as usize, MUTEX_ID);
    assert_eq!(semaphore_create(BUFFER_SIZE) as usize, EMPTY_SLOTS_SEM); // 初始有BUFFER_SIZE个空槽位
    assert_eq!(semaphore_create(0) as usize, FILLED_SLOTS_SEM); // 初始没有已填充槽位

    let mut threads = vec![];
    let mut ids = vec![];

    // 创建生产者ID数组
    for i in 0..PRODUCER_COUNT {
        ids.push(i);
    }

    // 创建消费者ID数组
    for i in 0..CONSUMER_COUNT {
        ids.push(i);
    }

    // 创建生产者线程
    for i in 0..PRODUCER_COUNT {
        threads.push(thread_create(
            producer as usize,
            &ids[i] as *const _ as usize,
        ));
    }

    // 创建消费者线程
    for i in 0..CONSUMER_COUNT {
        threads.push(thread_create(
            consumer as usize,
            &ids[PRODUCER_COUNT + i] as *const _ as usize,
        ));
    }

    // 等待所有线程完成
    for thread in threads.iter() {
        waittid(*thread as usize);
    }

    println!("Producer-Consumer test with semaphores passed!");
    0
}
