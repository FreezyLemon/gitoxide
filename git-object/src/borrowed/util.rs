use crate::borrowed::{Error, Signature};
use crate::{Sign, Time};
use bstr::{BStr, ByteSlice};
use btoi::btoi;
use nom::combinator::recognize;
use nom::multi::{many1_count, many1_countc};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take, take_until, take_while_m_n},
    character::is_digit,
    sequence::{preceded, terminated, tuple},
    IResult,
};

pub(crate) const NL: &[u8] = b"\n";
pub(crate) const SPACE: &[u8] = b" ";

pub(crate) fn parse_header_field_multiline<'a>(
    i: &'a [u8],
    name: &'static [u8],
) -> IResult<&'a [u8], &'a [u8], Error> {
    preceded(
        terminated(tag(name), tag(SPACE)),
        recognize(tuple((
            is_not(NL),
            tag(NL),
            many1_count(terminated(tuple((tag(SPACE), take_until(NL))), tag(NL))),
        ))),
    )(i)
}

pub(crate) fn parse_header_field<'a, T>(
    i: &'a [u8],
    name: &'static [u8],
    parse_value: impl Fn(&'a [u8]) -> IResult<&'a [u8], T, Error>,
) -> IResult<&'a [u8], T, Error> {
    terminated(
        preceded(terminated(tag(name), tag(SPACE)), parse_value),
        tag(NL),
    )(i)
}

fn is_hex_digit_lc(b: u8) -> bool {
    match b {
        b'0'..=b'9' => true,
        b'a'..=b'f' => true,
        _ => false,
    }
}

pub(crate) fn parse_hex_sha1(i: &[u8]) -> IResult<&[u8], &BStr, Error> {
    take_while_m_n(40usize, 40, is_hex_digit_lc)(i).map(|(i, o)| (i, o.as_bstr()))
}

pub(crate) fn parse_signature(i: &[u8]) -> IResult<&[u8], Signature, Error> {
    let (i, (name, email, time_in_seconds, tzsign, tzhour, tzminute)) = tuple((
        terminated(take_until(&b" <"[..]), take(2usize)),
        terminated(take_until(&b"> "[..]), take(2usize)),
        terminated(take_until(SPACE), take(1usize)),
        alt((tag(b"-"), tag(b"+"))),
        take_while_m_n(2usize, 2, |b| is_digit(b)),
        take_while_m_n(2usize, 2, |b| is_digit(b)),
    ))(i)
    .map_err(Error::context(
        "tagger <name> <<email>> <time seconds since epoch> <+|-><HHMM>",
    ))?;

    let sign = if tzsign[0] == b'-' {
        Sign::Minus
    } else {
        Sign::Plus
    };
    let hours = btoi::<i32>(&tzhour).map_err(|e| {
        nom::Err::Error(Error::ParseIntegerError(
            "invalid 'hours' string",
            tzhour.into(),
            e,
        ))
    })?;
    let minutes = btoi::<i32>(&tzminute).map_err(|e| {
        nom::Err::Error(Error::ParseIntegerError(
            "invalid 'minutes' string",
            tzminute.into(),
            e,
        ))
    })?;
    let offset = (hours * 3600 + minutes * 60) * if sign == Sign::Minus { -1 } else { 1 };

    Ok((
        i,
        Signature {
            name: name.as_bstr(),
            email: email.as_bstr(),
            time: Time {
                time: btoi::<u32>(time_in_seconds).map_err(|e| {
                    nom::Err::Error(Error::ParseIntegerError(
                        "Could parse to seconds",
                        time_in_seconds.into(),
                        e,
                    ))
                })?,
                offset,
                sign,
            },
        },
    ))
}
