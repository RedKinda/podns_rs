use crate::pronouns::{PronounDef, PronounRecord, PronounSet, PronounTag};

#[derive(Debug, PartialEq, Eq)]
pub enum ParserError {
    NotEnoughPronounParts,
    TooManyPronounParts,
    InvalidTag,
    TrailingCharacters,
    TrailingSlash,
    Empty,
}

enum ParserState {
    BuildingPronounDef { n: u8, trailing_slash: bool },
    BuildingTags,
}

struct Parser {
    state: ParserState,
    def_builder: Option<PronounSet>,
    tags: Vec<PronounTag>,
    comment: Option<String>,
}

impl Default for Parser {
    fn default() -> Self {
        Parser {
            def_builder: None,
            tags: Vec::new(),
            comment: None,
            state: ParserState::BuildingPronounDef {
                n: 0,
                trailing_slash: false,
            },
        }
    }
}

struct ParseStream<'a> {
    chars: std::str::Chars<'a>,
    peeked: Option<char>,
}
impl<'a> ParseStream<'a> {
    fn new(input: &'a str) -> Self {
        ParseStream {
            chars: input.chars(),
            peeked: None,
        }
    }

    fn peek(&mut self) -> Option<&char> {
        if self.peeked.is_none() {
            self.peeked = self.chars.next();
        }

        self.peeked.as_ref()
    }

    fn next(&mut self) -> Option<char> {
        if let Some(c) = self.peeked.take() {
            Some(c)
        } else {
            self.chars.next()
        }
    }

    fn skip_while<F: Fn(char) -> bool>(&mut self, predicate: F) {
        while let Some(c) = self.peek() {
            if predicate(*c) {
                self.next();
            } else {
                break;
            }
        }
    }

    fn skip_whitespace(&mut self) {
        self.skip_while(|c| c.is_whitespace());
    }

    fn take_while<F: Fn(char) -> bool>(&mut self, predicate: F) -> String {
        let mut result = String::new();
        while let Some(c) = self.peek() {
            if predicate(*c) {
                result.push(self.next().unwrap());
            } else {
                break;
            }
        }
        result
    }

    fn collect_remaining(&mut self) -> String {
        let mut result = String::new();
        while let Some(c) = self.next() {
            result.push(c);
        }
        result
    }
}

pub fn parse_record(input: &str) -> Result<PronounRecord, ParserError> {
    let mut parse_stream = ParseStream::new(input);
    let mut parser = Parser::default();

    parse_stream.skip_whitespace();

    while let Some(c) = parse_stream.peek() {
        match c {
            ';' => {
                if let ParserState::BuildingPronounDef {
                    n: _,
                    trailing_slash,
                } = parser.state
                    && trailing_slash
                {
                    return Err(ParserError::TrailingSlash);
                }
                // tag separator
                match parser.state {
                    ParserState::BuildingPronounDef { n, trailing_slash } => {
                        if n < 2 {
                            return Err(ParserError::NotEnoughPronounParts);
                        }
                        if trailing_slash {
                            return Err(ParserError::TrailingSlash);
                        }

                        parser.state = ParserState::BuildingTags;
                    }
                    ParserState::BuildingTags => {}
                }

                // process tag
                // first skip all `;`s, then skip all whitespace
                parse_stream.skip_while(|c| c == ';');
                parse_stream.skip_whitespace();

                let tag_string = parse_stream
                    .take_while(|ch| ch != ';' && ch != '#' && !ch.is_whitespace())
                    .to_lowercase();

                parser
                    .tags
                    .push(PronounTag::from_string(tag_string).ok_or(ParserError::InvalidTag)?);

                parse_stream.skip_whitespace();
            }
            '#' => {
                if let ParserState::BuildingPronounDef {
                    n: _,
                    trailing_slash,
                } = parser.state
                    && trailing_slash
                {
                    return Err(ParserError::TrailingSlash);
                }
                // comment, consume rest of line and add to comment
                parse_stream.next(); // skip the '#'
                parse_stream.skip_whitespace();
                parser.comment = Some(parse_stream.collect_remaining().trim_end().to_owned());
                break;
            }
            c => match parser.state {
                ParserState::BuildingPronounDef { n, trailing_slash } => {
                    match c {
                        '*' => {
                            if trailing_slash {
                                return Err(ParserError::TrailingSlash);
                            }
                            parser.def_builder = Some(PronounSet::Any);
                            parser.state = ParserState::BuildingTags;
                            parse_stream.next(); // consume '*'
                            parse_stream.skip_whitespace();
                            continue;
                        }
                        '!' => {
                            if trailing_slash {
                                return Err(ParserError::TrailingSlash);
                            }
                            parser.def_builder = Some(PronounSet::None);
                            parser.state = ParserState::BuildingTags;
                            parse_stream.next(); // consume '!'
                            parse_stream.skip_whitespace();
                            continue;
                        }
                        _ => {}
                    }

                    let part = parse_stream.take_while(|ch| {
                        ch != ';' && ch != '#' && ch != '/' && !ch.is_whitespace()
                    });

                    let pronoun_set = parser.def_builder.get_or_insert_with(|| {
                        PronounSet::Defined(PronounDef {
                            subject: String::new(),
                            object: String::new(),
                            possessive_determiner: None,
                            possessive_pronoun: None,
                            reflexive: None,
                        })
                    });

                    let pronoun_def = match pronoun_set {
                        PronounSet::Defined(def) => def,
                        _ => return Err(ParserError::TooManyPronounParts),
                    };

                    let part_to_update = match n {
                        0 => &mut pronoun_def.subject,
                        1 => &mut pronoun_def.object,
                        2 => pronoun_def.possessive_determiner.get_or_insert_default(),
                        3 => pronoun_def.possessive_pronoun.get_or_insert_default(),
                        4 => pronoun_def.reflexive.get_or_insert_default(),
                        _ => return Err(ParserError::TooManyPronounParts),
                    };

                    part_to_update.push_str(&part.to_lowercase());

                    parse_stream.skip_whitespace();
                    // take until the next /, then skip whitespace again
                    if let Some('/') = parse_stream.peek() {
                        parse_stream.next(); // consume '/'
                        parse_stream.skip_whitespace();
                        parser.state = ParserState::BuildingPronounDef {
                            n: n + 1,
                            trailing_slash: true,
                        };
                    } else {
                        parser.state = ParserState::BuildingPronounDef {
                            n: n + 1,
                            trailing_slash: false,
                        };
                    }
                }
                ParserState::BuildingTags => {
                    return Err(ParserError::TrailingCharacters);
                }
            },
        }
    }

    // finish parser, validate and build PronounRecord
    match parser.state {
        ParserState::BuildingPronounDef { n, trailing_slash } => {
            if parser.def_builder.is_some() && n < 2 {
                return Err(ParserError::NotEnoughPronounParts);
            }
            if trailing_slash {
                return Err(ParserError::TrailingSlash);
            }
        }
        ParserState::BuildingTags => {}
    }

    if parser.def_builder.is_none() && parser.tags.is_empty() && parser.comment.is_none() {
        return Err(ParserError::Empty);
    }

    if !parser.tags.is_empty() && parser.def_builder.is_none() {
        return Err(ParserError::NotEnoughPronounParts);
    }

    let record = PronounRecord {
        set: parser.def_builder,
        comment: parser.comment,
        tags: parser.tags,
    };

    Ok(record)
}

#[cfg(test)]
mod parser_tests {
    use super::{ParserError, PronounRecord, PronounSet, PronounTag, parse_record};

    macro_rules! test_case {
        ($name:ident, $input:expr, $expected_pronoun_set:expr, $expected_tags:expr, $expected_comment:expr) => {
            #[test]
            fn $name() {
                let record = parse_record($input).unwrap();
                assert_eq!(record.set, $expected_pronoun_set);
                assert_eq!(record.tags, $expected_tags);
                assert_eq!(record.comment, $expected_comment);
            }
        };
    }

    macro_rules! error_case {
        ($name:ident, $input:expr, $expected_error:expr) => {
            #[test]
            fn $name() {
                let result = parse_record($input);
                assert!(result.is_err());
                assert_eq!(result.err().unwrap(), $expected_error);
            }
        };
    }

    test_case!(
        test_simple,
        "she/her",
        Some(PronounSet::new_defined(
            "she".to_string(),
            "her".to_string(),
            None,
            None,
            None
        )),
        vec![],
        None
    );
    test_case!(
        test_expanded,
        "they/them; preferred; plural # Example comment",
        Some(PronounSet::new_defined(
            "they".to_string(),
            "them".to_string(),
            None,
            None,
            None
        )),
        vec![PronounTag::Preferred, PronounTag::Plural],
        Some("Example comment".to_string())
    );

    test_case!(
        test_parse_record_any,
        "* # Any pronouns",
        Some(PronounSet::Any),
        vec![],
        Some("Any pronouns".to_string())
    );

    test_case!(
        test_whitespaced_expanded,
        "  ze/hir  ;  preferred  #  Another comment  ",
        Some(PronounSet::new_defined(
            "ze".to_string(),
            "hir".to_string(),
            None,
            None,
            None
        )),
        vec![PronounTag::Preferred],
        Some("Another comment".to_string())
    );

    test_case!(
        test_multiple_semicolons,
        "xe/xem;;; preferred;; plural # Comment",
        Some(PronounSet::new_defined(
            "xe".to_string(),
            "xem".to_string(),
            None,
            None,
            None
        )),
        vec![PronounTag::Preferred, PronounTag::Plural],
        Some("Comment".to_string())
    );

    test_case!(
        test_only_comment,
        "# Just a comment",
        None,
        vec![],
        Some("Just a comment".to_string())
    );

    // error cases

    error_case!(
        test_error_not_enough_pronoun_parts,
        "she",
        ParserError::NotEnoughPronounParts
    );

    error_case!(
        test_error_trailing_characters,
        "they/them; preferred extra",
        ParserError::TrailingCharacters
    );

    error_case!(
        test_error_too_many_pronoun_parts,
        "they/them/their/theirs/themself/extra",
        ParserError::TooManyPronounParts
    );

    // test trailing slashes in various positions
    error_case!(
        test_error_trailing_slash,
        "they/them/ ",
        ParserError::TrailingSlash
    );

    error_case!(
        test_error_trailing_slash_before_tag,
        "they/them/; preferred",
        ParserError::TrailingSlash
    );

    error_case!(
        test_error_trailing_slash_before_comment,
        "they/them/ # comment",
        ParserError::TrailingSlash
    );

    error_case!(test_error_empty, "   ", ParserError::Empty);

    // test from RFC examples
    /*
    + she/her
    + he/him/his/his/himself;preferred
    + they/them/their/theirs/themself
    + they/them;preferred;plural
    + *
    + !
    + ze/zir/zir/zirself
    */

    test_case!(
        test_rfc_example_1,
        "she/her",
        Some(PronounSet::new_defined(
            "she".to_string(),
            "her".to_string(),
            None,
            None,
            None
        )),
        vec![],
        None
    );

    test_case!(
        test_rfc_example_2,
        "he/him/his/his/himself;preferred",
        Some(PronounSet::new_defined(
            "he".to_string(),
            "him".to_string(),
            Some("his".to_string()),
            Some("his".to_string()),
            Some("himself".to_string())
        )),
        vec![PronounTag::Preferred],
        None
    );

    test_case!(
        test_rfc_example_3,
        "they/them/their/theirs/themself",
        Some(PronounSet::new_defined(
            "they".to_string(),
            "them".to_string(),
            Some("their".to_string()),
            Some("theirs".to_string()),
            Some("themself".to_string())
        )),
        vec![],
        None
    );

    test_case!(
        test_rfc_example_4,
        "they/them;preferred;plural",
        Some(PronounSet::new_defined(
            "they".to_string(),
            "them".to_string(),
            None,
            None,
            None
        )),
        vec![PronounTag::Preferred, PronounTag::Plural],
        None
    );

    test_case!(test_rfc_example_5, "*", Some(PronounSet::Any), vec![], None);

    test_case!(
        test_rfc_example_6,
        "!",
        Some(PronounSet::None),
        vec![],
        None
    );

    test_case!(
        test_rfc_example_7,
        "ze/zir/zir/zirself",
        Some(PronounSet::new_defined(
            "ze".to_string(),
            "zir".to_string(),
            Some("zir".to_string()),
            Some("zirself".to_string()),
            None,
        )),
        vec![],
        None
    );

    // test non-canonical examples from RFC
    /*
    + SHE/HER # -> she/her
    + SHE /    HER # -> she/her
    + he/him;;;preferred # -> he/him;preferred
     */

    test_case!(
        test_noncanonical_1,
        "SHE/HER #",
        Some(PronounSet::new_defined(
            "she".to_string(),
            "her".to_string(),
            None,
            None,
            None
        )),
        vec![],
        Some("".to_string())
    );

    test_case!(
        test_noncanonical_2,
        "SHE /    HER #",
        Some(PronounSet::new_defined(
            "she".to_string(),
            "her".to_string(),
            None,
            None,
            None
        )),
        vec![],
        Some("".to_string())
    );

    test_case!(
        test_noncanonical_3,
        "he/him;;;preferred #",
        Some(PronounSet::new_defined(
            "he".to_string(),
            "him".to_string(),
            None,
            None,
            None
        )),
        vec![PronounTag::Preferred],
        Some("".to_string())
    );

    // error cases from RFC
    /*
    - she/her/
    - she
    - they/them/their/theirs/themself/extra
    - she/her;unknown-tag
     */

    error_case!(test_rfc_error_1, "she/her/", ParserError::TrailingSlash);

    error_case!(test_rfc_error_2, "she", ParserError::NotEnoughPronounParts);

    error_case!(
        test_rfc_error_3,
        "they/them/their/theirs/themself/extra",
        ParserError::TooManyPronounParts
    );

    error_case!(
        test_rfc_error_4,
        "she/her;unknown-tag",
        ParserError::InvalidTag
    );
}
