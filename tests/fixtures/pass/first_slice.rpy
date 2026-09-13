newtype UserId = int

fn greet(id: UserId, name: str) -> str:
    return f"{id}: {name}"
