use super::basic::{vchar, wsp};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{crlf, one_of},
    combinator::{map_res, opt},
    multi::{many0, many1},
    sequence::{pair, preceded, tuple},
    Err::{Error, Failure, Incomplete},
    IResult,
};

// quoted-pair = ("\" (VCHAR / WSP)) / obs-qp
// TODO: support obs_qp in future
pub fn quoted_pair(input: &str) -> IResult<&str, &str> {
    preceded(tag("\\"), alt((vchar, wsp)))(input)
}

// FWS = ([*WSP CRLF] 1*WSP)
// Return the folding wsp with folding format
pub fn fws(input: &str) -> IResult<&str, &str> {
    let crlf_opt_len = map_res(
        opt(pair(many0(wsp), crlf)),
        |match_opt| -> Result<usize, ()> {
            let mut len = 0;
            if let Some((vec_wsp, crlf_match)) = match_opt {
                len += vec_wsp.into_iter().map(|c| c.len()).sum::<usize>();
                len += crlf_match.len();
            }
            Ok(len)
        },
    );
    map_res(
        pair(crlf_opt_len, many1(wsp)),
        |(crlf_len, vec_wsp)| -> Result<&str, ()> {
            let len = vec_wsp.into_iter().fold(crlf_len, |len, c| len + c.len());
            Ok(&input[..len])
        },
    )(input)
}

// ctext = %33-39 /
//         %42-91 /
//         %93-126 /
//         obs-ctext
pub fn ctext(input: &str) -> IResult<&str, &str> {
    let ctext_str: String = ('!'..='\'').collect::<String>()
        + ('*'..='[').collect::<String>().as_str()
        + (']'..='~').collect::<String>().as_str();
    let (next_input, ctext_c) = one_of(&ctext_str[..])(input)?;
    Ok((next_input, &input[..ctext_c.len_utf8()]))
}

// ccontent = ctext / quoted-pair / comment
pub fn ccontent(input: &str) -> IResult<&str, String> {
    match alt((ctext, quoted_pair))(input) {
        Ok((next_input, pattern)) => Ok((next_input, pattern.to_owned())),
        Err(e) => match e {
            Incomplete(i) => Err(Incomplete(i)),
            Failure(f) => Err(Failure(f)),
            Error(_) => comment(input),
        },
    }
}

// comment = "(" *([FWS] ccontent) [FWS] ")"
pub fn comment(input: &str) -> IResult<&str, String> {
    let ccontent_string = map_res(
        pair(opt(fws), ccontent),
        |(opt_fws, match_ccontent)| -> Result<String, ()> {
            match opt_fws {
                None => Ok(match_ccontent.to_owned()),
                Some(_) => Ok(format!(" {}", match_ccontent)),
            }
        },
    );

    map_res(
        tuple((tag("("), many0(ccontent_string), opt(fws), tag(")"))),
        |(_, ccontent_string_vec, opt_fws, _)| -> Result<String, ()> {
            let result = format!("({})", ccontent_string_vec.join(""));
            Ok(result)
        },
    )(input)
}

// CFWS = (1*([FWS] comment) [FWS]) / FWS
pub fn cfws(input: &str) -> IResult<&str, String> {
    let fws_comment1 = map_res(
        pair(many1(pair(opt(fws), comment)), opt(fws)),
        |(match_vec, ws)| -> Result<String, ()> {
            let comment_string = match_vec
                .into_iter()
                .map(|(opt_fws, match_comment)| -> String {
                    match opt_fws {
                        None => match_comment,
                        Some(_) => format!(" {}", match_comment),
                    }
                })
                .collect();
            match ws {
                None => Ok(comment_string),
                Some(_) => Ok(format!("{} ", comment_string)),
            }
        },
    );

    let fws_string = map_res(fws, |_| -> Result<String, ()> { Ok(" ".to_owned()) });

    alt((fws_comment1, fws_string))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::{self, ErrorKind};
    use nom::Err::Error;
    use nom::Needed;
    #[test]
    fn test_quoted_pair() {
        assert_eq!(quoted_pair(r"\rABC"), Ok(("ABC", "r")));
        assert_eq!(quoted_pair(r"\\ABC"), Ok(("ABC", r"\")));
        assert_eq!(
            quoted_pair(r"ABC"),
            Err(Error(error::Error {
                input: "ABC",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            quoted_pair(r"\"),
            Err(Error(error::Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
    }

    #[test]
    fn test_ccontent() {
        assert_eq!(ccontent("[ABC"), Ok(("ABC", "[".to_owned())));
        assert_eq!(
            ccontent("(ABC [dd(ABC\\\")])AC"),
            Ok(("AC", "(ABC [dd(ABC\")])".to_owned()))
        );
        // TODO: Add No match case
    }

    #[test]
    fn test_comment() {
        assert_eq!(
            comment(r#"(this is the good time \\n \"Yes\")"#),
            Ok(("", r#"(this is the good time \n "Yes")"#.to_owned()))
        );
        assert_eq!(
            comment("(Hello (You (Are))) ABC"),
            Ok((" ABC", "(Hello (You (Are)))".to_owned()))
        );
        assert_eq!(
            comment("(ABC"),
            Err(Error(error::Error {
                input: "",
                code: ErrorKind::Tag
            }))
        );
        // Contain folding whitespace
        let folding_ccontent = "(\r\n ABC\r\n CDE\r\n\t(\r\n EDC))ABC";
        assert_eq!(comment(folding_ccontent), Ok(("ABC", "( ABC CDE ( EDC))".to_owned())));
    }
}
