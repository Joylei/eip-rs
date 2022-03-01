use std::cell::UnsafeCell;
use std::sync::Arc;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rseip::cip::connection::OpenOptions;
use rseip::client::ab_eip::*;
use rseip::precludes::*;

fn bench_read(c: &mut Criterion) {
    c.bench_function("async read", |b| {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let client = rt.block_on(async {
            let client = AbEipConnection::new_host_lookup("192.168.0.83", OpenOptions::default())
                .await
                .unwrap();
            Arc::new(UnsafeCell::new(client))
        });
        b.to_async(rt).iter_batched(
            || client.clone(),
            |client| async move {
                let tag = EPath::from_symbol("test_car1_x");
                let client = unsafe { &mut *client.get() };
                let _value: TagValue<i32> = client.read_tag(tag).await.unwrap();
            },
            BatchSize::PerIteration,
        )
    });
}

criterion_group!(benches, bench_read);
criterion_main!(benches);
