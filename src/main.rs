use futures::future::ok;
use futures::stream::SplitSink;
use tokio::task::JoinHandle;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{accept_async, connect_async};
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio::net::TcpStream;

const INPUT: &str = "127.0.0.1:9005";
const OUTPUT: &str = "127.0.0.1:9001";

async fn test_echo_server() {
    let addr = "127.0.0.1:9001";
    let listener = TcpListener::bind(&addr).await.expect("Не удалось создать сервер");
    println!("Тестовый сервер запущен на ws://{}", addr);
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            let ws_stream = accept_async(stream)
                .await
                .expect("Ошибка при установлении WebSocket соединения");
            println!("Новое WebSocket соединение с эхо сервером установлено");

            let (mut write, mut read) = ws_stream.split();

            while let Some(message) = read.next().await {
                match message {
                    Ok(msg) => {
                        println!("cообщение получено");
                        if msg.is_text() || msg.is_binary() {
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

async fn test_sender_to_ws(messages: Vec<Message>, url: &str) {
    let (mut server_ws_stream, _) = connect_async(url).await.expect("Ошибка при подключении к hasura");
    let (mut server_write, mut server_read) = server_ws_stream.split();
    for msg in messages {
        server_write.send(msg).await.expect("Не удалось отправть тестовое сообщение");
    }
}

async fn server() {
    let addr = INPUT;
    let listener = TcpListener::bind(&addr).await.expect("Не удалось создать сервер");
    while let Ok((stream, _)) = listener.accept().await {
        trafick_manager(stream).await;
    }
}

async fn trafick_manager(stream: TcpStream) {
    let server_url = "ws://127.0.0.1:9001";
    let process_control = tokio::spawn(async move {
        let client_ws_stream = accept_async(stream)
            .await
            .expect("Ошибка при установлении WebSocket соединения c склиентом");
        println!("Новое WebSocket соединение с клиентом установлено");

        let (mut client_write, mut client_read) = client_ws_stream.split();

        let (mut server_ws_stream, _) = connect_async(server_url).await.expect("Ошибка при подключении к hasura");

        let (mut server_write, mut server_read) = server_ws_stream.split();

        loop {
            tokio::select! {
                // Чтение сообщения от клиента
                Some(clmsg) = client_read.next() => {
                    match clmsg {
                        Ok(msg) => {
                            match msg {
                                //логика отправки сообщений в Hasura, по умолчанию отправляем все, дополнительно обрабатываем только закрытие
                                Message::Close( _) => {
                                    if let Err(e) = server_write.send(msg).await {
                                        println!("1 Ошибка отправки на Hasura: {}", e);
                                    }
                                    break;
                                }
                                _ => {
                                    if let Err(e) = server_write.send(msg).await {
                                        println!("2 Ошибка отправки на Hasura: {}", e);
                                        break;
                                    }
                                }
                            }
                            
                            
                        }
                        Err(e) => {
                            println!("3 Ошибка получения от клиента: {}", e);
                            break;
                        }
                    }
                }
                Some(srvmsg) = server_read.next() => {
                    match srvmsg {
                        Ok(msg) => {
                            if let Err(e) = client_write.send(msg).await {
                                println!("4 Ошибка отправки на клиент: {}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            println!(" 5Ошибка получения от Нasura: {}", e);
                            break;
                        }
                    }
                }
            }
        }
    });
    
}


#[tokio::main]
async fn main() {
    let test = tokio::spawn(async {
        test_echo_server().await;
    });
    let _ = server().await;
    
    

    if let Err(e) = test.await {
        println!("Ошибка при выполнении задачи: {}", e);
    }
}
