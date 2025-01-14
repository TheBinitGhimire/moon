use super::host_parser::HostParser;
use super::Url;
use crate::{
    encode::{PercentEncodeSet, URLPercentEncode},
    helper::{
        is_c0_control_or_space, is_double_dot_path_segment, is_single_dot_path_segment,
        is_start_with_two_hex, is_start_with_windows_drive_letter, is_url_c,
        is_window_drive_letter, SPECIAL_SCHEME_PORTS,
    },
    UrlPath,
};
use regex::Regex;
use std::iter::FromIterator;

pub struct URLParser;

#[derive(Clone)]
pub enum URLParseState {
    SchemeStart,
    Scheme,
    NoScheme,
    File,
    SpecialRelativeOrAuthority,
    SpecialAuthoritySlashes,
    PathOrAuthority,
    OpaquePath,
    Fragment,
    Relative,
    SpecialAuthorityIgnoreSlashes,
    Authority,
    Path,
    RelativeSlash,
    Query,
    Host,
    Hostname,
    FileHost,
    Port,
    PathStart,
    FileSlash,
}

fn report_validation_error() {
    // TODO here....
}

impl URLParser {
    pub fn parse(raw_input: &str, base: Option<Url>) -> Option<Url> {
        URLParser::simple_parse(raw_input, base, None, None)
    }

    fn simple_parse(
        p_input: &str,
        base: Option<Url>,
        p_url: Option<Url>,
        p_state: Option<URLParseState>,
    ) -> Option<Url> {
        let mut url = p_url.unwrap_or(Url::new());
        let mut state = p_state.clone().unwrap_or(URLParseState::SchemeStart);

        let tab_newline_re = Regex::new(r"[\t\r\n]").unwrap();

        let mut input = p_input
            .trim_start_matches(|c| is_c0_control_or_space(c as u32))
            .to_string();
        input = tab_newline_re.replace_all(&input, "").to_string();

        let mut buffer = String::new();
        let mut at_sign_seen = false;
        let mut inside_brackets = false;
        let mut password_token_seen = false;
        let mut pointer = 0;

        let input_chars: Vec<char> = input.chars().collect();

        let remaining = |pointer| {
            let n = pointer + 1;
            String::from_iter(&input_chars.clone()[n..])
        };

        let codepoint_substr = |pointer| {
            let n = pointer;
            String::from_iter(&input_chars.clone()[n..])
        };

        let eof: char = unsafe { std::char::from_u32_unchecked(0xFFFFFFFF) };

        loop {
            let c = *input_chars.get(pointer).unwrap_or(&eof);

            match state {
                URLParseState::SchemeStart => {
                    if c.is_ascii_alphabetic() {
                        buffer.push(c.to_ascii_lowercase());
                        state = URLParseState::Scheme;
                    } else if p_state.is_none() {
                        state = URLParseState::NoScheme;
                        continue;
                    } else {
                        report_validation_error();
                        return None;
                    }
                }
                URLParseState::Scheme => {
                    if c != eof && (c.is_alphanumeric() || c == '+' || c == '-' || c == '.') {
                        buffer.push(c.to_ascii_lowercase());
                    } else if c == ':' {
                        // TODO: Skipped step 1, 3
                        url.scheme = buffer.clone();
                        buffer.clear();

                        if url.scheme == "file" {
                            if !remaining(pointer).starts_with("//") {
                                report_validation_error();
                            }
                            state = URLParseState::File;
                        } else if url.is_special()
                            && matches!(&base, Some(b) if b.scheme == url.scheme)
                        {
                            state = URLParseState::SpecialRelativeOrAuthority;
                        } else if url.is_special() {
                            state = URLParseState::SpecialAuthoritySlashes;
                        } else if remaining(pointer).starts_with("/") {
                            state = URLParseState::PathOrAuthority;
                            pointer += 1;
                        } else {
                            url.path = UrlPath::Opaque(String::new());
                            state = URLParseState::OpaquePath;
                        }
                    } else if p_state.is_none() {
                        buffer.clear();
                        state = URLParseState::NoScheme;
                        pointer = 0;
                        continue;
                    } else {
                        report_validation_error();
                        return None;
                    }
                }
                URLParseState::NoScheme => {
                    if base.is_none() || matches!(&base, Some(b) if b.has_opaque_path()) && c != '#'
                    {
                        report_validation_error();
                        return None;
                    } else if matches!(&base, Some(b) if b.has_opaque_path()) && c == '#' {
                        let base_clone = base.clone().unwrap();
                        url.scheme = base_clone.scheme;
                        url.path = base_clone.path;
                        url.query = base_clone.query;
                        url.fragment = Some(String::new());
                        state = URLParseState::Fragment;
                    } else if matches!(&base, Some(b) if b.scheme != "file") {
                        state = URLParseState::Relative;
                        continue;
                    } else {
                        state = URLParseState::File;
                        continue;
                    }
                }
                URLParseState::SpecialRelativeOrAuthority => {
                    if c == '/' && remaining(pointer).starts_with("/") {
                        state = URLParseState::SpecialAuthorityIgnoreSlashes;
                        pointer += 1;
                    } else {
                        report_validation_error();
                        state = URLParseState::Relative;
                        continue;
                    }
                }
                URLParseState::PathOrAuthority => {
                    if c == '/' {
                        state = URLParseState::Authority;
                    } else {
                        state = URLParseState::Path;
                        continue;
                    }
                }
                URLParseState::Relative => {
                    let base_clone = base.clone().unwrap();
                    url.scheme = base_clone.scheme;

                    if c == '/' {
                        state = URLParseState::RelativeSlash;
                    } else if url.is_special() && c == '\\' {
                        report_validation_error();
                        state = URLParseState::RelativeSlash;
                    } else {
                        url.host = base_clone.host;
                        url.port = base_clone.port;
                        url.path = base_clone.path;
                        url.query = base_clone.query;

                        if c == '?' {
                            url.query = Some(String::new());
                            state = URLParseState::Query;
                        } else if c == '#' {
                            url.fragment = Some(String::new());
                            state = URLParseState::Fragment;
                        } else if c != eof {
                            url.query = None;
                            url.shorten_path();
                            state = URLParseState::Path;
                            continue;
                        }
                    }
                }
                URLParseState::RelativeSlash => {
                    let base_clone = base.clone().unwrap();
                    if url.is_special() && c == '/' || c == '\\' {
                        if c == '\\' {
                            report_validation_error();
                            state = URLParseState::SpecialAuthorityIgnoreSlashes;
                        } else if c == '/' {
                            state = URLParseState::Authority;
                        } else {
                            url.host = base_clone.host;
                            url.port = base_clone.port;
                            state = URLParseState::Path;
                            continue;
                        }
                    }
                }
                URLParseState::SpecialAuthoritySlashes => {
                    if c == '/' && remaining(pointer).starts_with("/") {
                        state = URLParseState::SpecialAuthorityIgnoreSlashes;
                        pointer += 1;
                    } else {
                        report_validation_error();
                        state = URLParseState::SpecialAuthorityIgnoreSlashes;
                        continue;
                    }
                }
                URLParseState::SpecialAuthorityIgnoreSlashes => {
                    if c != '/' && c != '\\' {
                        state = URLParseState::Authority;
                        continue;
                    } else {
                        report_validation_error();
                    }
                }
                URLParseState::Authority => {
                    if c == '@' {
                        report_validation_error();
                        if at_sign_seen {
                            buffer.insert_str(0, "%40");
                        }
                        at_sign_seen = true;

                        for c in buffer.chars() {
                            if c == ':' && !password_token_seen {
                                password_token_seen = true;
                                continue;
                            }
                            // TODO: skipped password & username processing
                        }

                        buffer.clear();
                    } else if (c == eof || c == '/' || c == '?' || c == '#')
                        || (url.is_special() && c == '\\')
                    {
                        if at_sign_seen && buffer.is_empty() {
                            report_validation_error();
                            return None;
                        }
                        pointer -= buffer.len() + 1;
                        buffer.clear();
                        state = URLParseState::Host;
                    } else {
                        buffer.push(c);
                    }
                }
                URLParseState::Host | URLParseState::Hostname => {
                    if p_state.is_some() && url.scheme == "file" {
                        state = URLParseState::FileHost;
                        continue;
                    } else if c == ':' && !inside_brackets {
                        if buffer.is_empty() {
                            report_validation_error();
                            return None;
                        }

                        let host = HostParser::parse(&buffer, true);

                        if host.is_none() {
                            return None;
                        }

                        url.host = host;
                        buffer.clear();
                        state = URLParseState::Port;
                    } else if (c == eof || c == '/' || c == '?' || c == '#')
                        || (url.is_special() && c == '\\')
                    {
                        if url.is_special() && buffer.is_empty() {
                            report_validation_error();
                            return None;
                        }
                        let host = HostParser::parse(&buffer, true);

                        if host.is_none() {
                            return None;
                        }

                        url.host = host;
                        buffer.clear();
                        state = URLParseState::PathStart;
                        continue;
                    } else {
                        if c == '[' {
                            inside_brackets = true;
                        } else if c == ']' {
                            inside_brackets = false;
                        }
                        buffer.push(c);
                    }
                }
                URLParseState::Port => {
                    if c.is_ascii_digit() {
                        buffer.push(c);
                    } else if (c == eof || c == '/' || c == '?' || c == '#')
                        || (url.is_special() && c == '\\')
                        || p_state.is_some()
                    {
                        if !buffer.is_empty() {
                            match u16::from_str_radix(&buffer, 10) {
                                Ok(port) => {
                                    if SPECIAL_SCHEME_PORTS.iter().any(|(scheme, s_port)| {
                                        *scheme == url.scheme && *s_port == Some(port)
                                    }) {
                                        url.port = None;
                                    } else {
                                        url.port = Some(port);
                                    }
                                }
                                _ => {
                                    report_validation_error();
                                    return None;
                                }
                            }
                            buffer.clear();
                        }
                        state = URLParseState::PathStart;
                        continue;
                    } else {
                        report_validation_error();
                        return None;
                    }
                }
                URLParseState::File => {
                    url.scheme = String::from("file");
                    url.host = Some(String::new());
                    if c == '/' || c == '\\' {
                        if c == '\\' {
                            report_validation_error();
                        }
                        state = URLParseState::FileSlash;
                    } else if matches!(&base, Some(b) if b.scheme == "file") {
                        let base_clone = base.clone().unwrap();
                        url.host = base_clone.host;
                        url.path = base_clone.path;
                        url.query = base_clone.query;

                        if c == '?' {
                            url.query = Some(String::new());
                            state = URLParseState::Query;
                        } else if c == '#' {
                            url.fragment = Some(String::new());
                            state = URLParseState::Fragment;
                        } else if c != eof {
                            url.query = None;
                            if !is_start_with_windows_drive_letter(&codepoint_substr(pointer)) {
                                url.shorten_path();
                            } else {
                                report_validation_error();
                                url.path = UrlPath::List(Vec::new());
                            }
                            state = URLParseState::Path;
                            continue;
                        } else {
                            state = URLParseState::Path;
                            continue;
                        }
                    }
                }
                URLParseState::FileSlash => {
                    if c == '/' || c == '\\' {
                        if c == '\\' {
                            report_validation_error();
                        }
                        state = URLParseState::FileHost;
                    } else {
                        if matches!(&base, Some(b) if b.scheme == "file") {
                            let base_clone = base.clone().unwrap();
                            url.host = base_clone.host;
                        }
                        state = URLParseState::Path;
                        continue;
                    }
                }
                URLParseState::FileHost => {
                    if c == eof || c == '/' || c == '\\' || c == '?' || c == '#' {
                        if p_state.is_none() {
                            if is_window_drive_letter(&buffer) {
                                report_validation_error();
                            }
                            state = URLParseState::Path;
                        } else if buffer.is_empty() {
                            url.host = Some(String::new());
                            state = URLParseState::PathStart;
                        } else {
                            let mut host = HostParser::parse(&buffer, true);

                            if host.is_none() {
                                return None;
                            }

                            if host == Some("localhost".to_string()) {
                                host = Some(String::new());
                            }

                            url.host = host;
                            buffer.clear();
                            state = URLParseState::PathStart;
                        }
                        continue;
                    } else {
                        buffer.push(c);
                    }
                }
                URLParseState::PathStart => {
                    if url.is_special() {
                        if c == '\\' {
                            report_validation_error();
                        }
                        state = URLParseState::Path;
                        if c != '/' && c != '\\' {
                            continue;
                        }
                    } else if c == '?' {
                        url.query = Some(String::new());
                        state = URLParseState::Query;
                    } else if c == '#' {
                        url.fragment = Some(String::new());
                        state = URLParseState::Fragment;
                    } else if c != eof {
                        state = URLParseState::Path;
                        if c != '/' {
                            continue;
                        }
                    } else {
                        if p_state.is_some() && url.host.is_none() {
                            if let UrlPath::List(ref mut list) = &mut url.path {
                                list.push(String::new());
                            }
                        }
                    }
                }
                URLParseState::Path => {
                    if c == eof
                        || c == '/'
                        || (url.is_special() && c == '\\')
                        || c == '?'
                        || c == '#'
                    {
                        if url.is_special() && c == '\\' {
                            report_validation_error();
                        }
                        if is_double_dot_path_segment(&buffer) {
                            url.shorten_path();
                            if c != '/' && !(url.is_special() && c == '\\') {
                                if let UrlPath::List(ref mut list) = &mut url.path {
                                    list.push(String::new());
                                }
                            }
                        } else if is_single_dot_path_segment(&buffer)
                            && c != '/'
                            && !(url.is_special() && c == '\\')
                        {
                            if let UrlPath::List(ref mut list) = &mut url.path {
                                list.push(String::new());
                            }
                        } else if !is_single_dot_path_segment(&buffer) {
                            if url.scheme == "file"
                                && url.path.is_empty()
                                && is_window_drive_letter(&buffer)
                            {
                                buffer.replace_range(1..2, ":");
                            }
                            url.path.append(&buffer);
                        }
                        buffer.clear();
                        if c == '?' {
                            url.query = Some(String::new());
                            state = URLParseState::Query;
                        } else if c == '#' {
                            url.fragment = Some(String::new());
                            state = URLParseState::Fragment;
                        }
                    } else {
                        if !is_url_c(c) && c != '%' {
                            report_validation_error();
                        }
                        buffer.push_str(&URLPercentEncode::encode(
                            &[c as u8],
                            PercentEncodeSet::Path,
                            false,
                        ));
                    }
                }
                URLParseState::OpaquePath => {
                    if c == '?' {
                        url.query = Some(String::new());
                        state = URLParseState::Query;
                    } else if c == '#' {
                        url.fragment = Some(String::new());
                        state = URLParseState::Fragment;
                    } else {
                        if c != eof && !is_url_c(c) && c != '%' {
                            report_validation_error();
                        }

                        if c == '%' && !is_start_with_two_hex(&remaining(pointer)) {
                            report_validation_error();
                        }

                        if c != eof {
                            url.path.append(&URLPercentEncode::encode(
                                &[c as u8],
                                PercentEncodeSet::C0Control,
                                false,
                            ));
                        }
                    }
                }
                URLParseState::Query => {
                    // TODO: Set encoding to UTF8 if necessary
                    if (p_state.is_none() && c == '#') || c == eof {
                        let percent_encode_set = if url.is_special() {
                            PercentEncodeSet::SpecialQuery
                        } else {
                            PercentEncodeSet::Query
                        };
                        if let Some(query) = &mut url.query {
                            query.push_str(&URLPercentEncode::encode(
                                buffer.as_bytes(),
                                percent_encode_set,
                                false,
                            ));
                        }
                        buffer.clear();

                        if c == '#' {
                            url.fragment = Some(String::new());
                            state = URLParseState::Fragment;
                        }
                    } else {
                        if c != eof {
                            if !is_url_c(c) && c != '%' {
                                report_validation_error();
                            }
                            if c == '%' && !is_start_with_two_hex(&remaining(pointer)) {
                                report_validation_error();
                            }
                            buffer.push(c);
                        }
                    }
                }
                URLParseState::Fragment => {
                    if c != eof {
                        if !is_url_c(c) && c != '%' {
                            report_validation_error();
                        }

                        if c == '%' && !is_start_with_two_hex(&remaining(pointer)) {
                            report_validation_error();
                        }

                        if let Some(fragment) = &mut url.fragment {
                            fragment.push_str(&URLPercentEncode::encode(
                                &[c as u8],
                                PercentEncodeSet::Fragment,
                                false,
                            ));
                        }
                    }
                }
            }

            if input_chars.get(pointer).is_none() {
                break;
            }

            pointer += 1;
        }

        return Some(url);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let input_url = "http://google.com/index.php";

        let url = URLParser::parse(input_url, None).unwrap();

        assert_eq!(url.scheme, "http");
        assert_eq!(url.host, Some("google.com".to_string()));
        assert_eq!(url.port, None);
        assert_eq!(url.path, "index.php");
    }

    #[test]
    fn empty_path() {
        let input_url = "http://google.com";

        let url = URLParser::parse(input_url, None).unwrap();

        assert_eq!(url.scheme, "http");
        assert_eq!(url.host, Some("google.com".to_string()));
        assert_eq!(url.port, None);
        assert_eq!(url.path, "");
    }

    #[test]
    fn with_special_port() {
        let input_url = "https://google.com:443";

        let url = URLParser::parse(input_url, None).unwrap();

        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, Some("google.com".to_string()));
        assert_eq!(url.port, None);
        assert_eq!(url.path, "");
    }

    #[test]
    fn with_non_special_port() {
        let input_url = "https://google.com:1242";

        let url = URLParser::parse(input_url, None).unwrap();

        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, Some("google.com".to_string()));
        assert_eq!(url.port, Some(1242));
        assert_eq!(url.path, "");
    }

    #[test]
    fn invalid_port() {
        let input_url = "https://google.com:44a3";

        let url = URLParser::parse(input_url, None);

        assert!(url.is_none());
    }

    #[test]
    fn file_scheme() {
        let input_url = "file:///home/user/html/index.html";

        let url = URLParser::parse(input_url, None).unwrap();

        assert_eq!(url.scheme, "file");
        assert_eq!(url.path, "/home/user/html/index.html");
    }

    #[test]
    fn filename_with_base() {
        let input_url = "index.html";
        let base_url = URLParser::parse("http://google.com", None);

        let url = URLParser::parse(input_url, base_url).unwrap();

        assert_eq!(url.scheme, "http");
        assert_eq!(url.host, Some("google.com".to_string()));
        assert_eq!(url.path, "index.html");
    }

    #[test]
    fn filename_with_base_file() {
        let input_url = "index.html";
        let base_url = URLParser::parse("file:///home/user/data/", None);

        let url = URLParser::parse(input_url, base_url).unwrap();

        assert_eq!(url.scheme, "file");
        assert_eq!(url.path, "/home/user/data/index.html");
    }
}
