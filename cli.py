import logging

import click

from config import Config as config
from models import sql

@click.group()
def cli():
    pass

@click.command()
def expire():
    logging.basicConfig()
    logging.getLogger('sqlalchemy.engine').setLevel(logging.INFO)
    sql.Statistic.drop_old()

cli.add_command(expire)
if __name__ == "__main__":
    cli()
