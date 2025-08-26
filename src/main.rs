use regex::Regex;
use rfd::FileDialog;
use std::{env, fs, path::PathBuf, process, str};

/// Extracts ASCII strings from a byte slice with a minimum length.
/// Returns a vector of tuples `(offset, string)`.
fn extract_strings(data: &[u8], min_len: usize) -> Vec<(usize, String)> {
    let mut results = Vec::new();
    let mut start = None;

    for (i, &b) in data.iter().enumerate() {
        // Consider printable ASCII characters or space
        if b.is_ascii_graphic() || b == b' ' {
            if start.is_none() {
                start = Some(i);
            }
        } else if let Some(s) = start {
            // End of string sequence, check length
            if i - s >= min_len {
                if let Ok(s_str) = str::from_utf8(&data[s..i]) {
                    results.push((s, s_str.to_string()));
                }
            }
            start = None;
        }
    }

    results
}

fn main() {
    // Get user's home directory
    let home_dir = match env::var("HOME") {
        Ok(p) => p,
        Err(_) => {
            eprintln!("Unable to determine user's home directory");
            process::exit(1);
        }
    };

    // Show file picker dialog for input file
    let input_path = FileDialog::new()
        .set_title("Select input file")
        .pick_file()
        .unwrap_or_else(|| {
            eprintln!("No file selected");
            process::exit(1);
        });

    // Read the input file into memory
    let data = fs::read(&input_path).unwrap_or_else(|e| {
        eprintln!("Failed to read file {}: {}", input_path.display(), e);
        process::exit(1);
    });

    // Extract all strings of length >= 4 from the file
    let strings = extract_strings(&data, 4);
    let mut out_data = data.clone();

    // Regex to match any file path ending in `.rs` (non-space, non-colon characters)
    let re = Regex::new(r"([^\s:]*?\.rs)").unwrap();

    // Iterate over all extracted strings
    for (pos, s) in strings {
        // Replace every match of the regex with spaces in the output buffer
        for m in re.find_iter(&s) {
            let start_idx = pos + m.start();
            let end_idx = pos + m.end();

            if end_idx <= out_data.len() {
                for i in start_idx..end_idx {
                    out_data[i] = b' ';
                }
            }
        }
    }

    // Determine output path: user's home directory + original file name
    let output_path = PathBuf::from(&home_dir).join(input_path.file_name().unwrap_or_else(|| {
        eprintln!("Unable to retrieve input file name");
        process::exit(1);
    }));

    // Write sanitized output to file
    fs::write(&output_path, &out_data).unwrap_or_else(|e| {
        eprintln!("Failed to write file {}: {}", output_path.display(), e);
        process::exit(1);
    });

    println!(
        "\n✅ Successfully written output file: {}",
        output_path.display()
    );
}
