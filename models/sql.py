import os
from sqlalchemy import Column, Integer, String, DateTime, create_engine, distinct, func
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.sql import func, text
from sqlalchemy.orm import sessionmaker

from sqlalchemy.types import Integer
from sqlalchemy.dialects import postgresql
from sqlalchemy.sql.expression import desc

SQL_CONNECT_STRING = os.environ.get("SQL_CONNECT_STRING", "sqlite:///local.db")

if SQL_CONNECT_STRING.startswith("sqlite://"):
    engine = create_engine(SQL_CONNECT_STRING)
else:
    engine = create_engine(SQL_CONNECT_STRING, pool_size=25, max_overflow=25)

Session = sessionmaker(bind=engine)

Base = declarative_base()

class Statistic(Base):
    '''Main data table

    TODO(zifnab): indexes + migrations
    '''
    __tablename__ = "stats"
    device_id = Column(String, primary_key=True)
    model = Column(String)
    version = Column(String)
    country = Column(String)
    carrier = Column(String)
    carrier_id = Column(String)
    submit_time = Column(DateTime, server_default=func.now())

    @classmethod
    def create(cls, data):
        session = Session()
        session.merge(cls(
            device_id=data['device_hash'],
            model=data['device_name'],
            version=data['device_version'],
            country=data['device_country'],
            carrier=data['device_carrier'],
            carrier_id=data['device_carrier_id'],
        ))
        session.commit()
        session.close()

    @classmethod
    def get_most_popular(cls, field, days):
        session = Session()
        if hasattr(cls, field):
            return session.query(getattr(cls, field), func.count(cls.device_id).label('count')).group_by(getattr(cls, field)).order_by(desc('count'))
        session.close()

    @classmethod
    def get_count(cls, days=90):
        session = Session()
        return session.query(func.count(cls.device_id))
        session.close()

Base.metadata.create_all(engine)

