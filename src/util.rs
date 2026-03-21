use rerun::external::arrow;

pub fn extract_texts(array: &dyn arrow::array::Array) -> Vec<String> {
    use arrow::array::Array as _;
    let Some(string_array) = array
        .as_any()
        .downcast_ref::<arrow::array::StringArray>()
    else {
        return Vec::new();
    };
    (0..string_array.len())
        .map(|i| string_array.value(i).to_string())
        .collect()
}
