import asyncio
import websockets
import json

async def get_all_users():
    url = "ws://localhost:9005"  # Убедитесь, что это правильный URL для вашего прокси-сервера

    async with websockets.connect(url, subprotocols=["graphql-ws"]) as websocket:
        print("Соединение установлено.")
        
        # Ожидаем сообщения от сервера о готовности
        while True:
            init_message = {
                "type": "connection_init",
                "payload": {}
            }
            await websocket.send(json.dumps(init_message))
            response = await websocket.recv()
            print("Ответ получен:", response)
            message = json.loads(response)

            # Проверяем, получили ли мы сообщение типа "connection_ack"
            if message.get("type") == "connection_ack":
                print("Соединение успешно инициализировано.")
                break  # Выходим из цикла, когда соединение готово

        # Запрос для получения всех пользователей
        query = {
            "id": "1", "type": "start", "payload": {"query": "{ user { id name surname } }","variables": {}}
        }

        await websocket.send(json.dumps(query))
        print("Запрос отправлен:", query)

        while True:
            try:
                response = await websocket.recv()
                print("Ответ получен:", response)
                
                message = json.loads(response)
                
                if message.get("type") == "ka":
                    continue  # Игнорируем keep-alive сообщения
                
                if message.get("type") == "data":
                    users_data = message.get("payload", {}).get("data", {}).get("user", [])
                    print("Полученные данные пользователей:", users_data)
                
                if message.get("type") == "error":
                    print("Ошибка:", message.get("payload", {}).get("message", "Неизвестная ошибка"))

            except websockets.exceptions.ConnectionClosed:
                print("Соединение закрыто.")
                break


#{"id": "1", "type": "start", "payload": {"query": "{ user { id name surname } }","variables": {}}}


#{"id": "1", "type": "start","payload": {"query": "mutation AddUser($name: String!, $surname: String!) {insert_user(objects: { name: $name, surname: $surname }) {returning {id name surname}}}", "variables": {"name": "Anton", "surname": "Pavlov"}}}


# Запускаем асинхронную функцию
if __name__ == "__main__":
    asyncio.run(get_all_users())
