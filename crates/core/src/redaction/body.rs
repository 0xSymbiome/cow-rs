use super::REDACTED_PLACEHOLDER;

/// Maximum number of sanitized response-body bytes retained before appending
/// [`RESPONSE_BODY_TRUNCATION_MARKER`].
pub const REDACTED_RESPONSE_BODY_MAX_BYTES: usize = 256;
/// Marker appended when [`redact_response_body`] truncates a sanitized body.
pub const RESPONSE_BODY_TRUNCATION_MARKER: &str = "...[truncated]";

/// Strips credential-shaped tokens from a partner response body and bounds the
/// retained diagnostic text.
///
/// Redaction runs before truncation so credentials after the byte cap cannot be
/// preserved by a too-early slice. The scanner is dependency-free and covers
/// common header, URL-query, JSON-string, and JWT-shaped credential echoes.
#[must_use]
pub fn redact_response_body(input: &str) -> String {
    truncate_sanitized_body(&strip_credential_tokens(input))
}

fn truncate_sanitized_body(input: &str) -> String {
    if input.len() <= REDACTED_RESPONSE_BODY_MAX_BYTES {
        return input.to_owned();
    }

    let mut boundary = REDACTED_RESPONSE_BODY_MAX_BYTES;
    while !input.is_char_boundary(boundary) {
        boundary -= 1;
    }

    let mut output = input[..boundary].to_owned();
    output.push_str(RESPONSE_BODY_TRUNCATION_MARKER);
    output
}

/// Removes credential-shaped spans from a response-body preview.
///
/// # Panics
///
/// Panics only if the scan offset stops pointing at a valid UTF-8 character
/// boundary; all offset updates are derived from Rust string match indices or
/// `char::len_utf8`.
fn strip_credential_tokens(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut offset = 0;

    while offset < input.len() {
        // Order matters: JWT and Bearer detectors run before the URL scheme
        // detector because a JWT body is all alphanumeric and would otherwise
        // be misclassified as a valid URL scheme prefix and preserved verbatim
        // ahead of the userinfo redaction. The credential_value_span detector
        // runs last because its key-prefix copy is the only path that re-emits
        // input bytes verbatim, and it routes those bytes through a recursive
        // strip_credential_tokens call.
        if let Some(end) = jwt_token_end(input, offset) {
            output.push_str(REDACTED_PLACEHOLDER);
            offset = end;
            continue;
        }

        if let Some(end) = bearer_token_end(input, offset) {
            output.push_str(REDACTED_PLACEHOLDER);
            offset = end;
            continue;
        }

        if let Some(redaction) = url_userinfo_span(input, offset) {
            output.push_str(&input[offset..redaction.value_start]);
            output.push_str(REDACTED_PLACEHOLDER);
            offset = redaction.value_end;
            continue;
        }

        if let Some(redaction) = userinfo_only_span(input, offset) {
            output.push_str(&input[offset..redaction.value_start]);
            output.push_str(REDACTED_PLACEHOLDER);
            offset = redaction.value_end;
            continue;
        }

        if let Some(redaction) = credential_value_span(input, offset) {
            // Recursively scan the key prefix so a JWT or bearer-shaped
            // substring embedded inside the credential key itself is
            // redacted before the verbatim copy reaches the output. The
            // recursive substring is strictly shorter than the surrounding
            // input, and credential_value_span cannot fire on a substring
            // that ends inside the value's opening quote, so recursion is
            // bounded.
            output.push_str(&strip_credential_tokens(
                &input[offset..redaction.value_start],
            ));
            output.push_str(REDACTED_PLACEHOLDER);
            offset = redaction.value_end;
            continue;
        }

        let next = input[offset..]
            .chars()
            .next()
            // SAFETY: offset is initialized to zero and then advanced only by
            // string match boundaries or by the UTF-8 width of the current
            // character.
            .expect("offset is always within the string");
        output.push(next);
        offset += next.len_utf8();
    }

    output
}

struct ValueRedaction {
    value_start: usize,
    value_end: usize,
}

fn url_userinfo_span(input: &str, offset: usize) -> Option<ValueRedaction> {
    if offset > 0 && is_url_scheme_char(input.as_bytes()[offset - 1]) {
        return None;
    }

    let scheme_end = input[offset..].find("://")? + offset;
    if scheme_end == offset || !input[offset..scheme_end].bytes().all(is_url_scheme_char) {
        return None;
    }

    let authority_start = scheme_end + 3;
    let mut authority_end = authority_start;
    while let Some(byte) = input.as_bytes().get(authority_end).copied() {
        if byte.is_ascii_whitespace() || matches!(byte, b'/' | b'?' | b'#' | b'"' | b'\'') {
            break;
        }
        authority_end += 1;
    }

    let at = input[authority_start..authority_end].find('@')? + authority_start;
    (at > authority_start).then_some(ValueRedaction {
        value_start: authority_start,
        value_end: at,
    })
}

fn credential_value_span(input: &str, offset: usize) -> Option<ValueRedaction> {
    let parsed = parse_key(input, offset)?;
    if !is_credential_key(parsed.key) {
        return None;
    }

    let delimiter = skip_ascii_whitespace(input, parsed.after_key);
    let delimiter_byte = *input.as_bytes().get(delimiter)?;
    if !matches!(delimiter_byte, b':' | b'=') {
        return None;
    }

    let value_prefix = skip_ascii_whitespace(input, delimiter + 1);
    let value_start = if normalized_key(parsed.key) == "authorization" {
        skip_authorization_scheme(input, value_prefix)
    } else {
        value_prefix
    };
    let (value_start, value_end) = value_span(input, value_start)?;

    Some(ValueRedaction {
        value_start,
        value_end,
    })
}

struct ParsedKey<'a> {
    key: &'a str,
    after_key: usize,
}

fn parse_key(input: &str, offset: usize) -> Option<ParsedKey<'_>> {
    let first = *input.as_bytes().get(offset)?;
    if matches!(first, b'"' | b'\'') {
        let quote = first;
        let key_start = offset + 1;
        let key_end = find_unescaped_byte(input, key_start, quote)?;
        return Some(ParsedKey {
            key: &input[key_start..key_end],
            after_key: key_end + 1,
        });
    }

    if offset > 0 && is_key_char(input.as_bytes()[offset - 1]) {
        return None;
    }

    let mut end = offset;
    while let Some(byte) = input.as_bytes().get(end).copied() {
        if !is_key_char(byte) {
            break;
        }
        end += 1;
    }

    if end == offset {
        return None;
    }

    Some(ParsedKey {
        key: &input[offset..end],
        after_key: end,
    })
}

fn is_credential_key(key: &str) -> bool {
    let normalized = normalized_key(key);
    normalized == "authorization"
        || normalized == "apikey"
        || normalized == "xapikey"
        || normalized == "token"
        || normalized == "secret"
        || normalized == "password"
        || normalized.contains("apikey")
        || normalized.contains("token")
        || normalized.contains("secret")
        || normalized.contains("password")
        || normalized.contains("authorization")
        || normalized.contains("bearer")
}

fn normalized_key(key: &str) -> String {
    key.bytes()
        .filter(|byte| !matches!(byte, b'-' | b'_'))
        .map(|byte| byte.to_ascii_lowercase() as char)
        .collect()
}

fn value_span(input: &str, offset: usize) -> Option<(usize, usize)> {
    let quote = input.as_bytes().get(offset).copied();
    if matches!(quote, Some(b'"' | b'\'')) {
        let value_start = offset + 1;
        let value_end = find_unescaped_byte(input, value_start, quote?)?;
        return Some((value_start, value_end));
    }

    let mut end = offset;
    while let Some(byte) = input.as_bytes().get(end).copied() {
        if !is_credential_value_char(byte) {
            break;
        }
        end += 1;
    }

    (end > offset).then_some((offset, end))
}

fn skip_authorization_scheme(input: &str, offset: usize) -> usize {
    let Some(after_bearer) = advance_ascii_case_insensitive(input, offset, "bearer") else {
        return offset;
    };

    let after_spaces = skip_ascii_whitespace(input, after_bearer);
    if after_spaces == after_bearer {
        offset
    } else {
        after_spaces
    }
}

fn jwt_token_end(input: &str, offset: usize) -> Option<usize> {
    if offset > 0 && is_credential_value_char(input.as_bytes()[offset - 1]) {
        return None;
    }
    if !input[offset..].starts_with("eyJ") {
        return None;
    }

    let mut end = offset;
    while let Some(byte) = input.as_bytes().get(end).copied() {
        if !is_credential_value_char(byte) {
            break;
        }
        end += 1;
    }

    (end - offset >= 23).then_some(end)
}

/// Detects a bare `://userinfo@host` span at `offset` and returns the byte
/// range of the userinfo so the scanner can replace it with the redaction
/// placeholder. Fires only when the input at `offset` literally begins with
/// `://` — i.e., the scheme prefix (if any) has already been emitted as
/// verbatim bytes by the scanner. Closes two evasion paths the stricter
/// `url_userinfo_span` cannot cover on its own:
///
/// * A mangled or non-ASCII scheme prefix (`https\xc3\xb6://user:pass@host`)
///   makes the strict scheme check fail, so the scanner emits the prefix
///   byte-by-byte and lands at the `:` of `://` with `offset` pointing at
///   the colon. This detector fires there and redacts the userinfo.
/// * A credential-keyed value followed immediately by a URL fragment
///   (`apiKey=secret://user:pass@host`) leaves the scanner at the `:` of
///   `://` after `credential_value_span` consumes the credential. The
///   strict `url_userinfo_span` returns `None` here because the previous
///   byte was a credential-value character. This detector fires
///   independently.
fn userinfo_only_span(input: &str, offset: usize) -> Option<ValueRedaction> {
    if !input[offset..].starts_with("://") {
        return None;
    }

    let authority_start = offset + 3;
    let mut authority_end = authority_start;
    while let Some(byte) = input.as_bytes().get(authority_end).copied() {
        if byte.is_ascii_whitespace() || matches!(byte, b'/' | b'?' | b'#' | b'"' | b'\'') {
            break;
        }
        authority_end += 1;
    }

    let at = input[authority_start..authority_end].find('@')? + authority_start;
    (at > authority_start).then_some(ValueRedaction {
        value_start: authority_start,
        value_end: at,
    })
}

/// Detects a `Bearer <token>` span anywhere in the input and returns the byte
/// offset one past the end of the token. Mirrors [`jwt_token_end`] for the
/// bearer-scheme case so opaque bearer tokens are redacted even when they are
/// echoed in a freeform response body rather than embedded under an
/// `authorization` key.
///
/// The detector fires whenever:
/// - the `Bearer` keyword (case-insensitive) appears at the current offset,
///   regardless of the preceding byte. The earlier word-boundary guard was
///   removed after fuzzing demonstrated that an attacker can defeat it by
///   prepending arbitrary characters (e.g. `"toBearer secret-..."`) — real
///   partner-response text rarely contains `Bearer <token>` substrings
///   unless the token actually is a credential;
/// - the keyword is followed by at least one ASCII whitespace separator
///   (so `BearerFoo` does not match);
/// - the trailing token is at least 4 credential-value characters long (so
///   `Bearer .` or `Bearer ?` does not match).
fn bearer_token_end(input: &str, offset: usize) -> Option<usize> {
    let after_bearer = advance_ascii_case_insensitive(input, offset, "bearer")?;
    let token_start = skip_ascii_whitespace(input, after_bearer);
    if token_start == after_bearer {
        return None;
    }

    let mut end = token_start;
    while let Some(byte) = input.as_bytes().get(end).copied() {
        if !is_credential_value_char(byte) {
            break;
        }
        end += 1;
    }

    (end - token_start >= 4).then_some(end)
}

fn find_unescaped_byte(input: &str, offset: usize, target: u8) -> Option<usize> {
    let mut current = offset;
    let mut escaped = false;

    while let Some(byte) = input.as_bytes().get(current).copied() {
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == target {
            return Some(current);
        }
        current += 1;
    }

    None
}

fn advance_ascii_case_insensitive(input: &str, offset: usize, needle: &str) -> Option<usize> {
    let end = offset.checked_add(needle.len())?;
    let candidate = input.get(offset..end)?;
    candidate.eq_ignore_ascii_case(needle).then_some(end)
}

fn skip_ascii_whitespace(input: &str, mut offset: usize) -> usize {
    while let Some(byte) = input.as_bytes().get(offset).copied() {
        if !byte.is_ascii_whitespace() {
            break;
        }
        offset += 1;
    }
    offset
}

const fn is_key_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')
}

const fn is_url_scheme_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'.' | b'-')
}

const fn is_credential_value_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-')
}
