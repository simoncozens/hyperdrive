import tempfile
from pathlib import Path

import ufoLib2

tmp = Path(tempfile.gettempdir())

ufoLib2.Font.open(tmp / "NotoSansCJKJP-Thin.ufo", lazy=False)
ufoLib2.Font.open(tmp / "NotoSansCJKJP-Black.ufo", lazy=False)
