import math
from pathlib import Path

import pytest
import ufoLib2

import iondrive

UFOS = [
    Path("tests/data/MutatorSansBoldCondensed.ufo"),
    Path("tests/data/UbuTestData.ufo"),
    Path("tests/data/SourceSans_ExtraLight.ufo"),
    Path("tests/data/NotoSans-Regular.ufo"),
    Path("tests/data/Empty.ufo"),
    Path("tests/data/dataimagetest.ufo"),
]


def close_enough(a: float, b: float) -> bool:
    """Determine whether norad's f32 decimals are close enough to Python's
    f64 decimals."""
    return math.isclose(a, b, rel_tol=1e-07)


@pytest.mark.parametrize("path", UFOS, ids=lambda p: p.name)
def test_equivalence(path: Path) -> None:
    font = ufoLib2.Font.open(path)
    id_font = iondrive.load(ufoLib2.objects, path)

    assert font.lib == id_font.lib
    assert font.groups == id_font.groups
    assert font.kerning == id_font.kerning
    assert font.features == id_font.features
    assert font.info == id_font.info
    assert font.layers.keys() == id_font.layers.keys()
    for layer_name in font.layers.keys():
        layer = font.layers[layer_name]
        id_layer = id_font.layers[layer_name]
        assert layer.color == id_layer.color
        assert layer.lib == id_layer.lib
        assert layer.keys() == id_layer.keys()
        for glyph_name in layer.keys():
            # Compare fields one by one so we can use approximate equal comparisons.
            glyph = layer[glyph_name]
            id_glyph = id_layer[glyph_name]
            assert glyph.name == id_glyph.name
            assert close_enough(glyph.width, id_glyph.width)
            assert close_enough(glyph.height, id_glyph.height)
            assert glyph.unicodes == id_glyph.unicodes
            assert glyph.image == id_glyph.image
            assert glyph.lib == id_glyph.lib
            assert glyph.note == id_glyph.note
            assert len(glyph.anchors) == len(id_glyph.anchors)
            for anchor, id_anchor in zip(glyph.anchors, id_glyph.anchors):
                assert anchor.name == id_anchor.name
                assert close_enough(anchor.x, id_anchor.x)
                assert close_enough(anchor.y, id_anchor.y)
                assert anchor.color == id_anchor.color
                assert anchor.identifier == id_anchor.identifier
            assert len(glyph.components) == len(id_glyph.components)
            for component, id_component in zip(glyph.components, id_glyph.components):
                assert component.baseGlyph == id_component.baseGlyph
                assert component.identifier == id_component.identifier
                t, id_t = component.transformation, id_component.transformation
                assert close_enough(t.xx, id_t.xx)
                assert close_enough(t.xy, id_t.xy)
                assert close_enough(t.yx, id_t.yx)
                assert close_enough(t.yy, id_t.yy)
                assert close_enough(t.dx, id_t.dx)
                assert close_enough(t.dy, id_t.dy)
            assert len(glyph.contours) == len(id_glyph.contours)
            for contour, id_contour in zip(glyph.contours, id_glyph.contours):
                assert contour.identifier == id_contour.identifier
                assert len(contour.points) == len(id_contour.points)
                for point, id_point in zip(contour.points, id_contour.points):
                    assert close_enough(point.x, id_point.x)
                    assert close_enough(point.y, id_point.y)
                    assert point.type == id_point.type
                    assert point.smooth == id_point.smooth
                    assert point.name == id_point.name
                    assert point.identifier == id_point.identifier
            assert len(glyph.guidelines) == len(id_glyph.guidelines)
            for guideline, id_guideline in zip(glyph.guidelines, id_glyph.guidelines):
                if guideline.x is None:
                    assert guideline.x == id_guideline.x
                else:
                    assert close_enough(guideline.x, id_guideline.x)
                if guideline.y is None:
                    assert guideline.y == id_guideline.y
                else:
                    assert close_enough(guideline.y, id_guideline.y)
                if guideline.angle is None:
                    assert guideline.angle == id_guideline.angle
                else:
                    assert close_enough(guideline.angle, id_guideline.angle)
                assert guideline.name == id_guideline.name
                assert guideline.color == id_guideline.color
                assert guideline.identifier == id_guideline.identifier

    assert font.data == id_font.data
    assert font.images == id_font.images
