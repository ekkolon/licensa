use crate::license::template::header::extract_hash_bang;

pub fn prepend_license_notice<H, F>(header: H, file_content: F) -> Vec<u8>
where
    H: AsRef<str>,
    F: AsRef<str>,
{
    let template = header.as_ref().as_bytes().to_vec();
    let file_content = file_content.as_ref().as_bytes();
    let mut line = extract_hash_bang(file_content).unwrap_or_default();
    let mut content = file_content.to_vec();

    let line_break = b'\n';

    if !line.is_empty() {
        content = content.split_off(line.len());
        if line[line.len() - 1] != line_break {
            line.push(line_break);
        }
        content = [line, template, content].concat();
    } else {
        content = [template, content].concat();
    }

    content
}
