use super::wsp_comment::cfws;
use nom::{
    bytes::complete::tag,
    character::complete::one_of,
    combinator::{map_res, opt},
    multi::{many0, many1},
    sequence::{pair, tuple},
    IResult,
};

//atext   =   ALPHA / DIGIT /    ; Printable US-ASCII
//             "!" / "#" /        ;  characters not including
//             "$" / "%" /        ;  specials.  Used for atoms.
//             "&" / "'" /
//             "*" / "+" /
//             "-" / "/" /
//             "=" / "?" /
//             "^" / "_" /
//             "`" / "{" /
//             "|" / "}" /
//             "~"
pub fn atext(input: &str) -> IResult<&str, &str> {
    const ATEXT_STR: &str =
        "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!#$%&'*+-/=?^_`{|}~";
    let (next_input, c) = one_of(ATEXT_STR)(input)?;
    Ok((next_input, &input[..c.len_utf8()]))
}

pub struct Atom {
    pub pre_comment: Option<String>,
    pub content: String,
    pub post_comment: Option<String>,
}

// atom = [CFWS] 1*atext [CFWS]
pub fn atom(input: &str) -> IResult<&str, Atom> {
    let (next_input, (pre_comment, atext_vec, post_comment)) =
        tuple((opt(cfws), many1(atext), opt(cfws)))(input)?;

    let content: String = atext_vec.into_iter().collect();

    Ok((
        next_input,
        Atom {
            pre_comment,
            content,
            post_comment,
        },
    ))
}

// dot-atom-text =  1*atext *("." 1*atext)
pub fn dot_atom_text(input: &str) -> IResult<&str, &str> {
    let atext1_len = map_res(many1(atext), |atext1| -> Result<usize, ()> {
        Ok(atext1.into_iter().fold(0, |acc, c| acc + c.len()))
    });
    let dot_atext_len = map_res(
        pair(tag("."), many1(atext)),
        |(dot, atext1)| -> Result<usize, ()> {
            Ok(1 + atext1.into_iter().fold(0, |acc, c| acc + c.len()))
        },
    );
    map_res(
        pair(atext1_len, many0(dot_atext_len)),
        |(len1, len2_vec)| -> Result<&str, ()> {
            let len = len1 + len2_vec.into_iter().sum::<usize>();
            Ok(&input[..len])
        },
    )(input)
}

pub struct DotAtom {
    pub pre_comment: Option<String>,
    pub content: String,
    pub post_comment: Option<String>,
}

// dot-atom [CFWS] dot-atom-text [CFWS]
pub fn dot_atom(input: &str) -> IResult<&str, DotAtom> {
    let (next_input, (pre_comment, match_pattern, post_comment)) =
        tuple((opt(cfws), dot_atom_text, opt(cfws)))(input)?;
    let content = match_pattern.to_owned();
    Ok((
        next_input,
        DotAtom {
            pre_comment,
            content,
            post_comment,
        },
    ))
}

pub fn specials(input: &str) -> IResult<&str, &str> {
    const special_str: &str = r#"()<>[]:;@\,.""#;
    let (next_input, c) = one_of(special_str)(input)?;
    Ok((next_input, &input[..c.len_utf8()]))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_atext() {
        unimplemented!();
    }

    #[test]
    fn test_dot_atom_text() {
        unimplemented!();
    }

    #[test]
    fn test_atom() {
        unimplemented!();
    }

    #[test]
    fn test_dot_atom() {
        unimplemented!();
    }
}
