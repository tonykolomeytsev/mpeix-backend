class Model:
    def __init__(self, path_to_weights="./weights.pt"):
        print(f"Path to weights: {path_to_weights}")

    def predict(self, text: str) -> dict[str, str]:
        print(f"Text: {text}")
        return {
            "name": "TODO",
            "date": "TODO",
            "time": "TODO",
            "place": "TODO",
        }
