use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyDict;
use pyo3::wrap_pyfunction;

mod anchor;
mod component;
mod contour;
mod contourpoint;
mod guideline;
mod info;
mod plist;

trait ToWrappedPyObject {
    fn to_wrapped_object(&self, loader: &PyModule, py: Python) -> PyObject;
}

trait MyToPyObject {
    fn to_object(&self, py: Python) -> PyObject;
}

impl<T> ToWrappedPyObject for Option<T>
where
    T: ToWrappedPyObject,
{
    fn to_wrapped_object(&self, loader: &PyModule, py: Python) -> PyObject {
        return self
            .as_ref()
            .map_or(py.None(), |x| x.to_wrapped_object(loader, py));
    }
}

impl<T> ToWrappedPyObject for Vec<T>
where
    T: ToWrappedPyObject,
{
    fn to_wrapped_object(&self, loader: &PyModule, py: Python) -> PyObject {
        self.iter()
            .map(|x| x.to_wrapped_object(loader, py))
            .collect::<Vec<PyObject>>()
            .to_object(py)
    }
}

impl<T> MyToPyObject for Vec<T>
where
    T: MyToPyObject,
{
    fn to_object(&self, py: Python) -> PyObject {
        self.iter()
            .map(|x| x.to_object(py))
            .collect::<Vec<PyObject>>()
            .to_object(py)
    }
}
impl<T> MyToPyObject for Option<T>
where
    T: MyToPyObject,
{
    fn to_object(&self, py: Python) -> PyObject {
        self.as_ref().map_or(py.None(), |x| x.to_object(py))
    }
}

impl MyToPyObject for Arc<str> {
    fn to_object(&self, py: Python) -> PyObject {
        Arc::clone(self).to_string().to_object(py)
    }
}

impl<A, B> MyToPyObject for BTreeMap<A, B>
where
    A: ToPyObject + std::fmt::Debug,
    B: MyToPyObject + std::fmt::Debug,
{
    fn to_object(&self, py: Python) -> PyObject {
        let d = PyDict::new(py);
        for (k, v) in self.iter() {
            d.set_item(k, v.to_object(py)).unwrap();
        }
        d.into()
    }
}

impl ToWrappedPyObject for Arc<norad::Glyph> {
    fn to_wrapped_object(&self, loader: &PyModule, py: Python) -> PyObject {
        let cls = loader.getattr("Glyph").unwrap();
        let kwargs = [
            ("name", self.name.to_object(py)),
            ("width", self.width.to_object(py)),
            (
                "unicodes",
                self.codepoints
                    .iter()
                    .map(|l| (*l as u32).to_object(py))
                    .collect::<Vec<PyObject>>()
                    .to_object(py),
            ),
            ("lib", self.lib.to_object(py)),
            ("note", self.note.to_object(py)),
            ("anchors", self.anchors.to_wrapped_object(loader, py)),
            ("contours", self.contours.to_wrapped_object(loader, py)),
            ("components", self.components.to_wrapped_object(loader, py)),
            ("guidelines", self.guidelines.to_wrapped_object(loader, py)),
        ]
        .into_py_dict(py);
        cls.call((), Some(kwargs)).unwrap().into()
    }
}

impl ToWrappedPyObject for norad::Layer {
    fn to_wrapped_object(&self, loader: &PyModule, py: Python) -> PyObject {
        let cls = loader.getattr("Layer").unwrap();
        let kwargs = [
            ("name", self.name().to_object(py)),
            (
                "glyphs",
                self.iter()
                    .map(|l| l.to_wrapped_object(loader, py))
                    .collect::<Vec<PyObject>>()
                    .to_object(py),
            ),
            ("lib", self.lib.to_object(py)),
            (
                "color",
                self.color
                    .as_ref()
                    .map(|c| c.to_rgba_string())
                    .to_object(py),
            ), // ()
        ]
        .into_py_dict(py);
        cls.call((), Some(kwargs)).unwrap().into()
    }
}

fn wrap_layerset(layers: &norad::LayerSet, loader: &PyModule, py: Python) -> PyObject {
    let wrapped_layers: Vec<PyObject> = layers
        .iter()
        .map(|l| l.to_wrapped_object(loader, py))
        .collect();

    let cls = loader.getattr("LayerSet").unwrap();
    cls.call_method(
        "from_iterable",
        (wrapped_layers, layers.default_layer().name().as_ref()),
        None,
    )
    .unwrap()
    .into()
}

fn wrap_kerning(kerning: Option<&norad::Kerning>, py: Python) -> PyObject {
    match kerning {
        Some(kerning) => {
            let d = PyDict::new(py);
            for (left, v) in kerning.iter() {
                for (right, kern) in v.iter() {
                    d.set_item((left, right).to_object(py), kern.to_object(py))
                        .unwrap();
                }
            }
            d.into()
        }
        None => PyDict::new(py).into(),
    }
}

impl ToWrappedPyObject for norad::Font {
    fn to_wrapped_object(&self, loader: &PyModule, py: Python) -> PyObject {
        let font = loader.getattr("Font").unwrap();

        let kwargs = [
            ("lib", self.lib.to_object(py)),
            ("layers", wrap_layerset(&self.layers, loader, py)),
            ("info", self.font_info.to_wrapped_object(loader, py)),
            (
                "features",
                self.features
                    .as_ref()
                    .map_or("", |v| v.as_str())
                    .to_object(py),
            ),
            (
                "groups",
                self.groups
                    .as_ref()
                    .map_or(PyDict::new(py).to_object(py), |v| v.to_object(py)),
            ),
            ("kerning", wrap_kerning(self.kerning.as_ref(), py)),
        ]
        .into_py_dict(py);
        font.call((), Some(kwargs)).unwrap().into()
    }
}

create_exception!(readwrite_ufo_glif, IondriveError, PyException);

/// Load and return a UFO from `path`, using the objects from `font_objects_module`.
///
/// The font objects module is the Python namespace containing the classes as
/// exported by ufoLib2, typically this will be the module `ufoLib2.objects`.
#[pyfunction]
#[pyo3(text_signature = "(font_objects_module, path, /)")]
fn load(loader: &PyModule, path: PathBuf) -> PyResult<PyObject> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    match norad::Font::load(Path::new(&path)) {
        Ok(ufo) => Ok(ufo.to_wrapped_object(loader, py)),
        Err(error) => Err(IondriveError::new_err(error.to_string())),
    }
}

/// Iondrive is a glue library to load [Unified Font Object](ufo) files using norad.
///
/// The goal is to load data faster than can be done by Python and then pass it
/// over to Python.
///
/// [ufo]: https://unifiedfontobject.org/
#[pymodule]
fn iondrive(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(load, m)?).unwrap();

    m.add("IondriveError", py.get_type::<IondriveError>())?;

    Ok(())
}
