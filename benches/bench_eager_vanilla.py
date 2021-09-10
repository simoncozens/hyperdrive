import tempfile
from pathlib import Path

import ufoLib2

tmp = Path(tempfile.gettempdir())

ufoLib2.Font.open(tmp / "NotoSans-Bold.ufo", lazy=False)
ufoLib2.Font.open(tmp / "NotoSans-CondensedBold.ufo", lazy=False)
ufoLib2.Font.open(tmp / "NotoSans-CondensedLight.ufo", lazy=False)
ufoLib2.Font.open(tmp / "NotoSans-CondensedSemiBold.ufo", lazy=False)
ufoLib2.Font.open(tmp / "NotoSans-Condensed.ufo", lazy=False)
ufoLib2.Font.open(tmp / "NotoSans-Light.ufo", lazy=False)
ufoLib2.Font.open(tmp / "NotoSans-Regular.ufo", lazy=False)
ufoLib2.Font.open(tmp / "NotoSans-SemiBold.ufo", lazy=False)
