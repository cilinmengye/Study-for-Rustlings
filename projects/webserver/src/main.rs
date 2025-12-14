use std::{
    fs,
    // 将 std::io::prelude 和 std::io::BufReader 引入作用域，来获取读写流所需的 trait 和类型。
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};
use webserver::ThreadPool;

fn main() {
    // **监听 TCP 连接**
    // 监听本地地址 127.0.0.1:7878
    // bind 函数类似于 new 函数，在这里它返回一个新的 TcpListener 实例。
    // bind 函数返回 Result<T, E>，这表明绑定可能会失败。
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    // TcpListener 的 incoming 方法返回一个迭代器，它提供了一系列的流（更准确的说是 TcpStream 类型的流）。
    // 流（stream）代表一个客户端和服务端之间打开的连接。连接（connection）代表客户端连接服务端、服务端生成
    // 响应以及服务端关闭连接的整个请求 / 响应过程。为此，我们会从 TcpStream 读取客户端发送了什么并接着向流
    // 发送响应以向客户端发回数据。incoming() 在没有连接时，会“阻塞当前线程”。
    
    // 测试退出功能所以使用take(10)，此将执行10次迭代后不再执行
    for stream in listener.incoming().take(10) {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream)
{   // **读取请求**
    // 新建了一个 BufReader 实例来封装一个 stream 的引用。
    // BufReader 通过替我们管理 std::io::Read trait 方法的调用增加了缓冲。
    let buf_reader = BufReader::new(&stream);
    // http_request 变量来收集浏览器发送给服务端的请求行。
    // BufReader 实现了 std::io::BufRead trait
    // let http_request: Vec<_> = buf_reader
    //     .lines()    // 它提供了 lines 方法。lines 方法通过遇到换行符（newline）字节就切分数据流来返回一个 Result<String, std::io::Error> 的迭代器。
    //     .map(|result| result.unwrap())
    //     .take_while(|line| !line.is_empty())    // 从迭代器开头开始，连续地取元素，只要闭包返回 true 就继续；一旦第一次返回 false，迭代立刻终止，并且永不再继续。
    //     .collect();
    // println!("Request: {http_request:#?}");
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    // **编写响应** 响应模板如下：
    // HTTP-Version Status-Code Reason-Phrase CRLF; CRLF 序列将请求行与其余请求数据分开。
    // headers CRLF
    // 空行
    // message-body
    let (status_line, filename) = match request_line.as_str() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => { 
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        },
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    // as_bytes() 返回该 String 内部 UTF-8 编码数据的只读字节切片（&[u8]）。
    // stream 的 write_all 方法获取一个 &[u8] 并直接将这些字节发送给连接。
    //  write_all 操作可能会失败，所以像之前那样对任何错误结果使用 unwrap。
    stream.write_all(response.as_bytes()).unwrap();
}