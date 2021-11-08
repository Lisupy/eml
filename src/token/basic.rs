use nom::{
    branch::alt, 
    bytes::complete::tag, 
    character::complete::one_of, 
    IResult,
};

pub fn vchar(input: &str) -> IResult<&str, &str> {
    let vchar_str: String = ('!'..='~').collect();
    let (next_input, vc) = one_of(&vchar_str[..])(input)?;
    Ok((next_input, &input[..vc.len_utf8()]))
}

pub fn wsp(input: &str) -> IResult<&str, &str> {
    alt((tag(" "), tag("\t")))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::{self, ErrorKind};
    use nom::Err::Error;
    #[test]
    fn test_vchar() {
        assert_eq!(vchar("johndoe"), Ok(("ohndoe", "j")));
        assert_eq!(vchar("!ABCCDD"), Ok(("ABCCDD", "!")));
        assert_eq!(
            vchar(" aabbcc"),
            Err(Error(error::Error {
                input: " aabbcc",
                code: ErrorKind::OneOf
            }))
        );
        assert_eq!(
            vchar("\r\nabc"),
            Err(Error(error::Error {
                input: "\r\nabc",
                code: ErrorKind::OneOf
            }))
        );
    }

    #[test]
    fn test_wsp() {
        assert_eq!(wsp(" "), Ok(("", " ")));
        assert_eq!(wsp("\t"), Ok(("", "\t")));
        assert_eq!(
            wsp("\r"),
            Err(Error(error::Error {
                input: "\r",
                code: ErrorKind::Tag
            }))
        );
        assert_eq!(
            wsp("\n"),
            Err(Error(error::Error {
                input: "\n",
                code: ErrorKind::Tag,
            }))
        )
    }
}
