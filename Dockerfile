# Linux image we are going to use
FROM alpine

# Install git
RUN apk update && apk upgrade && \
    apk add --no-cache bash git openssh

# Install JDK
RUN apk add openjdk8
ENV JAVA_HOME /usr/lib/jvm/java-1.8-openjdk
ENV PATH $PATH:$JAVA_HOME/bin

# Clone repository and run our example
RUN git clone https://github.com/geocarvalho/ssdp_plus.git

ENTRYPOINT java -jar ssdp_plus/out/artifacts/ssdp_plus_jar/ssdp_plus.jar ssdp_plus/SSDPplus/pastas/bases/alon-clean50-pn-width-2.CSV 5 0.9 p
