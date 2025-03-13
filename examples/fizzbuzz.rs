#![feature(uefi_std)]

use embassy_executor::{Executor, Spawner};
use embassy_time::{Duration, Instant, Timer};
use embassy_uefi as _;
use static_cell::StaticCell;
use std::os::uefi as uefi_std;
use uefi::{
    Handle,
    boot::{self},
};

/// Performs the necessary setup code for the `uefi` crate.
fn setup_uefi_crate() {
    let st = uefi_std::env::system_table();
    let ih = uefi_std::env::image_handle();

    // Mandatory setup code for `uefi` crate.
    unsafe {
        uefi::table::set_system_table(st.as_ptr().cast());

        let ih = Handle::from_ptr(ih.as_ptr().cast()).unwrap();
        uefi::boot::set_image_handle(ih);
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

fn main() {
    setup_uefi_crate();
    std::panic::set_hook(Box::new(|p| {
        println!("{p}");
        loop {
            boot::stall(1_000_000_000)
        }
    }));

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| spawner.spawn(main_task(spawner)).unwrap());
}

#[embassy_executor::task]
async fn main_task(spawner: Spawner) {
    println!("i am spawned!");
    spawner
        .spawn(repeat("n".into(), Duration::from_secs(1)))
        .unwrap();
    spawner
        .spawn(repeat("fizz".into(), Duration::from_secs(3)))
        .unwrap();
    spawner
        .spawn(repeat("buzz".into(), Duration::from_secs(5)))
        .unwrap();
}

#[embassy_executor::task(pool_size = 4)]
async fn repeat(name: String, period: Duration) {
    let mut i = 0;
    loop {
        println!("{name} {i} @ {}", Instant::now());
        Timer::after(period).await;
        i += 1;
    }
}
