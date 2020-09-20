FROM python:3.6

WORKDIR /app
COPY requirements.txt /app

ENV MONGODB_DB "stats"
ENV MONGODB_USERNAME ""
ENV MONGODB_PASSWORD ""
ENV MONGODB_HOST "mongo"

ENV prometheus_multiproc_dir /app/metrics

RUN pip install -r /app/requirements.txt

COPY . /app

CMD gunicorn app:app -b 0.0.0.0:8080 -w 9
