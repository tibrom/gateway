use futures::future::ok;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};

const INPUT: &str = "127.0.0.1:9001";
const OUTPUT: &str = "127.0.0.1:9001";



#[tokio::main]
async fn main() {
    
    let addr = "127.0.0.1:9001";
    let listener = TcpListener::bind(&addr).await.expect("Не удалось создать сервер");
    println!("WebSocket сервер запущен на ws://{}", addr);

    // Обрабатываем каждое новое соединение
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            // Устанавливаем асинхронное WebSocket соединение
            let ws_stream = accept_async(stream)
                .await
                .expect("Ошибка при установлении WebSocket соединения");
            println!("Новое WebSocket соединение установлено");

            // Разделяем поток на передающий и принимающий
            let (mut write, mut read) = ws_stream.split();
            loop {
                let message = read.next().await;
                match message {
                    Some(res_msg) => {
                        match res_msg {
                            Ok(msg) => {
                                match msg {
                                    Message::Text(text) => {
                                        println!("Received a text message: {}", text);
                                    }
                                    Message::Binary(data) => {
                                        println!("Received binary data: {:?}", data);
                                    }
                                    Message::Ping(ping_data) => {
                                        println!("Received ping: {:?}", ping_data);
                                    }
                                    Message::Pong(pong_data) => {
                                        println!("Received pong: {:?}", pong_data);
                                    }
                                    Message::Close(close_frame) => {
                                        println!("Received close message: {:?}", close_frame);
                                    }
                                    _ => {
                                        println!("Received an unknown type of message");
                                    }
                                }
                            }
                            Err(msg) => {

                            }
                        }
                    }
                    _ => {

                    }
                }
            }

            // Чтение сообщений
            while let Some(message) = read.next().await {
                match message {
                    Ok(msg) => {
                        if msg.is_text() || msg.is_binary() {
                            // Отправляем полученное сообщение обратно клиенту
                            write.send(msg).await.expect("Ошибка при отправке сообщения");
                        } else if msg.is_close() {
                            println!("Соединение закрыто");
                            break;
                        }
                    }
                    Err(e) => {
                        println!("Ошибка получения сообщения: {}", e);
                        break;
                    }
                }
            }
        });
    }
}
