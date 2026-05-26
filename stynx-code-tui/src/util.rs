/// Strip ANSI CSI / OSC escape sequences so tool output renders cleanly.
/// Handles the common cases (`\x1b[...m`, OSC `\x1b]...\x07`/`\x1b\\`,
/// single-char escapes like `\x1b(B`).
pub fn strip_ansi(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == 0x1b && i + 1 < bytes.len() {
            let next = bytes[i + 1];
            // CSI: ESC '[' ... final byte in 0x40..=0x7E
            if next == b'[' {
                i += 2;
                while i < bytes.len() {
                    let c = bytes[i];
                    if (0x40..=0x7e).contains(&c) {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
            // OSC: ESC ']' ... terminated by BEL (0x07) or ESC '\\'
            if next == b']' {
                i += 2;
                while i < bytes.len() {
                    let c = bytes[i];
                    if c == 0x07 {
                        i += 1;
                        break;
                    }
                    if c == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
                continue;
            }
            // Two-char escapes: ESC '(' x or ESC ')' x etc.
            if matches!(next, b'(' | b')' | b'*' | b'+' | b'#') && i + 2 < bytes.len() {
                i += 3;
                continue;
            }
            // Skip lone ESC.
            i += 1;
            continue;
        }
        // Drop other C0 control chars except \n, \t, \r.
        if b < 0x20 && b != b'\n' && b != b'\t' && b != b'\r' {
            i += 1;
            continue;
        }
        // Keep multibyte UTF-8 intact.
        let char_end = utf8_char_end(bytes, i);
        out.push_str(&input[i..char_end]);
        i = char_end;
    }
    out
}

fn utf8_char_end(bytes: &[u8], i: usize) -> usize {
    let b = bytes[i];
    let width = if b < 0x80 {
        1
    } else if b & 0xe0 == 0xc0 {
        2
    } else if b & 0xf0 == 0xe0 {
        3
    } else if b & 0xf8 == 0xf0 {
        4
    } else {
        1
    };
    (i + width).min(bytes.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_csi() {
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m"), "red");
        assert_eq!(strip_ansi("\x1b[1;33mwarn\x1b[m hi"), "warn hi");
    }

    #[test]
    fn strips_osc() {
        assert_eq!(strip_ansi("\x1b]0;title\x07after"), "after");
        assert_eq!(strip_ansi("\x1b]0;title\x1b\\after"), "after");
    }

    #[test]
    fn keeps_unicode() {
        assert_eq!(strip_ansi("héllo 🦀"), "héllo 🦀");
    }
}
