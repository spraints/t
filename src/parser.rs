use crate::entry::{into_time_entries, Entry, Time, TimeEntry, TZ};
use crate::timesource::TimeSource;
use std::error::Error;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::str::Utf8Error;

#[derive(Debug)]
struct ParseError {
    message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ParseError {}

pub fn parse_entries<R: Read, TS: TimeSource>(r: R, ts: &TS) -> Result<Vec<Entry>, Box<dyn Error>> {
    let r = BufReader::new(r);
    let mut parser = Parser::new(r, ts.local_offset());
    let mut res = vec![];
    loop {
        match parser.parse_entry()? {
            None => break,
            Some(entry) => res.push(entry),
        }
    }
    Ok(res)
}

pub fn parse_time_entries<R: Read, TS: TimeSource>(
    r: R,
    ts: &TS,
) -> Result<Vec<TimeEntry>, Box<dyn Error>> {
    Ok(into_time_entries(parse_entries(r, ts)?))
}

pub fn parse_entry<R: BufRead, TS: TimeSource>(
    r: R,
    ts: &TS,
) -> Result<(Option<Entry>, R), Box<dyn Error>> {
    let mut parser = Parser::new(r, ts.local_offset());
    let entry = parser.parse_entry()?;
    Ok((entry, parser.reader))
}

pub fn write_entries(w: &mut impl Write, entries: &[Entry]) -> Result<(), Box<dyn Error>> {
    for entry in entries {
        write!(w, "{}", entry)?;
    }
    Ok(())
}

struct Parser<R: BufRead> {
    reader: R,
    default_tz: time::UtcOffset,
    line: usize,
    col: usize,
    parse_state: ParseState,
}

/// Morsel is either an annotation or the start of a time entry.
enum Morsel {
    None,
    Time(Time, bool),
    Note(String),
}

struct ParseState {
    states: Vec<ParseStateMote>,
    state: usize,
    digits: [u128; 256],

    init: usize,

    note: usize,

    year1: usize,
    month1: usize,
    day1: usize,
    hour1: usize,
    min1: usize,
    tzsign1: usize,
    tz1: usize,

    comma: usize,

    year2: usize,
    month2: usize,
    day2: usize,
    hour2: usize,
    min2: usize,
    tzsign2: usize,
    tz2: usize,
}

const MINUS: u128 = 99;
const PLUS: u128 = 88;

impl ParseState {
    fn new() -> Self {
        let mut digits = [0; 256];
        digits['0' as usize] = 0;
        digits['1' as usize] = 1;
        digits['2' as usize] = 2;
        digits['3' as usize] = 3;
        digits['4' as usize] = 4;
        digits['5' as usize] = 5;
        digits['6' as usize] = 6;
        digits['7' as usize] = 7;
        digits['8' as usize] = 8;
        digits['9' as usize] = 9;
        digits['-' as usize] = 99;
        digits['+' as usize] = 88;

        let mut states = vec![];

        ParseStateMote::error(&mut states);

        let init = states.len();
        ParseStateMote::first_char(&mut states, 1, 2);
        let note = states.len();
        ParseStateMote::note(&mut states);
        let year1 = states.len();
        ParseStateMote::number(&mut states, '-'); // year
        ParseStateMote::skip(&mut states);
        let month1 = states.len();
        ParseStateMote::number(&mut states, '-'); // month
        ParseStateMote::skip(&mut states);
        let day1 = states.len();
        ParseStateMote::number(&mut states, ' '); // day
        ParseStateMote::skip(&mut states);
        let hour1 = states.len();
        ParseStateMote::number(&mut states, ':'); // hour
        ParseStateMote::skip(&mut states);
        let min1 = states.len();
        ParseStateMote::minute(&mut states, Some(4)); // minute
        ParseStateMote::tz_sign(&mut states);
        let tzsign1 = states.len();
        ParseStateMote::skip(&mut states);
        let tz1 = states.len();
        ParseStateMote::tz_value(&mut states, Some(1));

        let comma = states.len();
        ParseStateMote::skip(&mut states); // ,

        let year2 = states.len();
        ParseStateMote::number(&mut states, '-'); // year
        ParseStateMote::skip(&mut states);
        let month2 = states.len();
        ParseStateMote::number(&mut states, '-'); // month
        ParseStateMote::skip(&mut states);
        let day2 = states.len();
        ParseStateMote::number(&mut states, ' '); // day
        ParseStateMote::skip(&mut states);
        let hour2 = states.len();
        ParseStateMote::number(&mut states, ':'); // hour
        ParseStateMote::skip(&mut states);
        let min2 = states.len();
        ParseStateMote::minute(&mut states, None); // minute
        ParseStateMote::tz_sign(&mut states);
        let tzsign2 = states.len();
        ParseStateMote::skip(&mut states);
        let tz2 = states.len();
        ParseStateMote::tz_value(&mut states, None);

        println!("tzsign1 = {tzsign1}, tzsign2 = {tzsign2}");

        Self {
            state: init,
            states,
            digits,

            init,

            note,

            year1,
            month1,
            day1,
            hour1,
            min1,
            tzsign1,
            tz1,

            comma,

            year2,
            month2,
            day2,
            hour2,
            min2,
            tzsign2,
            tz2,
        }
    }

    fn reset(&mut self) {
        for st in self.states.iter_mut() {
            st.i = 0;
            st.accum = 0;
            st.visited = false;
        }
        self.state = self.init;
    }

    fn add(&mut self, c: u8) {
        let old_state = self.state;
        self.state = self.states[self.state].next[c as usize];
        let st = &mut self.states[self.state];
        let old_val = st.accum;
        st.accum = (st.shift_factor * st.accum) + self.digits[c as usize];
        st.chars[st.i] = c;
        st.i += 1;
        println!(
            "{:?}: from {old_state} to {}, value {old_val} => {}",
            c as char, self.state, st.accum
        );
        st.visited = true;
    }

    fn res<F: Fn() -> TZ>(&self, implied_tz: F) -> Result<Option<Entry>, Box<dyn Error>> {
        let res = match self.state {
            x if x == self.init => Ok(None),
            x if x == self.note => Ok(Some(Entry::Note(self.string(self.note)?))),
            x if x == self.min1 => Ok(Some(Entry::Time(TimeEntry {
                start: Time::new(
                    // todo - count digits and fail here if it's not the right number?
                    self.val(self.year1) as u16,
                    self.val(self.month1) as u8,
                    self.val(self.day1) as u8,
                    self.val(self.hour1) as u8,
                    self.val(self.min1) as u8,
                    implied_tz(),
                )?,
                stop: None,
            }))),
            x if x == self.tz1 => Ok(Some(Entry::Time(TimeEntry {
                start: Time::new(
                    // todo - count digits and fail here if it's not the right number?
                    self.val(self.year1) as u16,
                    self.val(self.month1) as u8,
                    self.val(self.day1) as u8,
                    self.val(self.hour1) as u8,
                    self.val(self.min1) as u8,
                    self.tz(self.tzsign1, self.tz1),
                )?,
                stop: None,
            }))),
            x if x == self.comma => Ok(Some(Entry::Time(TimeEntry {
                start: Time::new(
                    // todo - count digits and fail here if it's not the right number?
                    self.val(self.year1) as u16,
                    self.val(self.month1) as u8,
                    self.val(self.day1) as u8,
                    self.val(self.hour1) as u8,
                    self.val(self.min1) as u8,
                    self.maybe_tz(self.tzsign1, self.tz1, implied_tz),
                )?,
                stop: None,
            }))),
            x if x == self.min2 => Ok(Some(Entry::Time(TimeEntry {
                start: Time::new(
                    // todo - count digits and fail here if it's not the right number?
                    self.val(self.year1) as u16,
                    self.val(self.month1) as u8,
                    self.val(self.day1) as u8,
                    self.val(self.hour1) as u8,
                    self.val(self.min1) as u8,
                    self.maybe_tz(self.tzsign1, self.tz1, &implied_tz),
                )?,
                stop: Some(Time::new(
                    self.val(self.year2) as u16,
                    self.val(self.month2) as u8,
                    self.val(self.day2) as u8,
                    self.val(self.hour2) as u8,
                    self.val(self.min2) as u8,
                    implied_tz(),
                )?),
            }))),
            x if x == self.tz2 => Ok(Some(Entry::Time(TimeEntry {
                start: Time::new(
                    // todo - count digits and fail here if it's not the right number?
                    self.val(self.year1) as u16,
                    self.val(self.month1) as u8,
                    self.val(self.day1) as u8,
                    self.val(self.hour1) as u8,
                    self.val(self.min1) as u8,
                    self.maybe_tz(self.tzsign1, self.tz1, &implied_tz),
                )?,
                stop: Some(Time::new(
                    self.val(self.year2) as u16,
                    self.val(self.month2) as u8,
                    self.val(self.day2) as u8,
                    self.val(self.hour2) as u8,
                    self.val(self.min2) as u8,
                    self.tz(self.tzsign2, self.tz2),
                )?),
            }))),
            _ => Err(format!("invalid entry (state = {})", self.state).into()),
        };
        println!("=> {res:?}");
        res
    }

    fn val(&self, i: usize) -> u128 {
        self.states[i].accum
    }

    fn string(&self, i: usize) -> Result<String, Utf8Error> {
        let st = &self.states[i];
        let chars = &st.chars[1..st.i];
        std::str::from_utf8(chars).map(str::to_string)
    }

    fn maybe_tz<F: Fn() -> TZ>(&self, s: usize, n: usize, implied_tz: F) -> TZ {
        if self.states[n].visited {
            println!("COMMA! has tz");
            self.tz(s, n)
        } else {
            println!("COMMA! does not has tz");
            implied_tz()
        }
    }

    fn tz(&self, s: usize, n: usize) -> TZ {
        println!("sign! {}", self.states[s].accum);
        let val = self.val(n);
        let hr = val / 100;
        let min = val % 100;
        let mut off = hr as i16 * 60 + min as i16;
        if self.states[s].accum == MINUS {
            off *= -1;
        }
        TZ::Known(time::UtcOffset::minutes(off))
    }
}

struct ParseStateMote {
    next: [usize; 256],
    shift_factor: u128,

    accum: u128,
    visited: bool,
    i: usize,
    chars: [u8; 100],
}

impl ParseStateMote {
    fn error(v: &mut Vec<Self>) {
        v.push(Self {
            next: [0; 256],
            accum: 0,
            shift_factor: 0,
            visited: false,
            i: 0,
            chars: [0; 100],
        })
    }
    fn first_char(v: &mut Vec<Self>, note_off: usize, year_off: usize) {
        let mut next = [0; 256];
        next[' ' as usize] = v.len();
        next['#' as usize] = v.len() + note_off;
        next['0' as usize] = v.len() + year_off;
        next['1' as usize] = v.len() + year_off;
        next['2' as usize] = v.len() + year_off;
        next['3' as usize] = v.len() + year_off;
        next['4' as usize] = v.len() + year_off;
        next['5' as usize] = v.len() + year_off;
        next['6' as usize] = v.len() + year_off;
        next['7' as usize] = v.len() + year_off;
        next['8' as usize] = v.len() + year_off;
        next['9' as usize] = v.len() + year_off;
        v.push(Self {
            next,
            accum: 0,
            shift_factor: 0,
            visited: false,
            i: 0,
            chars: [0; 100],
        })
    }

    fn note(v: &mut Vec<Self>) {
        v.push(Self {
            next: [v.len(); 256],
            accum: 0,
            shift_factor: 0,
            visited: false,
            i: 0,
            chars: [0; 100],
        })
    }

    fn number(v: &mut Vec<Self>, nextc: char) {
        let mut next = [0; 256];
        next['0' as usize] = v.len();
        next['1' as usize] = v.len();
        next['2' as usize] = v.len();
        next['3' as usize] = v.len();
        next['4' as usize] = v.len();
        next['5' as usize] = v.len();
        next['6' as usize] = v.len();
        next['7' as usize] = v.len();
        next['8' as usize] = v.len();
        next['9' as usize] = v.len();
        next[nextc as usize] = v.len() + 1;
        v.push(Self {
            next,
            accum: 0,
            shift_factor: 10,
            visited: false,
            i: 0,
            chars: [0; 100],
        })
    }

    fn skip_n(v: &mut Vec<Self>, offset: usize) {
        let mut next = [0; 256];
        next['0' as usize] = v.len() + offset;
        next['1' as usize] = v.len() + offset;
        next['2' as usize] = v.len() + offset;
        next['3' as usize] = v.len() + offset;
        next['4' as usize] = v.len() + offset;
        next['5' as usize] = v.len() + offset;
        next['6' as usize] = v.len() + offset;
        next['7' as usize] = v.len() + offset;
        next['8' as usize] = v.len() + offset;
        next['9' as usize] = v.len() + offset;
        next[' ' as usize] = v.len();
        v.push(Self {
            next,
            accum: 0,
            shift_factor: 0,
            visited: false,
            i: 0,
            chars: [0; 100],
        })
    }

    fn skip(v: &mut Vec<Self>) {
        Self::skip_n(v, 1)
    }

    fn minute(v: &mut Vec<Self>, comma_off: Option<usize>) {
        let mut next = [0; 256];
        next['0' as usize] = v.len();
        next['1' as usize] = v.len();
        next['2' as usize] = v.len();
        next['3' as usize] = v.len();
        next['4' as usize] = v.len();
        next['5' as usize] = v.len();
        next['6' as usize] = v.len();
        next['7' as usize] = v.len();
        next['8' as usize] = v.len();
        next['9' as usize] = v.len();
        next[' ' as usize] = v.len() + 1;
        if let Some(x) = comma_off {
            next[',' as usize] = v.len() + x;
        }
        v.push(Self {
            next,
            accum: 0,
            shift_factor: 10,
            visited: false,
            i: 0,
            chars: [0; 100],
        })
    }

    fn tz_sign(v: &mut Vec<Self>) {
        let mut next = [0; 256];
        next['-' as usize] = v.len() + 1;
        next['+' as usize] = v.len() + 1;
        v.push(Self {
            next,
            accum: 0,
            shift_factor: 0,
            visited: false,
            i: 0,
            chars: [0; 100],
        })
    }

    fn tz_value(v: &mut Vec<Self>, comma_off: Option<usize>) {
        let mut next = [0; 256];
        next['0' as usize] = v.len();
        next['1' as usize] = v.len();
        next['2' as usize] = v.len();
        next['3' as usize] = v.len();
        next['4' as usize] = v.len();
        next['5' as usize] = v.len();
        next['6' as usize] = v.len();
        next['7' as usize] = v.len();
        next['8' as usize] = v.len();
        next['9' as usize] = v.len();
        if let Some(x) = comma_off {
            next[',' as usize] = v.len() + x;
        }
        v.push(Self {
            next,
            accum: 0,
            shift_factor: 10,
            visited: false,
            i: 0,
            chars: [0; 100],
        })
    }
}

impl<R: BufRead> Parser<R> {
    fn new(r: R, default_tz: time::UtcOffset) -> Parser<R> {
        Parser {
            reader: r,
            default_tz,
            line: 1,
            col: 0,
            parse_state: ParseState::new(),
        }
    }

    fn parse_entry(&mut self) -> Result<Option<Entry>, Box<dyn Error>> {
        loop {
            match self.read()? {
                None => return Ok(None),
                Some(b' ' | b'\n') => (),
                Some(c) => {
                    self.parse_state.reset();
                    self.parse_state.add(c);
                    loop {
                        match self.read()? {
                            None | Some(b'\n') => {
                                return self.parse_state.res(|| self.implied_tz())
                            }
                            Some(c) => self.parse_state.add(c),
                        };
                    }
                }
            };
        }
    }

    fn old_parse_entry(&mut self) -> Result<Option<Entry>, Box<dyn Error>> {
        match self.parse_morsel()? {
            Morsel::None => Ok(None),
            Morsel::Note(note) => Ok(Some(Entry::Note(note))),
            Morsel::Time(start, true) => Ok(Some(TimeEntry { start, stop: None }.into())),
            Morsel::Time(start, false) => match (self.parse_time(None))? {
                None => Ok(Some(TimeEntry { start, stop: None }.into())),
                Some((stop, _)) => Ok(Some(
                    TimeEntry {
                        start,
                        stop: Some(stop),
                    }
                    .into(),
                )),
            },
        }
    }

    fn parse_morsel(&mut self) -> Result<Morsel, Box<dyn Error>> {
        loop {
            match self.read()? {
                None => return Ok(Morsel::None),
                Some(b' ') | Some(b'\n') => (),
                Some(b'#') => return Ok(Morsel::Note(self.read_line()?)),
                Some(digit) => {
                    let digit = self.parse_digit(digit)?;
                    let year = 1000 * digit + self.read_number(100)?;
                    let (t, b) = self.parse_time(Some(year))?.unwrap();
                    return Ok(Morsel::Time(t, b));
                }
            }
        }
    }

    fn parse_time(&mut self, year: Option<u16>) -> Result<Option<(Time, bool)>, Box<dyn Error>> {
        let year = match year {
            Some(y) => Some(y),
            None => self.read_year()?,
        };
        match year {
            None => Ok(None),
            Some(year) => {
                self.read_expected(b'-')?;
                let month = self.read_number(10)? as u8;
                self.read_expected(b'-')?;
                let day = self.read_number(10)? as u8;
                self.read_expected(b' ')?;
                let hour = self.read_number(10)? as u8;
                self.read_expected(b':')?;
                let minute = self.read_number(10)? as u8;

                match self.read()? {
                    // EOF or EOL.
                    None | Some(b'\n') => Ok(Some((
                        Time::new(year, month, day, hour, minute, self.implied_tz())?,
                        true,
                    ))),
                    // End of current entry.
                    Some(b',') => Ok(Some((
                        Time::new(year, month, day, hour, minute, self.implied_tz())?,
                        false,
                    ))),
                    // TZ follows the space.
                    Some(b' ') => {
                        let sign: i16 = match self.read()? {
                            Some(b'-') => -1,
                            Some(b'+') => 1,
                            None => return Err(self.error("expected +/- but got EOF".to_string())),
                            Some(x) => {
                                return Err(
                                    self.error(format!("expected +/- but got '{}'", x as char))
                                )
                            }
                        };
                        let hr_off = self.read_number(10)? as i16;
                        let min_off = self.read_number(10)? as i16;

                        let total_min_off = sign * ((hr_off * 60) + min_off);
                        let tz = TZ::Known(time::UtcOffset::minutes(total_min_off));
                        let res = Time::new(year, month, day, hour, minute, tz)?;

                        match self.read()? {
                            // EOF or EOL.
                            None | Some(b'\n') => Ok(Some((res, true))),
                            // End of current entry.
                            Some(b',') => Ok(Some((res, false))),
                            // Anything else.
                            Some(x) => {
                                Err(self.error(format!("expected '\\n' but got '{}'", x as char)))
                            }
                        }
                    }
                    // Anything else.
                    Some(x) => Err(self.error(format!(
                        "expected newline, comma, or space, but got '{}'",
                        x as char
                    ))),
                }
            }
        }
    }

    fn implied_tz(&self) -> TZ {
        TZ::Implied(self.default_tz)
    }

    fn read_year(&mut self) -> Result<Option<u16>, Box<dyn Error>> {
        loop {
            match self.read()? {
                None => return Ok(None),
                Some(b' ') | Some(b'\n') => (),
                Some(digit) => {
                    let digit = self.parse_digit(digit)?;
                    let year = 1000 * digit + self.read_number(100)?;
                    return Ok(Some(year));
                }
            }
        }
    }

    fn read_line(&mut self) -> Result<String, Box<dyn Error>> {
        let mut s = String::new();
        self.reader.read_line(&mut s)?;
        if let Some('\n') = s.chars().last() {
            s.pop();
        }
        Ok(s)
    }

    fn read_expected(&mut self, expected: u8) -> Result<(), Box<dyn Error>> {
        match self.read()? {
            None => Err(self.error(format!("expected '{}' but got EOF", expected as char))),
            Some(x) => {
                if x == expected {
                    Ok(())
                } else {
                    Err(self.error(format!(
                        "expected '{}' but got '{}'",
                        expected as char, x as char
                    )))
                }
            }
        }
    }

    fn read_number(&mut self, scale: u16) -> Result<u16, Box<dyn Error>> {
        match self.read()? {
            None => Err(self.error("expected a digit but got EOF".to_string())),
            Some(digit) => {
                let digit = self.parse_digit(digit)?;
                match scale {
                    1 => Ok(digit),
                    _ => Ok(digit * scale + self.read_number(scale / 10)?),
                }
            }
        }
    }

    fn read(&mut self) -> io::Result<Option<u8>> {
        let mut buf = [0; 1];
        let len = self.reader.read(&mut buf)?;
        if len == 0 {
            return Ok(None);
        }
        let c = buf[0];
        if c == b'\n' {
            self.line += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }
        Ok(Some(c))
    }

    fn parse_digit(&self, digit: u8) -> Result<u16, Box<dyn Error>> {
        match digit {
            b'0' | b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' | b'8' | b'9' => {
                Ok((digit - b'0') as u16)
            }
            x => Err(self.error(format!("expected a digit but got '{}'", x as char))),
        }
    }

    fn error(&self, message: String) -> Box<dyn Error> {
        let message = format!("line {}, col {}: {}", self.line, self.col, message);
        Box::new(ParseError { message })
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_entries, write_entries, Time, TimeEntry, TZ};
    use crate::{entry::Entry, timesource::real_time::DefaultTimeSource};

    type TestRes = Result<(), Box<dyn std::error::Error>>;

    fn mktime(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        offset: Option<i16>,
    ) -> Result<Time, Box<dyn std::error::Error>> {
        let tz = TZ::from(offset, &DefaultTimeSource);
        Time::new(year, month, day, hour, minute, tz)
    }

    #[test]
    fn test_empty() -> TestRes {
        let actual = parse_entries("".as_bytes(), &DefaultTimeSource)?;
        assert_eq!(Vec::<Entry>::new(), actual);
        Ok(())
    }

    #[test]
    fn test_start_no_tz() -> TestRes {
        let actual = parse_entries("2020-01-02 12:34\n".as_bytes(), &DefaultTimeSource)?;
        assert_eq!(
            vec![Entry::Time(TimeEntry {
                start: mktime(2020, 1, 2, 12, 34, None)?,
                stop: None,
            })],
            actual
        );
        Ok(())
    }

    #[test]
    fn test_start_with_neg_tz() -> TestRes {
        let actual = parse_entries("2020-01-02 12:34 -1001\n".as_bytes(), &DefaultTimeSource)?;
        assert_eq!(
            vec![Entry::Time(TimeEntry {
                start: mktime(2020, 1, 2, 12, 34, Some(-601))?,
                stop: None,
            })],
            actual
        );
        Ok(())
    }

    #[test]
    fn test_start_with_pos_tz_and_comma() -> TestRes {
        let actual = parse_entries("2020-01-02 12:34 +1001,\n".as_bytes(), &DefaultTimeSource)?;
        assert_eq!(
            vec![Entry::Time(TimeEntry {
                start: mktime(2020, 1, 2, 12, 34, Some(601))?,
                stop: None,
            })],
            actual
        );
        Ok(())
    }

    #[test]
    fn test_single_entry() -> TestRes {
        let actual = parse_entries(
            "2020-01-02 12:34 -0400,2020-01-02 13:34 -0400\n".as_bytes(),
            &DefaultTimeSource,
        )?;
        assert_eq!(
            vec![Entry::Time(TimeEntry {
                start: mktime(2020, 1, 2, 12, 34, Some(-240))?,
                stop: Some(mktime(2020, 1, 2, 13, 34, Some(-240))?),
            })],
            actual
        );
        Ok(())
    }

    #[test]
    fn test_single_entry_mixed_tz_and_space_between_times() -> TestRes {
        let actual = parse_entries(
            "2020-01-02 12:34 +1000,   2020-01-02 13:34 -0400\n".as_bytes(),
            &DefaultTimeSource,
        )?;
        assert_eq!(
            vec![Entry::Time(TimeEntry {
                start: mktime(2020, 1, 2, 12, 34, Some(600))?,
                stop: Some(mktime(2020, 1, 2, 13, 34, Some(-240))?),
            })],
            actual
        );
        Ok(())
    }

    #[test]
    fn test_several_entries() -> TestRes {
        let original = "2020-01-02 12:34 +1000,2020-01-02 13:34 -0400\n\
                        2020-01-03 09:00,2020-01-03 10:30\n\
                        2020-02-02 11:11,2020-02-02 12:12 -0400\n";
        let actual = parse_entries(original.as_bytes(), &DefaultTimeSource)?;
        assert_eq!(3, actual.len());

        // Verify that it round-trips.
        let mut output = Vec::new();
        write_entries(&mut output, &actual)?;
        assert_eq!(std::str::from_utf8(&output)?, original);
        Ok(())
    }

    #[test]
    fn test_several_entries_last_is_partial() -> TestRes {
        let original = "2020-01-02 12:34 +1000,2020-01-02 13:34 -0400\n\
                        2020-01-03 09:00,2020-01-03 10:30\n\
                        2020-02-02 11:11\n";
        let actual = parse_entries(original.as_bytes(), &DefaultTimeSource)?;
        assert_eq!(3, actual.len());
        assert_eq!(
            &TimeEntry {
                start: mktime(2020, 2, 2, 11, 11, None)?,
                stop: None,
            },
            actual[2].time()
        );

        // Verify that it round-trips.
        let mut output = Vec::new();
        write_entries(&mut output, &actual)?;
        assert_eq!(std::str::from_utf8(&output)?, original);
        Ok(())
    }

    #[test]
    fn test_annotations() -> TestRes {
        let original = "# first\n\
                        2020-01-02 12:34 -0400,2020-01-02 13:34 -0400\n\
                        # and another one \n\
                        2020-01-03 12:34 -0400,2020-01-03 14:00 -0400\n\
                        # lastly blah blah; no newline, oh the nerve!";
        let actual = parse_entries(original.as_bytes(), &DefaultTimeSource)?;
        assert_eq!(
            vec![
                Entry::note(" first"),
                Entry::Time(TimeEntry {
                    start: mktime(2020, 1, 2, 12, 34, Some(-240))?,
                    stop: Some(mktime(2020, 1, 2, 13, 34, Some(-240))?),
                }),
                Entry::note(" and another one "),
                Entry::Time(TimeEntry {
                    start: mktime(2020, 1, 3, 12, 34, Some(-240))?,
                    stop: Some(mktime(2020, 1, 3, 14, 0, Some(-240))?),
                }),
                Entry::note(" lastly blah blah; no newline, oh the nerve!"),
            ],
            actual
        );
        Ok(())
    }

    // TODO - tests for errors?
}
