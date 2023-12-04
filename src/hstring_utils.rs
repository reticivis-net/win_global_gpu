use anyhow::Result;
use windows::core::HSTRING;

pub fn combine_hstring_paths(
    parent: &HSTRING,
    delimiter: &HSTRING,
    child: &HSTRING,
) -> Result<HSTRING> {
    hstring_from_utf16_buffer(
        [parent.as_wide(), delimiter.as_wide(), child.as_wide()]
            .concat()
            .as_slice(),
    )
}

pub fn hstring_from_utf16_buffer(utf16: &[u16]) -> Result<HSTRING> {
    let string = HSTRING::from_wide(utf16)?;
    // this function handles buffers which can have trailing nulls
    truncate_hstring(string, 0)
}

pub fn hstring_from_utf16(utf16: &[u8]) -> Result<HSTRING> {
    // vec of words to HSTRING
    Ok(HSTRING::from_wide(&bytes_to_words(utf16))?)
}

fn bytes_to_words(bytes: &[u8]) -> Vec<u16> {
    // thanks chatgpt for this btw
    // tried to do a transmute to a &[u16], broke shit
    bytes
        // group by 2 bytes
        .chunks(2)
        // map bytes to words
        // yes [chunk[0], chunk[1]] is necessary because 🤓 size cant be known at compile time
        .map(|chunk| u16::from_ne_bytes([chunk[0], chunk[1]]))
        // collect
        .collect()
}

pub fn truncate_hstring(hstring: HSTRING, trim: u16) -> Result<HSTRING> {
    let wide: &[u16] = hstring.as_wide();
    let to = match wide.iter().rposition(|c| *c != trim) {
        Some(pos) => pos + 1, // include the last char that's not `trim`
        None => 0,
    };

    Ok(HSTRING::from_wide(&wide[..to])?)
}

pub fn replace(haystack: HSTRING, needle: HSTRING, replace: HSTRING) -> Result<HSTRING> {
    let mut out: Vec<u16> = vec![];
    let haystack_words = haystack.as_wide();
    let needle_words = needle.as_wide();
    let replace_words = replace.as_wide();

    let haystack_len = haystack_words.len();
    let needle_len = needle_words.len();

    let mut i = 0;
    while i < haystack_len {
        if i + needle_len <= haystack_len && &haystack_words[i..i + needle_len] == needle_words {
            for word in replace_words {
                out.push(*word)
            }
            i += needle_len;
        } else {
            out.push(haystack_words[i]);
            i += 1;
        }
    }

    Ok(HSTRING::from_wide(out.as_slice())?)
}
