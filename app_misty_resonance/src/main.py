from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from model import Model
import uvicorn
import os

path_to_weights = os.getenv("WEIGHTS_PATH", "/var/opt/weights.pt")
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


if __name__ == "__main__":
    print("Starting server...")
    uvicorn.run(
        app,
        host="0.0.0.0",
        port=int(os.getenv("PORT", 8080)),
        log_level=os.getenv("LOG_LEVEL", "info"),
        proxy_headers=True,
    )
