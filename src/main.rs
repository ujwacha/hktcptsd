use hktcptsd::get_addr_thread;
use hktcptsd::{connection_handler, ThreadPool};
use std::net::TcpListener;

fn main() {
    let (adress, no_of_threads) = get_addr_thread();

    let listener = match TcpListener::bind(adress) {
        Ok(t) => t,
        Err(t) => panic!("[-]ERROR: {t}"),
    };

    let pool = ThreadPool::new(no_of_threads);

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(t) => t,
            Err(_t) => continue,
        };

        pool.execute(move || {
            connection_handler(stream);
        })
    }
}
