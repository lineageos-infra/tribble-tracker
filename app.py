import json
import os
import pycountry
from datetime import datetime
from time import time

import falcon
import jinja2

from falcon.media.validators import jsonschema
from prometheus_client import (
    multiprocess,
    generate_latest,
    CollectorRegistry,
    CONTENT_TYPE_LATEST,
    Counter,
    Histogram,
)

from models import sql

j2env = jinja2.Environment(
    loader=jinja2.FileSystemLoader("templates"),
    autoescape=jinja2.select_autoescape(["html", "xml"]),
)

# These things are either misconfigured to not send a static device_id
# or they're maliciously inflating their values. As such, we reject stats
# coming from them.
BLACKLIST = {"device_version": {"13.0-20180304-UNOFFICIAL-ht16": True}}


REQUEST_LATENCY = Histogram(
    "falcon_request_latency_seconds", "Request Latency", ["method", "endpoint"]
)
REQUEST_COUNT = Counter(
    "falcon_request_count", "Request Count", ["method", "endpoint", "status"]
)


class PrometheusComponent(object):
    def process_request(self, req, resp):
        req.context["start_time"] = time()

    def process_response(self, req, resp, resource, req_suceeded):
        delta = time() - req.context["start_time"]
        if req.relative_uri in ["/api/v1/stats", "/"]:
            REQUEST_LATENCY.labels(req.method, req.relative_uri).observe(delta)
            REQUEST_COUNT.labels(req.method, req.relative_uri, resp.status).inc()


class PrometheusMetricsResource(object):
    def on_get(self, req, resp):
        registry = CollectorRegistry()
        multiprocess.MultiProcessCollector(registry)
        resp.body = generate_latest(registry)
        resp.content_type = CONTENT_TYPE_LATEST


def load_template(name):
    return j2env.get_template(name)


def validate_data(data, resp):
    if BLACKLIST["device_version"].get(data["device_version"], False):
        return False
    country = pycountry.countries.get(alpha_2=data["device_country"])
    if country is None:
        data["device_country"] = "Unknown"
    else:
        data["device_country"] = country.alpha_2
    return True


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
        if validate_data(data, resp):
            sql.Statistic.create(data)
            resp.body = "neat"
        else:
            falcon.status = falcon.HTTP_BAD_REQUEST
            resp.body = "Failed"
        resp.content_type = "text/plain"

    def on_get(self, req, resp):
        """Handles get requests to /api/v1/stats"""
        stats = {
            "model": {x[0]: x[1] for x in sql.Statistic.get_most_popular("model", 90)},
            "country": {
                x[0]: x[1] for x in sql.Statistic.get_most_popular("country", 90)
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
app.add_route("/{field}/{value}", FieldResource())
app.add_route("/static/{kind}/{filename}", StaticResource())
app.add_route("/api/v1/stats", StatsApiResource())
app.add_route("/metrics", PrometheusMetricsResource())
