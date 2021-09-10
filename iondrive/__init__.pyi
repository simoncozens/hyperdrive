import os
from types import ModuleType
from typing import Any

class IondriveError(Exception): ...

def load(font_objects_module: ModuleType, path: str | os.PathLike[str]) -> Any: ...
