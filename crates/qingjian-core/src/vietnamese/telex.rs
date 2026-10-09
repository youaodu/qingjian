//! 越南语 Telex 转写。

const TONES: [(char, Tone); 5] = [
    ('s', Tone::Acute),
    ('f', Tone::Grave),
    ('r', Tone::Hook),
    ('x', Tone::Tilde),
    ('j', Tone::Dot),
];

#[derive(Debug, Clone, Copy)]
enum Tone {
    Acute,
    Grave,
    Hook,
    Tilde,
    Dot,
}

pub fn transcribe(input: &str) -> String {
    input
        .split(' ')
        .map(transcribe_word)
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn backspace_unit_len(input: &str, cursor: usize) -> usize {
    if cursor == 0 || cursor > input.len() || !input.is_char_boundary(cursor) {
        return 0;
    }
    let before = &input[..cursor];
    let Some(last) = before.chars().next_back() else {
        return 0;
    };
    let last_len = last.len_utf8();
    let previous_end = cursor.saturating_sub(last_len);
    if is_tone_key(last) && word_before(input, previous_end).chars().any(is_vowel) {
        return last_len + shape_or_letter_len_before(input, previous_end);
    }
    shape_or_letter_len_before(input, cursor)
}

fn shape_or_letter_len_before(input: &str, cursor: usize) -> usize {
    let before = &input[..cursor];
    let Some(last) = before.chars().next_back() else {
        return 0;
    };
    let last_len = last.len_utf8();
    let previous_end = cursor.saturating_sub(last_len);
    let previous = input[..previous_end].chars().next_back();
    if let Some(previous) = previous
        && is_shape_pair(previous, last)
        && !is_cancelled_double_shape(input, previous_end, previous, last)
    {
        return previous.len_utf8() + last_len;
    }
    last_len
}

fn transcribe_word(input: &str) -> String {
    let mut letters: Vec<char> = Vec::new();
    let mut tone = None;
    let mut cancelled_tone = None;
    for c in input.chars() {
        let lower = c.to_ascii_lowercase();
        if let Some((_, found)) = TONES.iter().find(|(key, _)| *key == lower)
            && letters.iter().any(|c| is_vowel(*c))
        {
            if tone.is_some_and(|(key, _)| key == lower) {
                tone = None;
                cancelled_tone = Some(lower);
                letters.push(c);
                continue;
            }
            if cancelled_tone == Some(lower) {
                letters.push(c);
                continue;
            }
            tone = Some((lower, *found));
            continue;
        }
        cancelled_tone = None;
        if apply_shape(&mut letters, c) {
            continue;
        }
        letters.push(c);
    }
    if let Some((_, tone)) = tone
        && let Some(index) = tone_index(&letters)
    {
        letters[index] = apply_tone(letters[index], tone);
    } else if let Some((key, _)) = tone {
        letters.push(key);
    }
    letters.into_iter().collect()
}

fn word_before(input: &str, cursor: usize) -> &str {
    input[..cursor]
        .rsplit_once(' ')
        .map_or(&input[..cursor], |(_, word)| word)
}

fn apply_shape(letters: &mut Vec<char>, c: char) -> bool {
    let Some(last) = letters.last_mut() else {
        return false;
    };
    let upper = last.is_ascii_uppercase();
    if let Some((first, second)) = unshape_pair(*last, c) {
        *last = if upper {
            first.to_ascii_uppercase()
        } else {
            first
        };
        letters.push(if c.is_ascii_uppercase() {
            second.to_ascii_uppercase()
        } else {
            second
        });
        return true;
    }
    let shaped = match (last.to_ascii_lowercase(), c.to_ascii_lowercase()) {
        ('a', 'a') => Some('â'),
        ('a', 'w') => Some('ă'),
        ('e', 'e') => Some('ê'),
        ('o', 'o') => Some('ô'),
        ('o', 'w') => Some('ơ'),
        ('u', 'w') => Some('ư'),
        ('d', 'd') => Some('đ'),
        _ => None,
    };
    if let Some(ch) = shaped {
        *last = if upper { uppercase(ch) } else { ch };
        true
    } else {
        false
    }
}

fn is_shape_pair(previous: char, current: char) -> bool {
    matches!(
        (previous.to_ascii_lowercase(), current.to_ascii_lowercase()),
        ('a', 'a') | ('a', 'w') | ('e', 'e') | ('o', 'o') | ('o', 'w') | ('u', 'w') | ('d', 'd')
    )
}

fn is_cancelled_double_shape(
    input: &str,
    previous_end: usize,
    previous: char,
    current: char,
) -> bool {
    if !matches!(
        (previous.to_ascii_lowercase(), current.to_ascii_lowercase()),
        ('a', 'a') | ('e', 'e') | ('o', 'o') | ('d', 'd')
    ) {
        return false;
    }
    let earlier_end = previous_end.saturating_sub(previous.len_utf8());
    let Some(earlier) = input[..earlier_end].chars().next_back() else {
        return false;
    };
    earlier.eq_ignore_ascii_case(&previous)
}

fn unshape_pair(shaped: char, current: char) -> Option<(char, char)> {
    Some(
        match (shaped.to_lowercase().next()?, current.to_ascii_lowercase()) {
            ('â', 'a') => ('a', 'a'),
            ('ă', 'w') => ('a', 'w'),
            ('ê', 'e') => ('e', 'e'),
            ('ô', 'o') => ('o', 'o'),
            ('ơ', 'w') => ('o', 'w'),
            ('ư', 'w') => ('u', 'w'),
            ('đ', 'd') => ('d', 'd'),
            _ => return None,
        },
    )
}

fn is_tone_key(c: char) -> bool {
    matches!(c.to_ascii_lowercase(), 's' | 'f' | 'r' | 'x' | 'j')
}

fn tone_index(letters: &[char]) -> Option<usize> {
    let vowels: Vec<usize> = letters
        .iter()
        .enumerate()
        .filter_map(|(index, c)| is_vowel(*c).then_some(index))
        .collect();
    if vowels.is_empty() {
        return None;
    }
    for preferred in ['ê', 'ơ', 'ô', 'â', 'ă'] {
        if let Some(index) = vowels
            .iter()
            .copied()
            .find(|index| letters[*index].to_lowercase().next() == Some(preferred))
        {
            return Some(index);
        }
    }
    Some(vowels[vowels.len() / 2])
}

fn is_vowel(c: char) -> bool {
    matches!(
        c.to_lowercase().next().unwrap_or(c),
        'a' | 'ă' | 'â' | 'e' | 'ê' | 'i' | 'o' | 'ô' | 'ơ' | 'u' | 'ư' | 'y'
    )
}

fn apply_tone(c: char, tone: Tone) -> char {
    let upper = c.is_uppercase();
    let lower = c.to_lowercase().next().unwrap_or(c);
    let toned = match (lower, tone) {
        ('a', Tone::Acute) => 'á',
        ('a', Tone::Grave) => 'à',
        ('a', Tone::Hook) => 'ả',
        ('a', Tone::Tilde) => 'ã',
        ('a', Tone::Dot) => 'ạ',
        ('ă', Tone::Acute) => 'ắ',
        ('ă', Tone::Grave) => 'ằ',
        ('ă', Tone::Hook) => 'ẳ',
        ('ă', Tone::Tilde) => 'ẵ',
        ('ă', Tone::Dot) => 'ặ',
        ('â', Tone::Acute) => 'ấ',
        ('â', Tone::Grave) => 'ầ',
        ('â', Tone::Hook) => 'ẩ',
        ('â', Tone::Tilde) => 'ẫ',
        ('â', Tone::Dot) => 'ậ',
        ('e', Tone::Acute) => 'é',
        ('e', Tone::Grave) => 'è',
        ('e', Tone::Hook) => 'ẻ',
        ('e', Tone::Tilde) => 'ẽ',
        ('e', Tone::Dot) => 'ẹ',
        ('ê', Tone::Acute) => 'ế',
        ('ê', Tone::Grave) => 'ề',
        ('ê', Tone::Hook) => 'ể',
        ('ê', Tone::Tilde) => 'ễ',
        ('ê', Tone::Dot) => 'ệ',
        ('i', Tone::Acute) => 'í',
        ('i', Tone::Grave) => 'ì',
        ('i', Tone::Hook) => 'ỉ',
        ('i', Tone::Tilde) => 'ĩ',
        ('i', Tone::Dot) => 'ị',
        ('o', Tone::Acute) => 'ó',
        ('o', Tone::Grave) => 'ò',
        ('o', Tone::Hook) => 'ỏ',
        ('o', Tone::Tilde) => 'õ',
        ('o', Tone::Dot) => 'ọ',
        ('ô', Tone::Acute) => 'ố',
        ('ô', Tone::Grave) => 'ồ',
        ('ô', Tone::Hook) => 'ổ',
        ('ô', Tone::Tilde) => 'ỗ',
        ('ô', Tone::Dot) => 'ộ',
        ('ơ', Tone::Acute) => 'ớ',
        ('ơ', Tone::Grave) => 'ờ',
        ('ơ', Tone::Hook) => 'ở',
        ('ơ', Tone::Tilde) => 'ỡ',
        ('ơ', Tone::Dot) => 'ợ',
        ('u', Tone::Acute) => 'ú',
        ('u', Tone::Grave) => 'ù',
        ('u', Tone::Hook) => 'ủ',
        ('u', Tone::Tilde) => 'ũ',
        ('u', Tone::Dot) => 'ụ',
        ('ư', Tone::Acute) => 'ứ',
        ('ư', Tone::Grave) => 'ừ',
        ('ư', Tone::Hook) => 'ử',
        ('ư', Tone::Tilde) => 'ữ',
        ('ư', Tone::Dot) => 'ự',
        ('y', Tone::Acute) => 'ý',
        ('y', Tone::Grave) => 'ỳ',
        ('y', Tone::Hook) => 'ỷ',
        ('y', Tone::Tilde) => 'ỹ',
        ('y', Tone::Dot) => 'ỵ',
        _ => c,
    };
    if upper { uppercase(toned) } else { toned }
}

fn uppercase(c: char) -> char {
    c.to_uppercase().next().unwrap_or(c)
}

#[cfg(test)]
mod tests {
    use super::{backspace_unit_len, transcribe};

    #[test]
    fn transcribes_common_telex_words() {
        assert_eq!(transcribe("tooi yeeu tieengs vieetj"), "tôi yêu tiếng việt");
        assert_eq!(transcribe("Vieetj Nam"), "Việt Nam");
        assert_eq!(transcribe("em trai"), "em trai");
        assert_eq!(transcribe("sau xinh rau"), "sau xinh rau");
        assert_eq!(transcribe("ooo"), "oo");
        assert_eq!(transcribe("aaa eee ddd"), "aa ee dd");
        assert_eq!(transcribe("os oss osss sss"), "ó os oss sss");
    }

    #[test]
    fn backspace_deletes_one_visible_shape_unit() {
        assert_eq!(backspace_unit_len("too", 3), 2);
        assert_eq!(backspace_unit_len("tos", 3), 2);
        assert_eq!(backspace_unit_len("toos", 4), 3);
        assert_eq!(backspace_unit_len("ooo", 3), 1);
        assert_eq!(backspace_unit_len("tooi", 4), 1);
        assert_eq!(backspace_unit_len("too", 1), 1);
        assert_eq!(backspace_unit_len("dd", 2), 2);
        assert_eq!(backspace_unit_len("tr", 2), 1);
        assert_eq!(backspace_unit_len("em tr", 5), 1);
    }
}
