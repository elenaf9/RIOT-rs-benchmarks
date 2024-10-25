#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]
#![feature(impl_trait_in_assoc_type)]

#[cfg(feature = "dual-core")]
use riot_rs::thread::sync::Channel;
use riot_rs::{debug::log::*, thread};

#[cfg(feature = "dual-core")]
static INPUT_CHANNEL: Channel<([[u16; N]; N / 2], [[u16; N]; N])> = Channel::new();
#[cfg(feature = "dual-core")]
static RESULT_CHANNEL: Channel<[[u16; N]; N / 2]> = Channel::new();

#[cfg(feature = "n10")]
const N: usize = 10;
#[cfg(feature = "n20")]
const N: usize = 20;
#[cfg(feature = "n30")]
const N: usize = 30;
#[cfg(feature = "n40")]
const N: usize = 40;

fn matrix_mult<const M: usize>(
    matrix_a: &[[u16; N]; M],
    matrix_b: &[[u16; N]; N],
) -> [[u16; N]; M] {
    let mut matrix_c = [[0; N]; M];
    for i in 0..M {
        for j in 0..N {
            for k in 0..N {
                matrix_c[i][j] += matrix_a[i][k] * matrix_b[k][j]
            }
        }
    }
    matrix_c
}

#[riot_rs::task(autostart)]
async fn start() {
    thread::thread_flags::set(thread::ThreadId::new(0), 1);
}

#[riot_rs::thread(autostart, stacksize = 32768)]
fn thread0() {
    while thread::thread_flags::get() == 0 {}

    let matrix_a = core::hint::black_box([[3; N]; N]);
    let matrix_b = core::hint::black_box([[7; N]; N]);
    match bench_multicore::benchmark(10, || {
        #[cfg(not(feature = "dual-core"))]
        {
            let matrix_c = matrix_mult(&matrix_a, &matrix_b);
            core::hint::black_box(matrix_c);
        }
        #[cfg(feature = "dual-core")]
        {
            let (matrix_a1, matrix_a2) = matrix_a.split_at(N / 2);
            let matrix_a1: [_; N / 2] = matrix_a1.try_into().unwrap();
            let matrix_a2: [_; N / 2] = matrix_a2.try_into().unwrap();

            INPUT_CHANNEL.send(&(matrix_a2, matrix_b));

            let matrix_c1 = matrix_mult(&matrix_a1, &matrix_b);
            let matrix_c2 = RESULT_CHANNEL.recv();

            core::hint::black_box((matrix_c1, matrix_c2));
        }
    }) {
        Ok(ticks) => info!("took {} ticks per iteration", ticks),

        Err(err) => error!("benchmark error: {}", err),
    }
}
#[cfg(feature = "dual-core")]
#[riot_rs::thread(autostart, stacksize = 32768)]
fn thread1() {
    loop {
        let (matrix_a, matrix_b) = INPUT_CHANNEL.recv();
        let matrix_c = matrix_mult(&matrix_a, &matrix_b);
        RESULT_CHANNEL.send(&matrix_c);
    }
}
