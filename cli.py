import click
import mongoengine

from config import Config as config
from models import mongo, sql

@click.group()
def cli():
    pass

@click.command()
def migrate_stats():
    for stat in mongo.Statistic.objects:
        session = sql.Session()
        s = sql.Statistic(
            device_id=stat.d,
            model=stat.m,
            version=stat.v,
            country=stat.u,
            carrier=stat.c,
            carrier_id=stat.c_id,
            submit_time=stat.t)
        session.add(s)
        session.commit()
        click.echo(stat)
    click.echo("foo")

cli.add_command(migrate_stats)

if __name__ == "__main__":
    db = mongoengine.connect('stats')
    cli()
