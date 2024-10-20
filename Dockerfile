FROM amazoncorretto:23-jdk AS codegen

WORKDIR /app
COPY openapi.yaml ./

RUN curl -L https://repo1.maven.org/maven2/org/openapitools/openapi-generator-cli/7.8.0/openapi-generator-cli-7.8.0.jar -o openapi-generator-cli.jar
RUN java -jar openapi-generator-cli.jar generate -i openapi.yaml -g rust-axum -o openapi_gen

FROM rust:1.81.0 AS build

RUN apt update && apt install -y mold

WORKDIR /app

COPY Cargo.toml Cargo.lock openapi.yaml ./
COPY src ./src
COPY --from=codegen /app/openapi_gen ./openapi_gen

RUN mold --run cargo build --release

FROM gcr.io/distroless/cc

COPY --from=build /app/target/release/nahlun-server .

CMD ["./nahlun-server"]

