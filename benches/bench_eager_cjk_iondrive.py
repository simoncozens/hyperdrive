import tempfile
from pathlib import Path

import iondrive
import ufoLib2.objects

tmp = Path(tempfile.gettempdir())

iondrive.load(ufoLib2.objects, tmp / "NotoSansCJKJP-Thin.ufo")
iondrive.load(ufoLib2.objects, tmp / "NotoSansCJKJP-Black.ufo")
