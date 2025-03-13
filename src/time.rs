use core::{
    ffi::c_void,
    ptr::NonNull,
    sync::atomic::{AtomicBool, Ordering},
    task::Waker,
};

use embassy_time_driver::Driver;
use spin::mutex::Mutex;
use uefi::{
    Event,
    boot::{self, EventType, Tpl, create_event, set_timer},
};

const MAX_TIMERS: usize = 128;

struct UefiTimeDriver {
    contexts: Mutex<ContextAllocator>,
}

embassy_time_driver::time_driver_impl!(
    static DRIVER: UefiTimeDriver = UefiTimeDriver {
        contexts: Mutex::new(ContextAllocator { items: [const { None }; MAX_TIMERS] })
    }
);

impl Driver for UefiTimeDriver {
    fn now(&self) -> u64 {
        instant_internal::timestamp_rdtsc().unwrap()
    }

    fn schedule_wake(&self, at: u64, waker: &Waker) {
        let event = unsafe {
            let mut lock = self.contexts.lock();
            let context = lock.allocate_event(waker).unwrap();
            let context_ptr = NonNull::new_unchecked(context as *mut _);
            create_event(
                EventType::TIMER | EventType::NOTIFY_SIGNAL,
                Tpl::NOTIFY,
                Some(notify),
                Some(context_ptr.cast()),
            )
        }
        .unwrap();

        // From https://uefi.org/specs/UEFI/2.9_A/07_Services_Boot_Services.html#efi-boot-services-settimer
        // TriggerTime - The number of 100ns units until the timer expires. A TriggerTime of 0 is legal.
        let delta_ns = at - self.now();
        set_timer(&event, boot::TimerTrigger::Relative(delta_ns / 100)).unwrap();
    }
}

unsafe extern "efiapi" fn notify(event: Event, context: Option<NonNull<c_void>>) {
    let ctx = unsafe {
        context
            .expect("got non-null context")
            .cast::<EventContext>()
            .as_mut()
    };
    ctx.waker.wake_by_ref();
    ctx.done.store(true, Ordering::Relaxed);
}

/// A simple bump allocator.
struct ContextAllocator {
    items: [Option<EventContext>; MAX_TIMERS],
}

impl ContextAllocator {
    fn allocate_event(&mut self, waker: &Waker) -> Option<&mut EventContext> {
        match self.next_available() {
            Some(cell) => {
                *cell = Some(EventContext {
                    waker: waker.clone(),
                    done: false.into(),
                });
                cell.as_mut()
            }
            None => None,
        }
    }

    fn next_available(&mut self) -> Option<&mut Option<EventContext>> {
        for item in &mut self.items {
            match item {
                None => return Some(item),
                Some(e) if e.done.load(Ordering::Relaxed) => return Some(item),
                _ => continue,
            }
        }
        None
    }
}

struct EventContext {
    waker: Waker,
    done: AtomicBool,
}

// inspired by rust-std sys/pal/uefi/time.rs
mod instant_internal {
    /// returns nanoseconds
    #[cfg(target_arch = "x86_64")]
    pub fn timestamp_rdtsc() -> Option<u64> {
        use spin::Once;

        enum Frequency {
            Known(u64),
            Unsupported,
        }

        static FREQUENCY: Once<Frequency> = Once::new();

        // Get Frequency in Mhz
        // Inspired by [`edk2/UefiCpuPkg/Library/CpuTimerLib/CpuTimerLib.c`](https://github.com/tianocore/edk2/blob/master/UefiCpuPkg/Library/CpuTimerLib/CpuTimerLib.c)
        let frequency = FREQUENCY.call_once(|| {
            let cpuid = unsafe { core::arch::x86_64::__cpuid(0x15) };
            if cpuid.eax == 0 || cpuid.ebx == 0 || cpuid.ecx == 0 {
                return Frequency::Unsupported;
            }
            let f = cpuid.ecx as u64 * cpuid.ebx as u64 / cpuid.eax as u64;
            Frequency::Known(f)
        });

        let frequency = match frequency {
            Frequency::Known(f) => f,
            Frequency::Unsupported => return None,
        };

        let ts = unsafe { core::arch::x86_64::_rdtsc() };
        let ns = ts * 1000 / frequency;
        Some(ns)
    }
}
