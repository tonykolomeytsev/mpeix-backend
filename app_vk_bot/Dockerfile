ARG APP_NAME="app_vk_bot"

# Build our app here
FROM rust:1.76.0 as build
ARG APP_NAME
ENV PKG_CONFIG_ALLOW_CROSS=1
WORKDIR /usr/src/${APP_NAME}
COPY . .
RUN cargo install --locked --path ${APP_NAME}

# Deploy our app with 'Distroless' image by Google
FROM gcr.io/distroless/cc-debian12
ARG APP_NAME
COPY --from=build /usr/local/cargo/bin/${APP_NAME} /usr/local/bin/${APP_NAME}
CMD [ "app_vk_bot" ]
