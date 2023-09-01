from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from .model import Model
import os

path_to_weights = os.environ.get("WEIGHTS_PATH", "/var/opt/weights.pt")
model = Model(path_to_weights)

app = FastAPI()


@app.get("/v1/health")
async def read_root() -> str:
    return "I'm alive :)"


class RequestBody(BaseModel):
    text: str


@app.post("/v1/predict")
async def read_item(body: RequestBody) -> dict[str, str]:
    if len(body.text) == 0:
        raise HTTPException(status_code=400, detail="Empty text")
    return model.predict(body.text)
