FROM scratch

COPY /client/ /client/
COPY /target/x86_64-unknown-linux-musl/release/blight .

EXPOSE 5400
ENTRYPOINT ["./blight", "-s", "[::]:5400", "-f", "./client/", "-b", "/40409/f/", "-w", "/40409/l/", "-x", "/40409/y/", "-p", "-u"]