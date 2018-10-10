import os
from sqlalchemy import Column, Integer, String, DateTime, create_engine, distinct, func
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.sql import func, text
from sqlalchemy.orm import sessionmaker

from sqlalchemy.types import Integer
from sqlalchemy.dialects import postgresql

SQL_CONNECT_STRING = os.environ.get("SQL_CONNECT_STRING", "sqlite:///local.db")

if SQL_CONNECT_STRING.startswith("sqlite://"):
    engine = create_engine(SQL_CONNECT_STRING, echo=True)
else:
    engine = create_engine(SQL_CONNECT_STRING, pool_size=25, max_overflow=25)

Session = sessionmaker(bind=engine)

Base = declarative_base()

PrimaryKey = Integer()
PrimaryKey = PrimaryKey.with_variant(postgresql.BIGINT, "postgresql")

class Statistic(Base):
    '''Main data table

    TODO(zifnab): indexes + migrations
    '''
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
    def create(cls, data):
        session = Session()
        session.add(cls(
            device_id=data['device_hash'],
            model=data['device_name'],
            version=data['device_version'],
            country=data['device_country'],
            carrier=data['device_carrier'],
            carrier_id=data['device_carrier_id'],
        ))
        session.commit()
        session.close()


class Aggregate(Base):
    '''THIS NEEDS TO BE A MATERIALIZED VIEW
       After intiial creation, you need to run the following in postgres:

       drop table aggregate;
       create materialized view aggregate as (select distinct on (device_id) * from (select * from stats where submit_time > localtimestamp - interval '3' month order by statistic_id desc limit 35000000) as foo);

    You'll then want to restart the service. Materalized views are cached, on some regular basis you'll need to run:

    refresh materialized view aggregate;

    This process may take a long time. Try ajusting the limit above to a sane value (35mil was picked by running something like select * from stats where submit_time > localtimestamp - interval '3' month and statistic_id = (max id) - 35000000 order by statistic_id desc limit 1)

    TODO(zifnab): automate this
    '''
    __tablename__ = "aggregate"
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
            return session.query(getattr(cls, field), func.count(cls.device_id).label('count')).group_by(getattr(cls, field)).order_by('count desc')
        session.close()

    @classmethod
    def get_count(cls):
        session = Session()
        data = {
            "total": session.query(func.count(cls.device_id)).first()[0],
        }
        if session.bind.dialect.name == "postgresql":
            data["official"] = session.query(func.count(cls.device_id)).filter(text("version ~ '\d\d\.\d-\d{8}-NIGHTLY-[a-zA-Z]*'")).first()[0],
        session.close()
        return data

    @classmethod
    def get_count_by_field(cls, field, value):
        session = Session()
        data = {
            "total": session.query(func.count(cls.device_id)).filter_by(**{field: value}).first()[0]
        }
        if session.bind.dialect.name == "postgresql":
            data["official"] = session.query(func.count(cls.device_id)).filter_by(**{field: value}).filter(text("version ~ '\d\d\.\d-\d{8}-NIGHTLY-[a-zA-Z]*'")).first()[0],
        session.close()
        return data

Base.metadata.create_all(engine)

