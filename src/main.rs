use std::net::TcpListener;
use hktcptsd::{ThreadPool, connection_handler};


fn main() {
    let listener = match TcpListener::bind("127.0.0.1:6969") {
	Ok(t) => t,
	Err(t) => panic!("[-]ERROR: {t}"),
    };

    let pool = ThreadPool::new(8);
    
    for stream in listener.incoming() {
	let stream = match stream {
	    Ok(t) => t,
	    Err(_t) => continue,
	};

	pool.execute(move || {
	    connection_handler(stream);
	});

	
    }

}
