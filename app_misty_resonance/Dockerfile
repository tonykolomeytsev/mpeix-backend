FROM python:3.9-slim AS base
ENV PYTHONDONTWRITEBYTECODE 1
ENV PYTHONFAULTHANDLER 1

FROM base AS python-deps
RUN pip install pipenv && \
    apt update && \
    apt install -y --no-install-recommends gcc wget
RUN wget -O /tmp/torch-2.0.1+cpu-cp39-cp39-linux_x86_64.whl https://download.pytorch.org/whl/cpu/torch-2.0.1%2Bcpu-cp39-cp39-linux_x86_64.whl
COPY Pipfile .
ENV PIPENV_VENV_IN_PROJECT 1
RUN pipenv install --deploy

FROM gcr.io/distroless/python3-debian11
WORKDIR /app
COPY --from=python-deps /.venv/lib/python3.9/site-packages /app/site-packages
ENV PYTHONPATH /app/site-packages
COPY . .
ENTRYPOINT [ "python", "src/main.py" ]
