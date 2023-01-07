FROM golang:1.19 as builder

COPY . /app
WORKDIR /app

RUN go build -o tribble .

FROM busybox

COPY static /static
COPY templates /templates
COPY --from=builder /app/tribble /

CMD ["/tribble"]
