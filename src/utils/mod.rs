use std::io::Read;
use tokio::io::AsyncBufReadExt;

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

pub async fn read_nulltermed_str<R: Read + Sync + AsyncBufReadExt + Unpin>(buf: &mut R) -> Result<String, std::io::Error> {
    let mut temp = vec![];
    buf.read_until(0x00, &mut temp).await?;
    Ok( String::from_utf8_lossy(&temp.as_slice()[0..temp.len()-1]).to_string())
}