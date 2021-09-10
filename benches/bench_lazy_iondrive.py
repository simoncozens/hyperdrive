import tempfile
from pathlib import Path

import iondrive
import ufoLib2.objects

tmp = Path(tempfile.gettempdir())

u = iondrive.load(ufoLib2.objects, tmp / "NotoSans-Bold.ufo")
for g in u:
    pass
u = iondrive.load(ufoLib2.objects, tmp / "NotoSans-CondensedBold.ufo")
for g in u:
    pass
u = iondrive.load(ufoLib2.objects, tmp / "NotoSans-CondensedLight.ufo")
for g in u:
    pass
u = iondrive.load(ufoLib2.objects, tmp / "NotoSans-CondensedSemiBold.ufo")
for g in u:
    pass
u = iondrive.load(ufoLib2.objects, tmp / "NotoSans-Condensed.ufo")
for g in u:
    pass
u = iondrive.load(ufoLib2.objects, tmp / "NotoSans-Light.ufo")
for g in u:
    pass
u = iondrive.load(ufoLib2.objects, tmp / "NotoSans-Regular.ufo")
for g in u:
    pass
u = iondrive.load(ufoLib2.objects, tmp / "NotoSans-SemiBold.ufo")
for g in u:
    pass
