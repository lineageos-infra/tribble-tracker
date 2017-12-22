FROM python:latest

COPY . /app
WORKDIR /app

ENV MONGODB_DB "stats"
ENV MONGODB_USERNAME ""
ENV MONGODB_PASSWORD ""
ENV MONGODB_HOST "mongo"

ENV REDIS_URL "redis://redis:6379/4"

ENV FLASK_APP app.py

ENV prometheus_multiproc_dir /app/metrics

RUN pip install -r /app/requirements.txt
RUN pip install gunicorn

CMD gunicorn app:app -b 0.0.0.0:8080 -w 4
