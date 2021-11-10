import datetime
import os

from contextlib import contextmanager

from sqlalchemy import Column, String, DateTime, create_engine, Computed
from sqlalchemy.ext.declarative import declarative_base
from sqlalchemy.sql import func
from sqlalchemy.orm import sessionmaker

from sqlalchemy.sql.expression import desc

SQL_CONNECT_STRING = os.environ.get("SQL_CONNECT_STRING", "sqlite:///local.db")

if SQL_CONNECT_STRING.startswith("sqlite://"):
    engine = create_engine(SQL_CONNECT_STRING)
else:
    engine = create_engine(SQL_CONNECT_STRING, pool_size=25, max_overflow=25)

Session = sessionmaker(bind=engine)

Base = declarative_base()


@contextmanager
def session_scope():
    session = Session()
    try:
        yield session
        session.commit()
    except Exception:
        session.rollback()
        raise
    finally:
        session.close()


class Statistic(Base):
    """Main data table

    TODO(zifnab): indexes + migrations
    """

    __tablename__ = "stats"
    device_id = Column(String, primary_key=True)
    model = Column(String)
    version_raw = Column(String)
    country = Column(String)
    carrier = Column(String)
    carrier_id = Column(String)
    submit_time = Column(DateTime, server_default=func.now())
    if SQL_CONNECT_STRING.startswith("sqlite://"):
        computed = "substr(version_raw, 0, 3)"
    else:
        computed = "substring(version_raw, '^\\d\\d\\.\\d')"
    version = Column(String, Computed(computed))

    @classmethod
    def create(cls, data):
        with session_scope() as session:
            session.merge(
                cls(
                    device_id=data["device_hash"],
                    model=data["device_name"],
                    version_raw=data["device_version"],
                    country=data["device_country"],
                    carrier=data["device_carrier"],
                    carrier_id=data["device_carrier_id"],
                )
            )

    @classmethod
    def get_most_popular(cls, field, days):
        with session_scope() as session:
            if hasattr(cls, field):
                return (
                    session.query(
                        getattr(cls, field), func.count(cls.device_id).label("count")
                    )
                    .group_by(getattr(cls, field))
                    .order_by(desc("count"))
                )

    @classmethod
    def get_count(cls, days=90):
        with session_scope() as session:
            return session.query(func.count(cls.device_id))

    @classmethod
    def drop_old(cls, days=90):
        with session_scope() as session:
            limit = datetime.datetime.now() - datetime.timedelta(days=days)
            session.query(cls).filter(cls.submit_time <= limit).delete()


Base.metadata.create_all(engine)
