use std::collections::VecDeque;

use std::char;

macro_rules! try_option {
    ($o:expr) => {
        match $o {
            Some(s) => s,
            None => return None,
        }
    }
}

/// Takes in a string with backslash escapes written out with literal backslash characters and
/// converts it to a string with the proper escaped characters.
///
/// This is the reverse operation to [str::escape_default].
///
/// # Arguments
///
/// * `s`: String to unescape.
///
/// # Returns
///
/// On success, the unescaped string is returned. If invalid escape sequences were found, `None`
/// is returned.
///
/// [str::escape_default]: https://doc.rust-lang.org/std/primitive.str.html#method.escape_default
pub fn unescape(s: &str) -> Option<String> {
    let mut queue : VecDeque<_> = String::from(s).chars().collect();
    let mut s = String::new();

    while let Some(c) = queue.pop_front() {
        if c != '\\' {
            s.push(c);
            continue;
        }

        match queue.pop_front() {
            Some('b') => s.push('\u{0008}'),
            Some('f') => s.push('\u{000C}'),
            Some('n') => s.push('\n'),
            Some('r') => s.push('\r'),
            Some('t') => s.push('\t'),
            Some('\'') => s.push('\''),
            Some('\"') => s.push('\"'),
            Some('\\') => s.push('\\'),
            Some('u') => s.push(try_option!(unescape_unicode(&mut queue))),
            Some('x') => s.push(try_option!(unescape_byte(&mut queue))),
            Some(c) if c.is_digit(8) => s.push(try_option!(unescape_octal(c, &mut queue))),
            _ => return None
        };
    }

    Some(s)
}

fn unescape_unicode(queue: &mut VecDeque<char>) -> Option<char> {
    let mut s = String::new();

    if try_option!(queue.front()) == &'{' {
        // \u{X} form with an arbitrary number of digits.
        queue.pop_front();
        'outer: loop {
            match queue.pop_front() {
                Some('}') => break 'outer,
                Some(c) => s.push(c),
                None => return None,
            }
        }
    } else {
        // \uXXXX form with exactly four digits.
        for _ in 0..4 {
            s.push(try_option!(queue.pop_front()));
        }
    }

    let u = try_option!(u32::from_str_radix(&s, 16).ok());
    char::from_u32(u)
}

fn unescape_byte(queue: &mut VecDeque<char>) -> Option<char> {
    let mut s = String::new();

    for _ in 0..2 {
        s.push(try_option!(queue.pop_front()));
    }

    let u = try_option!(u32::from_str_radix(&s, 16).ok());
    char::from_u32(u)
}

fn unescape_octal(c: char, queue: &mut VecDeque<char>) -> Option<char> {
    let mut s = String::new();
    s.push(c);

    match c {
        '0' | '1' | '2' | '3' => {
            if push_octal_char(queue, &mut s) {
                push_octal_char(queue, &mut s);
            }
        }
        '4' | '5' | '6' | '7' => {
            push_octal_char(queue, &mut s);
        }
        _ => {}
    }

    let u = try_option!(u32::from_str_radix(&s, 8).ok());
    char::from_u32(u)
}

/// Looks at the queue and pushes the next character to the string if it's an
/// octal digit. Returns whether a push was done.
fn push_octal_char(queue: &mut VecDeque<char>, s: &mut String) -> bool {
    match queue.front() {
        Some(c) => {
            if c.is_digit(8) {
                s.push(*c);
                queue.pop_front();
                true
            } else {
                false
            }
        }
        None => false,
    }
}
