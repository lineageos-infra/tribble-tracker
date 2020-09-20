import click

from config import Config as config
from models import sql

@click.group()
def cli():
    pass

if __name__ == "__main__":
    cli()
