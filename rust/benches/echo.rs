#![feature(test)]
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;
use futures::stream::StreamExt;

const SIZE: u64 = 1000;
const COUNT: u64 = 1000;

extern crate test;

#[bench]
fn bench_tcp(b: &mut test::Bencher) {
    let address = "localhost:11011";

    let run_server = || {
        task::block_on(async {
            task::spawn(async move {
                let listener = TcpListener::bind(&address).await.unwrap();
                let mut incoming = listener.incoming();
                while let Some(Ok(mut stream)) = incoming.next().await {
                    task::spawn(async move {
                        let mut read = stream.clone();
                        let write = &mut stream;
                        async_std::io::copy(&mut read, write).await.unwrap();
                    });
                }
            })
        })
    };

    b.bytes = SIZE * COUNT;
    let server = run_server();
    b.iter(|| {
        task::block_on(async move {
            let data = vec![1u8; SIZE as usize];
            let mut buf = vec![0u8; (SIZE * COUNT) as usize];

            let mut stream = TcpStream::connect(&address).await.unwrap();
            let mut stream_clone = stream.clone();

            let writer = task::spawn_local(async move {
                for _i in 0..COUNT {
                    stream_clone.write_all(&data).await.unwrap();
                    stream_clone.flush().await.unwrap();
                }
            });

            let reader = task::spawn_local(async move {
                stream.read_exact(&mut buf).await.unwrap();
                assert!(&buf[0] == &1u8);
                assert!(&buf[buf.len() - 1] == &1u8);
            });
            reader.await;
            writer.await;
        })
    });
    task::block_on(async move { server.cancel().await });
}
