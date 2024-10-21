use tokio::task::JoinHandle;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{accept_async, connect_async};
use futures_util::{StreamExt, SinkExt};
use tokio::net::TcpStream;





async fn proxy_server(server_addr: &str, service_url:&str) {
    let listener = TcpListener::bind(&server_addr).await.expect("err 7 Не удалось создать сервер");
    while let Ok((stream, _)) = listener.accept().await {
        _ = trafick_manager(stream, service_url.to_string()).await;
    }
}

async fn trafick_manager(stream: TcpStream, service_url: String) -> JoinHandle<()> {
    //let server_url = "ws://127.0.0.1:9001";
    let process_control = tokio::spawn(async move {
        let client_ws_stream = accept_async(stream)
            .await
            .expect("err 8 Ошибка при установлении WebSocket соединения c склиентом");
        println!("Новое WebSocket соединение с клиентом установлено");
        let mut request = service_url.into_client_request().expect("Invalid URL");
        request.headers_mut().insert("Sec-WebSocket-Protocol", "graphql-ws".parse().unwrap());
        
        let (mut server_ws_stream, _) = connect_async(request).await.expect("Ошибка при подключении к Hasura");
        let (mut client_write, mut client_read) = client_ws_stream.split();

        //let (mut server_ws_stream, _) = connect_async(&service_url).await.expect("err 9 Ошибка при подключении к hasura");

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
                                        println!("err 10 Ошибка отправки на Hasura: {}", e);
                                    }
                                    break;
                                }
                                _ => {
                                    if let Err(e) = server_write.send(msg).await {
                                        println!("err 11 Ошибка отправки на Hasura: {}", e);
                                        break;
                                    }
                                }
                            }
                            
                        }
                        Err(e) => {
                            println!("err 12 Ошибка получения от клиента: {}", e);
                            break;
                        }
                    }
                }
                Some(srvmsg) = server_read.next() => {
                    match srvmsg {
                        Ok(msg) => {
                            match msg {
                                Message::Close(_) => {
                                    if let Err(e) = client_write.send(msg).await {
                                        println!("err 13 Ошибка отправки клиенту: {}", e);
                                    }
                                    break;
                                }
                                _ => {
                                    if let Err(e) = client_write.send(msg).await {
                                        println!("err 14 Ошибка отправки клиенту: {}", e);
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            println!("err 15 Ошибка получения от Нasura: {}", e);
                            break;
                        }
                    }
                }
            }
        }
    });
    process_control
    
}


#[tokio::main]
async fn main() {
    let service_url = "ws://localhost:8080/v1/graphql";
    let server_addr = "127.0.0.1:9005";
    
    
    let _ = proxy_server(server_addr, service_url).await;
    
    

    
}


#[cfg(test)]
mod test {
    use futures::stream::SplitStream;
    use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

    use super::*;

    async fn test_sender_to_ws(messages: Vec<Message>, url: &str) -> SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>> {
        // Подключаемся к WebSocket серверу по заданному URL
        let (mut server_ws_stream, _) = connect_async(url).await.expect("err 5 Ошибка при подключении к серверу");
        let (mut server_write, server_read) = server_ws_stream.split();

        // Отправляем все тестовые сообщения на сервер
        for msg in messages {
            server_write.send(msg).await.expect("err 6 Не удалось отправить тестовое сообщение");
        }

        return server_read; // Возвращаем поток для чтения ответных сообщений от сервера
    }

    async fn test_echo_server() {
        let addr = "127.0.0.1:9001";
        let listener = TcpListener::bind(&addr).await.expect("err 1 Не удалось создать сервер");
        println!("Тестовый сервер запущен на ws://{}", addr);
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let ws_stream = accept_async(stream)
                    .await
                    .expect("err 2 Ошибка при установлении WebSocket соединения");
                println!("Новое WebSocket соединение с эхо сервером установлено");
    
                let (mut write, mut read) = ws_stream.split();
    
                while let Some(message) = read.next().await {
                    match message {
                        Ok(msg) => {
                            println!("cообщение получено");
                            if msg.is_text() || msg.is_binary() {
                                write.send(msg).await.expect("err 3 Ошибка при отправке сообщения");
                            } else if msg.is_close() {
                                println!("Соединение закрыто");
                                break;
                            }
                        }
                        Err(e) => {
                            println!("err 4 Ошибка получения сообщения: {}", e);
                            break;
                        }
                    }
                }
            });
        }
    }

    #[tokio::test]
    async fn main_test() {
        let service_url = "ws://127.0.0.1:9001";
        let server_addr = "127.0.0.1:9005";
        let url = "ws://127.0.0.1:9005";
        
        let message_text: Vec<Message> = vec![Message::text("привет"), Message::text("привет2")];

        let _echo_server = tokio::spawn(async {
            test_echo_server().await;
        });

      
        let _proxy_server = tokio::spawn(async {
            proxy_server(server_addr, service_url).await;
        });

        
        let mut answer_buffer = test_sender_to_ws(message_text.clone(), &url).await;

        
        let mut received_messages = Vec::new();
        while let Some(msg) = answer_buffer.next().await {
            match msg {
                Ok(message) => {
                    if message.is_text() {
                        received_messages.push(message);
                    }
                }
                Err(e) => {
                    println!("Ошибка при получении сообщения: {}", e);
                }
            }

            if received_messages.len() == message_text.len() {
                break; 
            }
        }

        println!("{:?}", received_messages);
        assert_eq!(
            received_messages.into_iter().map(|msg| msg.into_text().unwrap()).collect::<Vec<_>>(),
            message_text.into_iter().map(|msg| msg.into_text().unwrap()).collect::<Vec<_>>()
        );

       
        
    }
        
}