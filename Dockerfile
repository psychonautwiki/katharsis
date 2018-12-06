FROM psychonaut/rust-nightly:2018-12-05

COPY . /my-source

RUN cd /my-source && cargo build -v --release

CMD ["/my-source/target/release/katharsis"]
