import json
import os
from datetime import datetime
from time import time

import falcon
import jinja2

from falcon.media.validators import jsonschema

from models import sql
from middleware.prometheus import PrometheusMetricsResource, PrometheusComponent


j2env = jinja2.Environment(
    loader=jinja2.FileSystemLoader("templates"),
    autoescape=jinja2.select_autoescape(["html", "xml"]),
)

# These things are either misconfigured to not send a static device_id
# or they're maliciously inflating their values. As such, we reject stats
# coming from them.
BLACKLIST = {
    "device_version": {
        "13.0-20180304-UNOFFICIAL-ht16": True,
        "15.1-20201008-UNOFFICIAL-sagit": True,
        "15.1-20200901-UNOFFICIAL-sagit": True,
        "15.1-20200708-UNOFFICIAL-sagit": True,
        "15.1-20200619-UNOFFICIAL-sagit": True,
        "17.1-20210813-UNOFFICIAL-tissot": True,
        "18.1-20210914-UNOFFICIAL-tissot": True,
        "17.1-20200715-UNOFFICIAL-tissot": True,
    }
}


def load_template(name):
    return j2env.get_template(name)


def normalize_country(country):
    if len(country) != 2:
        return "Unknown"
    return country.upper()


class StatsApiResource(object):

    schema = {
        "title": "Stats Object",
        "type": "object",
        "properties": {
            "device_hash": {"type": "string"},
            "device_name": {"type": "string"},
            "device_version": {"type": "string"},
            "device_country": {"type": "string"},
            "device_carrier": {"type": "string"},
            "device_carrier_id": {"type": "string"},
        },
        "required": [
            "device_hash",
            "device_name",
            "device_version",
            "device_country",
            "device_carrier",
            "device_carrier_id",
        ],
    }

    @jsonschema.validate(schema)
    def on_post(self, req, resp):
        """Handles post requests to /api/v1/stats"""
        data = req.media
        if not BLACKLIST["device_version"].get(data["device_version"], False):
            data["device_country"] = normalize_country(data["device_country"])
            sql.Statistic.create(data)
        resp.body = "neat"
        resp.content_type = "text/plain"

    def on_get(self, req, resp):
        """Handles get requests to /api/v1/stats"""
        stats = {
            "model": {x[0]: x[1] for x in sql.Statistic.get_most_popular("model", 90)},
            "country": {
                x[0]: x[1] for x in sql.Statistic.get_most_popular("country", 90)
            },
            "version": {
                x[0]: x[1] for x in sql.Statistic.get_most_popular("version", 90)
            },
            "total": sql.Statistic.get_count(90).first()[0],
        }
        resp.body = json.dumps(stats)


class StaticResource(object):
    def on_get(self, req, resp, kind, filename):
        if kind == "css":
            resp.content_type = "text/css"
        resp.stream = open(
            os.path.abspath(os.path.join("static", kind, filename)), "rb"
        )


class IndexResource(object):
    def on_get(self, req, resp):
        """Render the main page"""
        stats = {
            "model": sql.Statistic.get_most_popular("model", 90),
            "country": sql.Statistic.get_most_popular("country", 90),
            "total": sql.Statistic.get_count(90).first()[0],
        }
        template = load_template("index.html").render(
            stats=stats,
            columns=["model", "country"],
            date=datetime.utcnow().strftime("%Y-%m-%d %H:%M"),
        )
        resp.content_type = "text/html"
        resp.body = template


class RobotsResource(object):
    def on_get(self, req, resp):
        resp.status = falcon.HTTP_200
        resp.content_type = "text/plain"
        resp.body = "User-agent: *\nDisallow: /"
        return


class FieldResource(object):
    def on_get(self, req, resp, field, value):
        if field not in ["model", "carrier", "version", "country"]:
            resp.status = falcon.HTTP_404
            resp.content_type = "text/plain"
            resp.body = "Not Found"
            return
        valuemap = {
            "model": ["version", "country"],
            "carrier": ["model", "country"],
            "version": ["model", "country"],
            "country": ["model", "carrier"],
        }
        left, right = valuemap[field]
        stats = {
            left: sql.Statistic.get_most_popular(left, 90).filter_by(**{field: value}),
            right: sql.Statistic.get_most_popular(right, 90).filter_by(
                **{field: value}
            ),
            "total": sql.Statistic.get_count(90).filter_by(**{field: value}).first()[0],
        }
        template = load_template("index.html").render(
            stats=stats,
            columns=valuemap[field],
            value=value,
            date=datetime.utcnow().strftime("%Y-%m-%d %H:%M"),
        )
        resp.content_type = "text/html"
        resp.body = template


app = falcon.API(middleware=[PrometheusComponent()])
app.add_route("/", IndexResource())
app.add_route("/robots.txt", RobotsResource())
app.add_route("/{field}/{value}", FieldResource())
app.add_route("/static/{kind}/{filename}", StaticResource())
app.add_route("/api/v1/stats", StatsApiResource())
app.add_route("/metrics", PrometheusMetricsResource())
