import asyncio
from databases import Database
from sqlalchemy import create_engine, MetaData, Table, Column, String, Integer


DATABASE_URL = "postgresql://root:root@localhost:5432/test_bd"


database = Database(DATABASE_URL)
metadata = MetaData()
engine = create_engine(DATABASE_URL)


user = Table(
    'user',
    metadata,
    Column('id', Integer, primary_key=True, autoincrement=True, unique=True),
    Column('name', String),
    Column('surname', String),
)


metadata.create_all(bind=engine)


users_list = [
    {'name': 'Andrey', 'surname': 'Groulanov'},
    {'name': 'Ivan', 'surname': 'Ivanov'},
]


async def run():
    await database.connect()
    insert_query = user.insert().values(users_list)
    await database.execute(insert_query)
    await database.disconnect()


if __name__ == "__main__":
    
    asyncio.run(run())
