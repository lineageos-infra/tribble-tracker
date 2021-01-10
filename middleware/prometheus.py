from time import time
from prometheus_client import (
    multiprocess,
    generate_latest,
    CollectorRegistry,
    CONTENT_TYPE_LATEST,
    Counter,
    Histogram,
)

REQUEST_LATENCY = Histogram(
    "falcon_request_latency_seconds", "Request Latency", ["method", "endpoint"]
)
REQUEST_COUNT = Counter(
    "falcon_request_count", "Request Count", ["method", "endpoint", "status"]
)

class PrometheusMetricsResource(object):
    def on_get(self, req, resp):
        registry = CollectorRegistry()
        multiprocess.MultiProcessCollector(registry)
        resp.body = generate_latest(registry)
        resp.content_type = CONTENT_TYPE_LATEST


class PrometheusComponent(object):
    def process_request(self, req, resp):
        req.context["start_time"] = time()

    def process_response(self, req, resp, resource, req_suceeded):
        delta = time() - req.context["start_time"]
        if req.relative_uri in ["/api/v1/stats", "/"]:
            REQUEST_LATENCY.labels(req.method, req.relative_uri).observe(delta)
            REQUEST_COUNT.labels(req.method, req.relative_uri, resp.status).inc()
