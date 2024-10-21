# gateway

Для запуска необходимо 
1) подготовить бд
1.1 Сборка докер компоса docker-compose build после docker-compose up
2) Создание и подготовка таблицы в  бд
2.1 cd ./db
2.1 python -m venv venv
2.1 venv/Sctipt/activate
2.2 pip install -r requirements.txt
2.3 создаение таблицы и заполнение ее: python create_db.py
2.4 Можно проверить соединение: python ws_client.py

