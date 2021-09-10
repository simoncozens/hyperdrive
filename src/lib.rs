use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyBytes;
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
            (
                "info",
                self.font_info
                    .as_ref()
                    .map_or(PyDict::new(py).to_object(py), |v| {
                        v.to_wrapped_object(loader, py)
                    }),
            ),
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

fn wrap_data(
    object: &Py<PyAny>,
    py: Python,
    ufo: &norad::Font,
    path: &PathBuf,
) -> Result<(), PyErr> {
    // For each path, do `Font.data[path] = content`.

    let data_set = object.getattr(py, "data").map_err(|e| {
        IondriveError::new_err(format!("Cannot get at Font.data: {}", e.to_string()))
    })?;
    for data_path in ufo.data_paths() {
        // ufoLib conventions require the path to always have POSIX forward slashes as
        // path separators.
        let internal_path = data_path.strip_prefix("data").map_err(|e| {
            IondriveError::new_err(format!(
                "Failed to prepare data file path: {}",
                e.to_string()
            ))
        })?;
        let internal_path_posix = path_as_posix(&internal_path)?;

        let full_path = path.join(data_path);
        let contents = std::fs::read(&full_path).map_err(|e| {
            IondriveError::new_err(format!(
                "Failed to read file {}: {}",
                &full_path.display(),
                e.to_string()
            ))
        })?;

        data_set
            .as_ref(py)
            .set_item(internal_path_posix, PyBytes::new(py, &contents))?;
    }

    // Signal to ufoLib2 code that norad loads data eagerly.
    if data_set.as_ref(py).hasattr("_lazy")? {
        data_set.as_ref(py).setattr("_lazy", false)?;
    }

    Ok(())
}

fn wrap_images(
    object: &Py<PyAny>,
    py: Python,
    ufo: &norad::Font,
    path: &PathBuf,
) -> Result<(), PyErr> {
    // For each path, do `Font.images[path] = content`.

    let images_set = object.getattr(py, "images").map_err(|e| {
        IondriveError::new_err(format!("Cannot get at Font.images: {}", e.to_string()))
    })?;
    for image_path in ufo.images_paths() {
        // ufoLib conventions require the path to always have POSIX forward slashes as
        // path separators.
        let internal_path = image_path.strip_prefix("images").map_err(|e| {
            IondriveError::new_err(format!(
                "Failed to prepare images file path: {}",
                e.to_string()
            ))
        })?;
        let internal_path_posix = path_as_posix(&internal_path)?;

        let full_path = path.join(image_path);
        let contents = std::fs::read(&full_path).map_err(|e| {
            IondriveError::new_err(format!(
                "Failed to read file {}: {}",
                &full_path.display(),
                e.to_string()
            ))
        })?;

        // Check for PNG header signature.
        if &contents[..8] != &[137u8, 80, 78, 71, 13, 10, 26, 10] {
            return Err(IondriveError::new_err(format!(
                "Image at {} does not seem to be a PNG file",
                image_path.display()
            )));
        }

        images_set
            .as_ref(py)
            .set_item(internal_path_posix, PyBytes::new(py, &contents))?;
    }

    // Signal to ufoLib2 code that norad loads data eagerly.
    if images_set.as_ref(py).hasattr("_lazy")? {
        images_set.as_ref(py).setattr("_lazy", false)?;
    }

    Ok(())
}

/// Return path as a string with only forward slashes as path separators.
///
/// Error out if the path
/// - cannot cleanly be converted to UTF-8, as the result is going to be
///   used as a key in the images and data dictionaries Python-side.
/// - does not appear to be relative or normalized, which is what we expect
///   from norad.
fn path_as_posix(path: &Path) -> PyResult<String> {
    let parts = path
        .components()
        .map(|c| match c {
            std::path::Component::Normal(n) => match n.to_str() {
                Some(s) => Ok(s),
                None => {
                    return Err(IondriveError::new_err(format!(
                        "Cannot cleanly represent path: {}",
                        c.as_os_str().to_string_lossy()
                    )))
                }
            },
            _ => {
                return Err(IondriveError::new_err(format!(
                    "Expected relative, normalized path but got: {}",
                    path.display()
                )))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(parts.join("/"))
}

create_exception!(iondrive, IondriveError, PyException);

/// Load and return a UFO from `path`, using the objects from `font_objects_module`.
///
/// The font objects module is the Python namespace containing the classes as
/// exported by ufoLib2, typically this will be the module `ufoLib2.objects`.
#[pyfunction]
#[pyo3(text_signature = "(font_objects_module, path, /)")]
fn load(font_objects_module: &PyModule, path: PathBuf) -> PyResult<PyObject> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    match norad::Font::load(&path) {
        Ok(ufo) => {
            let object = ufo.to_wrapped_object(font_objects_module, py);

            // Signal to ufoLib2 code that norad loads data eagerly.
            if object.as_ref(py).hasattr("_lazy")? {
                object.as_ref(py).setattr("_lazy", false)?;
            }

            // ufoLib2 and defcon objects set the `_path` attribute when loading
            // a UFO from disk, which fontmake relies on. Specifically set the
            // private attribute here because ufoLib2 doesn't allow to setattr
            // the public one.
            object.as_ref(py).setattr("_path", &path)?;

            // Wrap data and images separately until norad gains full support for them.
            wrap_data(&object, py, &ufo, &path)?;
            wrap_images(&object, py, &ufo, &path)?;

            Ok(object)
        }
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
