FROM golang:1.19 as builder

COPY . /app
WORKDIR /app

RUN go build -buildvcs=false -o tribble .

FROM busybox

COPY static /static
COPY templates /templates
COPY --from=builder /app/tribble /
COPY schema.sql /

CMD ["/tribble"]
