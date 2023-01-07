FROM go:1.19 as builder

COPY . /app
WORKDIR /app

RUN go build . -o tribble

FROM scratch

COPY static /static
COPY templates /templates
COPY tribble /

RUN /tribble
