FROM golang:1.19 as builder

COPY . /app
WORKDIR /app

RUN go build -buildvcs=false -o tribble .

FROM ubuntu:24.04

RUN apt-get update && apt-get install -y sqlite3

COPY static /static
COPY templates /templates
COPY --from=builder /app/tribble /
COPY schema.sql /

CMD ["/tribble"]
