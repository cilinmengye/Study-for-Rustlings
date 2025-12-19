use tokio::sync::{mpsc, oneshot};
use tokio::io::{self, AsyncBufReadExt, BufReader};
use std::io::Write;
use bytes::Bytes;
// 使用 mini-redis 库
use mini_redis::client;

type OneshotType = oneshot::Sender<String>; 

enum Command {
    Get {
        key: String,
        resp: Option<OneshotType>, // 给任务管理器向任务发送响应的句柄 
    },
    Set {
        key: String,
        value: Bytes,
        resp: Option<OneshotType>, // 给任务管理器向任务发送响应的句柄 
    },
    Unknow,
}

impl Command {
    fn parse(command: &str) -> Self {
        // 将字符串按照空格分割, 没有创建新的 String：只创建了新的 &str 切片
        let parts: Vec<&str> = command.split_whitespace().collect();
        match parts.get(0) {
            Some(&"get") => {
                if let Some(key) = parts.get(1) {
                    Command::Get {
                        key: key.to_string(),
                        resp: None
                    }
                } else {
                    Command::Unknow
                }
            },
            Some(&"set") => {
                if let (Some(key), Some(value)) = (parts.get(1), parts.get(2)) {
                    Command::Set{
                        key: key.to_string(),
                        // as_bytes()返回的切片中的每个元素是一个字节（8位无符号整数）。
                        value: Bytes::copy_from_slice(value.as_bytes()),
                        resp: None
                    }
                } else {
                    Command::Unknow
                }
            }
            _ => {
                Command::Unknow
            }
        }
    }

    fn set_resp(&mut self, tx: OneshotType) {
        match self {
            Command::Get { resp, .. } => {
                *resp = Some(tx);
            }
            Command::Set { resp, .. } => {
                *resp = Some(tx);
            }
            Command::Unknow => {}
        }
    }
}

async fn cli_command_manager(mut rx: mpsc::Receiver<Command>) {
    let mut client = client::connect("127.0.0.1:6379").await.expect("连接 Redis 失败");

    while let Some(cmd) = rx.recv().await {
        match cmd {
            Command::Get { key, resp } => {
                let res = client.get(&key).await;
                // mini-redis get 返回 Result<Option<Bytes>>
                let str = match res {
                    Ok(Some(value)) => {
                        match String::from_utf8(value.to_vec()) {
                            Ok(s) => format!("The query results is {s}"),
                            Err(e) => format!("The query have error: {e}")
                        }
                    },
                    Ok(None) => format!("(nil) - Key not found"),
                    Err(e) => format!("The query have error: {e}")
                };
                let _ = resp.unwrap().send(str);    // unwrap原因是resp是Option<T>
            }
            Command::Set { key, value, resp } => {
                let res = client.set(&key, value).await;
                // mini-redis set 返回的 Result<()>
                let str = match res {
                    Ok(_) => format!{"OK - Success write"},
                    Err(e) => format!{"Err - False Write: {e}"}
                };
                let _ = resp.unwrap().send(str);
            },
            Command::Unknow => panic!("Impossible")
        }
    }
}

// tokio::spawn 并不是一个独立的魔法函数，它必须运行在“运行时（Runtime）”的环境中。 
// 如果不把 main 改成异步并标记 #[tokio::main]，程序就没有“发动机”来驱动这些异步任务。
#[tokio::main]
async fn main() {
    // 创建和任务管理器的通信
    let (cmd_tx, cmd_rx) = mpsc::channel::<Command>(32);
    // 创建任务管理器
    tokio::spawn(async move {
        cli_command_manager(cmd_rx).await;
    });

    // UI 主循环
    // 创建一个通道：用于后台任务向主线程回传响应消息
    let(print_tx, mut print_rx) = mpsc::channel::<String>(32);

    let stdin = io::stdin();
    // 将输入包装成异步每次读取一行
    let mut reader = BufReader::new(stdin).lines();

    print!(">>> ");
    // 手动刷新 stdout，否则提示符可能留在缓冲区不显示
    std::io::stdout().flush().unwrap();
    loop {
        // 同时监听多个异步事件, 每次只对其中一个作出处理(输出到终端，这样可以保证终端的输出不会出现混乱)
        //  1. 用户输入了一行
        //  2. 后台任务返回响应
        tokio::select! {
            // 这里本质上为 let line = reader.next_lines().await(), 其中await()返回Option<T>, next_lines()返回Result<T>
            line = reader.next_line() => {
                let input = line.unwrap().unwrap();
                let command = input.trim();
                if command == "exit" { break; }

                let mut cmd = Command::parse(command);
                match cmd {
                    Command::Unknow => {
                        print_tx.send(format!("Command is not correct")).await.unwrap();
                        continue;
                    },
                    _ => {}
                }
                let clone_cmd_tx = cmd_tx.clone();
                let clone_print_tx = print_tx.clone();
                tokio::spawn(async move {
                    let (resp_tx, resp_rx) = oneshot::channel();
                    cmd.set_resp(resp_tx);
                    // 向任务管理器发送任务
                    // .await的结果是Result<(), SendError>;
                    clone_cmd_tx.send(cmd).await.unwrap();
                    let res = resp_rx.await.unwrap();
                    // 将结果发送给主线程
                    clone_print_tx.send(res).await.unwrap();
                });
            }
            // 监听后台任务的响应消息
            Some(msg) = print_rx.recv() => {
                println!("[通知] {msg}");
                print!(">>> ");
                // 手动刷新 stdout，否则提示符可能留在缓冲区不显示
                std::io::stdout().flush().unwrap();
            }
        }
    }
}