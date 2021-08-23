pub fn slice_index<T>(buf: &[T], needle: &[T]) -> Option<usize>
where T: Clone + PartialEq
{
    for i in 0..=buf.len() - needle.len() {
        if buf[i..].starts_with(needle) {
            return Some(i);
        }
    }
    None
}