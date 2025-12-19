use tokio::net::{TcpListener, TcpStream};
use mini_redis::{Connection, Frame};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use bytes::Bytes;

type Db = Arc<Mutex<HashMap<String, Bytes>>>;

fn handle_command(parts: &[Frame], db: &Db) -> Frame {
    let cmd_name = match &parts[0] {
        Frame::Bulk(data) => std::str::from_utf8(data).unwrap().to_lowercase(),
        _ => return Frame::Error("no match command".into()),
    };

    match cmd_name.as_str() {
        "get" => {
            if let Frame::Bulk(key) = &parts[1] {
                let db = db.lock().unwrap();
                let key_str = std::str::from_utf8(key).unwrap();
                if let Some(value) = db.get(key_str) {
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null // Redis 中的 nil
                }
            } else {
                Frame::Error("Get command lost key".into())
            }
        }
        "set" => {
            if let (Frame::Bulk(key), Frame::Bulk(value)) = (&parts[1], &parts[2]) {
                let mut db = db.lock().unwrap();
                let key_str = std::str::from_utf8(key).unwrap();
                db.insert(key_str.to_string(), value.clone());
                Frame::Simple("OK".into())
            } else {
                Frame::Error("SET command lost key or value".into())
            }
        }
        &_ => {
            Frame::Error("no match command".into())
        }
    }
}

async fn process(socket: TcpStream, db: Db) {
    // `Connection` 对于 redis 的读写进行了抽象封装，因此我们读到的是一个一个数据帧frame(数据帧 = redis命令 + 数据)，而不是字节流
    // `Connection` 是在 mini-redis 中定义
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match frame {
            Frame::Array(ref parts) => handle_command(parts, &db),
            _ => Frame::Error("Currently, only Array type instructions are supported.".into()),
        };
        connection.write_frame(&response).await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    // Bind the listener to the address
    // 监听指定地址，等待 TCP 连接进来
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let clone_db = db.clone();
        tokio::spawn(async move {
            process(socket, clone_db).await;
        });
    }
}