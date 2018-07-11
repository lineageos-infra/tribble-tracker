import os
from sqlalchemy import Column, Integer, String, DateTime, create_engine, distinct, func
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.sql import func
from sqlalchemy.orm import sessionmaker

from sqlalchemy.types import Integer
from sqlalchemy.dialects import postgresql

engine = create_engine(os.environ.get("SQL_CONNECT_STRING", "sqlite:///local.db"))
Session = sessionmaker(bind=engine)

Base = declarative_base()

PrimaryKey = Integer()
PrimaryKey = PrimaryKey.with_variant(postgresql.BIGINT, "postgresql")

class Statistic(Base):
    __tablename__ = "stats"
    statistic_id = Column(PrimaryKey, primary_key=True, autoincrement=True)
    device_id = Column(String)
    model = Column(String)
    version = Column(String)
    country = Column(String)
    carrier = Column(String)
    carrier_id = Column(String)
    submit_time = Column(DateTime, server_default=func.now())

    @classmethod
    def get_most_popular(cls, field, days):
        session = Session()
        if hasattr(cls, field):
            return session.query(getattr(cls, field), func.count(distinct(cls.device_id)).label('count')).group_by(getattr(cls, field)).order_by('count desc')
    @classmethod
    def get_count(cls, days=90):
        session = Session()
        return session.query(func.count(distinct(cls.device_id)))

Base.metadata.create_all(engine)
