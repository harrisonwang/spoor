use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString};
use spoor_core::{Format, ParseLimits, ParseRequest};
use std::str::FromStr;

pyo3::create_exception!(_native, SpoorError, PyException);

#[pyfunction(signature = (data, source_name=None, content_type=None, format=None, max_parse_bytes=None))]
fn parse_bytes(
    py: Python<'_>,
    data: &[u8],
    source_name: Option<&str>,
    content_type: Option<&str>,
    format: Option<&str>,
    max_parse_bytes: Option<usize>,
) -> PyResult<Py<PyAny>> {
    let request = request(data, source_name, content_type, format, max_parse_bytes)?;
    let result = py
        .detach(|| spoor_core::parse(&request))
        .map_err(to_py_error)?;
    let value =
        serde_json::to_value(result).map_err(|error| PyException::new_err(error.to_string()))?;
    value_to_python(py, &value)
}

#[pyfunction(signature = (data, source_name=None, content_type=None))]
fn detect_format(
    py: Python<'_>,
    data: &[u8],
    source_name: Option<&str>,
    content_type: Option<&str>,
) -> PyResult<String> {
    let request = request(data, source_name, content_type, None, None)?;
    py.detach(|| spoor_core::detect_format(&request))
        .map(|format| format.to_string())
        .map_err(to_py_error)
}

fn request<'a>(
    data: &'a [u8],
    source_name: Option<&'a str>,
    content_type: Option<&'a str>,
    format: Option<&str>,
    max_parse_bytes: Option<usize>,
) -> PyResult<ParseRequest<'a>> {
    let mut request = ParseRequest::new(data);
    request.source_name = source_name;
    request.content_type = content_type;
    request.format_hint = format
        .map(Format::from_str)
        .transpose()
        .map_err(to_py_error)?;
    if let Some(max_parse_bytes) = max_parse_bytes {
        request.limits = ParseLimits { max_parse_bytes };
    }
    Ok(request)
}

fn to_py_error(error: spoor_core::SpoorError) -> PyErr {
    SpoorError::new_err(error.to_json())
}

fn value_to_python(py: Python<'_>, value: &serde_json::Value) -> PyResult<Py<PyAny>> {
    match value {
        serde_json::Value::Null => Ok(py.None()),
        serde_json::Value::Bool(value) => {
            Ok(PyBool::new(py, *value).to_owned().into_any().unbind())
        }
        serde_json::Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                Ok(PyInt::new(py, value).into_any().unbind())
            } else if let Some(value) = value.as_u64() {
                Ok(PyInt::new(py, value).into_any().unbind())
            } else {
                Ok(PyFloat::new(py, value.as_f64().unwrap_or_default())
                    .into_any()
                    .unbind())
            }
        }
        serde_json::Value::String(value) => Ok(PyString::new(py, value).into_any().unbind()),
        serde_json::Value::Array(values) => {
            let values = values
                .iter()
                .map(|value| value_to_python(py, value))
                .collect::<PyResult<Vec<_>>>()?;
            Ok(PyList::new(py, values)?.into_any().unbind())
        }
        serde_json::Value::Object(values) => {
            let dict = PyDict::new(py);
            for (key, value) in values {
                dict.set_item(key, value_to_python(py, value)?)?;
            }
            Ok(dict.into_any().unbind())
        }
    }
}

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("SpoorError", module.py().get_type::<SpoorError>())?;
    module.add_function(wrap_pyfunction!(parse_bytes, module)?)?;
    module.add_function(wrap_pyfunction!(detect_format, module)?)?;
    Ok(())
}
