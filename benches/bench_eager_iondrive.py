import tempfile
from pathlib import Path

import iondrive
import ufoLib2.objects

tmp = Path(tempfile.gettempdir())

iondrive.load(ufoLib2.objects, tmp / "NotoSans-Bold.ufo")
iondrive.load(ufoLib2.objects, tmp / "NotoSans-CondensedBold.ufo")
iondrive.load(ufoLib2.objects, tmp / "NotoSans-CondensedLight.ufo")
iondrive.load(ufoLib2.objects, tmp / "NotoSans-CondensedSemiBold.ufo")
iondrive.load(ufoLib2.objects, tmp / "NotoSans-Condensed.ufo")
iondrive.load(ufoLib2.objects, tmp / "NotoSans-Light.ufo")
iondrive.load(ufoLib2.objects, tmp / "NotoSans-Regular.ufo")
iondrive.load(ufoLib2.objects, tmp / "NotoSans-SemiBold.ufo")
