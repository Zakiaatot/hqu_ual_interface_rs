FROM ubuntu:jammy
RUN apt-get update && apt-get upgrade -y 
WORKDIR /server
COPY ./bin/hqu_ual_interface_rs .
RUN mv /server/hqu_ual_interface_rs /hqu_ual_interface_rs
ENV RUST_LOG=info
CMD ["/hqu_ual_interface_rs"]
