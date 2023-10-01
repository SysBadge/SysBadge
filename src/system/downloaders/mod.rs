use alloc::string::String;

#[cfg(feature = "downloader-pk")]
mod pk;

#[cfg(feature = "downloader-pk")]
pub use pk::PkDownloader;

pub(crate) fn transform_name(input: &str) -> String {
    // Convert the input string to bytes
    let bytes = input.as_bytes();

    // Find the index of the first occurrence of more than 2 spaces or a tab
    let index = bytes.iter().enumerate().position(|(idx, &c)| {
        (c == b' ' && bytes.iter().skip(idx).take(3).all(|&x| x == b' ')) || c == b'\t'
    });

    // If such an index is found, truncate the input string at that position, else use the original input
    let filtered_input = match index {
        Some(idx) => &input[..idx],
        None => input,
    };

    // Filter out non-ASCII characters and create an iterator of chars
    let ascii_chars: String = filtered_input
        .chars()
        .filter(|c| {
            c.is_ascii_alphanumeric() || c.is_ascii_punctuation() || matches!(c, ' ' | '\t')
        })
        .collect();

    // Trim leading and trailing whitespace
    let trimmed_ascii = ascii_chars.trim();

    // Convert the trimmed string to a new String
    String::from(trimmed_ascii)
}
