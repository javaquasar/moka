use moka::future::Cache;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering};

struct CountingAllocator;

static ALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);
static ALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATED_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATED_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
        ALLOCATION_COUNT.fetch_add(1, Ordering::Relaxed);
        System.realloc(ptr, layout, new_size)
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

#[derive(Clone, Copy)]
enum Mode {
    Off,
    Listener,
    Observer,
}

impl Mode {
    fn parse(value: &str) -> Self {
        match value {
            "off" => Self::Off,
            "listener" => Self::Listener,
            "observer" => Self::Observer,
            _ => panic!("mode must be one of: off, listener, observer"),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Listener => "listener",
            Self::Observer => "observer",
        }
    }
}

#[derive(Clone, Copy)]
enum Operation {
    Insert,
    Remove,
}

impl Operation {
    fn parse(value: &str) -> Self {
        match value {
            "insert" => Self::Insert,
            "remove" => Self::Remove,
            _ => panic!("operation must be one of: insert, remove"),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Insert => "insert",
            Self::Remove => "remove",
        }
    }
}

fn snapshot() -> (u64, u64) {
    (
        ALLOCATED_BYTES.load(Ordering::Relaxed),
        ALLOCATION_COUNT.load(Ordering::Relaxed),
    )
}

fn delta(before: (u64, u64), after: (u64, u64)) -> (u64, u64) {
    (after.0 - before.0, after.1 - before.1)
}

fn cache(mode: Mode, capacity: u64) -> Cache<u64, u64> {
    let builder = Cache::builder().max_capacity(capacity);
    match mode {
        Mode::Off => builder.build(),
        Mode::Listener => builder.eviction_listener(|_, _, _| {}).build(),
        Mode::Observer => builder.post_removal_observer(|_, _, _| {}).build(),
    }
}

async fn insert_sample(mode: Mode, operations: u64) -> (u64, u64) {
    let cache = cache(mode, operations * 2);
    cache.insert(u64::MAX, u64::MAX).await;
    let before = snapshot();
    for key in 0..operations {
        cache.insert(key, key).await;
    }
    delta(before, snapshot())
}

async fn remove_sample(mode: Mode, operations: u64) -> (u64, u64) {
    let cache = cache(mode, operations * 2);
    for key in 0..operations {
        cache.insert(key, key).await;
    }
    cache.run_pending_tasks().await;
    let before = snapshot();
    for key in 0..operations {
        cache.invalidate(&key).await;
    }
    cache.run_pending_tasks().await;
    delta(before, snapshot())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut args = std::env::args().skip(1);
    let mode = Mode::parse(&args.next().expect("usage: BENCH MODE OPERATION [OPERATIONS]"));
    let operation = Operation::parse(
        &args
            .next()
            .expect("usage: BENCH MODE OPERATION [OPERATIONS]"),
    );
    let operations = args
        .next()
        .map(|value| value.parse::<u64>().expect("operations must be an integer"))
        .unwrap_or(20_000);
    assert!(args.next().is_none(), "too many arguments");

    let sample = match operation {
        Operation::Insert => insert_sample(mode, operations).await,
        Operation::Remove => remove_sample(mode, operations).await,
    };
    println!(
        "{},{},{},{},{},{:.6},{:.6}",
        mode.name(),
        operation.name(),
        operations,
        sample.0,
        sample.1,
        sample.0 as f64 / operations as f64,
        sample.1 as f64 / operations as f64,
    );
}
