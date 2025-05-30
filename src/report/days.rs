use crate::entry::TimeEntry;
use crate::iter::{each_day_in_week, each_week};
use crate::timesource::TimeSource;
use std::fmt::{self, Display, Formatter};
use time::{Date, Duration};

#[derive(Debug, PartialEq)]
pub struct Report {
    years: Vec<Year>,
    opts: Options,
}

#[derive(Debug, PartialEq)]
struct ReportData {
    years: Vec<Year>,
}

#[derive(Debug, PartialEq)]
struct Year {
    year: i32,
    months: Vec<Month>,
}

#[derive(Debug, PartialEq)]
struct Month {
    month: u8,
    weeks: Vec<Week>,
}

#[derive(Debug, PartialEq)]
struct Week {
    start: Date,
    minutes: [i64; 7],
}

struct State {
    report: ReportData,
    year: Year,
    month: Month,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Options {
    pub include_totals: bool,
    pub only_show_per_year: bool,
}

impl Options {
    fn show_weekly_total(&self) -> bool {
        !self.only_show_per_year
    }
    fn show_monthly_total(&self) -> bool {
        self.include_totals && !self.only_show_per_year
    }
    fn show_yearly_total(&self) -> bool {
        self.include_totals || self.only_show_per_year
    }
}

pub fn prepare<TS: TimeSource>(entries: Vec<TimeEntry>, ts: &TS, opts: Options) -> Report {
    let mut state = None;
    for (week_start, entries) in each_week(entries, ts) {
        state = Some(prepare_week(state, week_start, entries, ts));
    }
    finish(state, opts)
}

fn prepare_week<TS: TimeSource>(
    state: Option<State>,
    week_start: Date,
    entries: Vec<TimeEntry>,
    ts: &TS,
) -> State {
    let week = convert_week(week_start, entries, ts);
    match state {
        None => {
            let month = Month {
                month: week_start.month(),
                weeks: vec![week],
            };
            let year = Year {
                year: week_start.year(),
                months: vec![],
            };
            State {
                report: ReportData { years: vec![] },
                year,
                month,
            }
        }
        Some(State {
            mut report,
            mut year,
            mut month,
        }) => {
            if year.year != week_start.year() {
                year.months.push(month);
                report.years.push(year);
                let month = Month {
                    month: week_start.month(),
                    weeks: vec![week],
                };
                let year = Year {
                    year: week_start.year(),
                    months: vec![],
                };
                State {
                    report,
                    year,
                    month,
                }
            } else if month.month != week_start.month() {
                year.months.push(month);
                let month = Month {
                    month: week_start.month(),
                    weeks: vec![week],
                };
                State {
                    report,
                    year,
                    month,
                }
            } else {
                month.weeks.push(week);
                State {
                    report,
                    year,
                    month,
                }
            }
        }
    }
}

fn convert_week<TS: TimeSource>(start: Date, entries: Vec<TimeEntry>, ts: &TS) -> Week {
    let mut minutes = [0; 7];
    for (day_start, entries) in each_day_in_week(entries, start, ts) {
        let i = (day_start - start).whole_days();
        minutes[i as usize] = minutes_on_day(day_start, entries, ts);
    }
    Week { start, minutes }
}

fn minutes_on_day<TS: TimeSource>(start: Date, entries: Vec<TimeEntry>, ts: &TS) -> i64 {
    let stop = start.next_day().midnight().assume_offset(ts.local_offset());
    let start = start.midnight().assume_offset(ts.local_offset());
    entries
        .iter()
        .fold(0, |sum, entry| sum + entry.minutes_between(start, stop))
}

fn finish(state: Option<State>, opts: Options) -> Report {
    let years = match state {
        Some(State {
            mut report,
            mut year,
            month,
        }) => {
            year.months.push(month);
            report.years.push(year);
            report.years
        }
        None => vec![],
    };
    Report { years, opts }
}

#[cfg(test)]
mod tests {
    use super::{prepare, Month, Options, Report, Week, Year};
    use crate::entry::TimeEntry;
    use crate::parser::parse_time_entries;
    use crate::timesource::mock_time::mock_time;
    use crate::timesource::real_time::DefaultTimeSource;
    use pretty_assertions::assert_eq;
    use time::{date, offset, time};

    type TestRes = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn test_empty() {
        let entries: Vec<TimeEntry> = vec![];
        assert_eq!(
            prepare(
                entries,
                &DefaultTimeSource,
                Options {
                    include_totals: true,
                    only_show_per_year: false
                }
            ),
            Report {
                opts: Options {
                    include_totals: true,
                    only_show_per_year: false
                },
                years: vec![]
            }
        );
    }

    #[test]
    fn test_single_entry() -> TestRes {
        let input = "2013-09-04 11:04,2013-09-04 12:24\n";
        let entries = parse_time_entries(input.as_bytes(), &DefaultTimeSource)?;
        assert_eq!(
            prepare(
                entries,
                &DefaultTimeSource,
                Options {
                    include_totals: true,
                    only_show_per_year: false
                }
            ),
            Report {
                opts: Options {
                    include_totals: true,
                    only_show_per_year: false
                },
                years: vec![Year {
                    year: 2013,
                    months: vec![Month {
                        month: 9,
                        weeks: vec![Week {
                            start: date!(2013 - 09 - 01),
                            minutes: [0, 0, 0, 80, 0, 0, 0],
                        }]
                    }]
                }]
            }
        );
        Ok(())
    }

    #[test]
    fn test_entry_covering_now() -> TestRes {
        let ts = mock_time(date!(2013 - 09 - 04), time!(12:00), offset!(-04:00));
        let input = "2013-09-04 11:04 -0400";
        let entries = parse_time_entries(input.as_bytes(), &ts)?;
        assert_eq!(
            prepare(
                entries,
                &ts,
                Options {
                    include_totals: true,
                    only_show_per_year: false
                }
            ),
            Report {
                opts: Options {
                    include_totals: true,
                    only_show_per_year: false
                },
                years: vec![Year {
                    year: 2013,
                    months: vec![Month {
                        month: 9,
                        weeks: vec![Week {
                            start: date!(2013 - 09 - 01),
                            minutes: [0, 0, 0, 56, 0, 0, 0]
                        },]
                    },]
                },]
            }
        );
        Ok(())
    }

    #[test]
    fn test_entry_spanning_weekend() -> TestRes {
        let input = "2013-11-16 00:00,2013-11-17 09:20";
        let entries = parse_time_entries(input.as_bytes(), &DefaultTimeSource)?;
        assert_eq!(
            prepare(
                entries,
                &DefaultTimeSource,
                Options {
                    include_totals: false,
                    only_show_per_year: false
                }
            ),
            Report {
                opts: Options {
                    include_totals: false,
                    only_show_per_year: false
                },
                years: vec![Year {
                    year: 2013,
                    months: vec![Month {
                        month: 11,
                        weeks: vec![
                            Week {
                                start: date!(2013 - 11 - 10),
                                minutes: [0, 0, 0, 0, 0, 0, 1440]
                            },
                            Week {
                                start: date!(2013 - 11 - 17),
                                minutes: [560, 0, 0, 0, 0, 0, 0]
                            },
                        ]
                    },]
                },]
            }
        );
        Ok(())
    }

    #[test]
    fn test_spans_days_and_years() -> TestRes {
        let input = "2015-12-01 10:45,2015-12-01 11:15\n\
                     2015-12-02 10:15,2015-12-02 10:44\n\
                     2015-12-11 10:45,2015-12-11 11:46\n\
                     2015-12-22 10:45,2015-12-22 11:47\n\
                     2015-12-22 23:49,2015-12-23 00:02\n\
                     2015-12-31 10:45,2015-12-31 11:48\n\
                     2016-01-04 10:45,2016-01-04 11:04\n\
                     2016-01-04 11:04,2016-01-04 11:16\n\
                     2016-01-04 11:16,2016-01-04 11:26\n\
                     2016-01-05 11:26,2016-01-05 11:39\n\
                     2016-01-05 11:39,2016-01-05 11:49\n";
        let entries = parse_time_entries(input.as_bytes(), &DefaultTimeSource)?;
        assert_eq!(
            prepare(
                entries,
                &DefaultTimeSource,
                Options {
                    include_totals: true,
                    only_show_per_year: false
                }
            ),
            Report {
                opts: Options {
                    include_totals: true,
                    only_show_per_year: false
                },
                years: vec![
                    Year {
                        year: 2015,
                        months: vec![
                            Month {
                                month: 11,
                                weeks: vec![Week {
                                    start: date!(2015 - 11 - 29),
                                    minutes: [0, 0, 30, 29, 0, 0, 0]
                                },]
                            },
                            Month {
                                month: 12,
                                weeks: vec![
                                    Week {
                                        start: date!(2015 - 12 - 06),
                                        minutes: [0, 0, 0, 0, 0, 61, 0]
                                    },
                                    Week {
                                        start: date!(2015 - 12 - 13),
                                        minutes: [0, 0, 0, 0, 0, 0, 0]
                                    },
                                    Week {
                                        start: date!(2015 - 12 - 20),
                                        minutes: [0, 0, 73, 2, 0, 0, 0]
                                    },
                                    Week {
                                        start: date!(2015 - 12 - 27),
                                        minutes: [0, 0, 0, 0, 63, 0, 0]
                                    },
                                ]
                            },
                        ]
                    },
                    Year {
                        year: 2016,
                        months: vec![Month {
                            month: 1,
                            weeks: vec![Week {
                                start: date!(2016 - 01 - 03),
                                minutes: [0, 41, 23, 0, 0, 0, 0]
                            },]
                        },]
                    }
                ]
            }
        );
        Ok(())
    }
}

const SIX_DAYS: Duration = Duration::days(6);

impl Display for Report {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for year in &self.years {
            let mut year_tot = [0; 7];
            for month in &year.months {
                let mut month_tot = [0; 7];
                for week in &month.weeks {
                    if self.opts.show_weekly_total() {
                        write_report_line(
                            f,
                            format!("{} - {}", week.start, week.start + SIX_DAYS),
                            &week.minutes,
                        )?;
                    }
                    accum(&mut month_tot, &week.minutes);
                }
                if self.opts.show_monthly_total() {
                    write_report_line(
                        f,
                        format!("{:04}-{:02}", year.year, month.month),
                        &month_tot,
                    )?;
                }
                accum(&mut year_tot, &month_tot);
            }
            if self.opts.show_yearly_total() {
                write_report_line(f, format!("{:04}", year.year), &year_tot)?;
            }
        }
        Ok(())
    }
}

fn accum(a: &mut [i64; 7], b: &[i64; 7]) {
    for i in 0..7 {
        a[i] += b[i];
    }
}

fn write_report_line(f: &mut Formatter<'_>, label: String, minutes: &[i64; 7]) -> fmt::Result {
    write!(f, "{:23} |", label)?;
    let mut tot = 0;
    for min in minutes {
        if *min > 0 {
            write!(f, "| {:5} ", min)?;
            tot += min;
        } else {
            write!(f, "|       ")?;
        }
    }
    writeln!(f, "|| {:6}", tot)
}
