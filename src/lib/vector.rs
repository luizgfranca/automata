pub fn push_if_some<T>(v: &mut Vec<T>, obj: Option<T>) {
    if let Some(value) = obj {
        v.push(value);
    }
}
