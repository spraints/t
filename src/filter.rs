use crate::entry::Entry;

pub fn filter_entries(
    entries: Vec<Entry>,
    filters: Vec<String>,
) -> Result<Vec<Entry>, &'static str> {
    let filter = build_filter(filters)?;
    Ok(entries.into_iter().filter(|e| filter.filter(e)).collect())
}

fn build_filter(filters: Vec<String>) -> Result<Filter, &'static str> {
    match filters.len() {
        0 => Ok(Filter::NoFilter),
        1 => {
            filters[0]
                .split("-")
                .fold(Ok(Filter::NoFilter), |res, part| {
                    res.and_then(|v| v.add_part(part))
                })
            /*
            let mut filter = Filter::NoFilter;
            for part in filters[0].split("-") {
                filter = filter.add_part(part)?;
            }
            Ok(filter)
            */
        }
        _ => Err("only YYYY or YYYY-MM is supported"),
    }
}

enum Filter {
    NoFilter,
    YearFilter(i32),
    YearMonthFilter(i32, u8),
}

impl Filter {
    fn filter(&self, entry: &Entry) -> bool {
        match self {
            Filter::NoFilter => true,
            Filter::YearFilter(year) => entry.includes_year(*year),
            Filter::YearMonthFilter(year, month) => entry.includes_year_month(*year, *month),
        }
    }

    fn add_part(self, part: &str) -> Result<Self, &'static str> {
        match self {
            Filter::NoFilter => part
                .parse()
                .and_then(|y| Ok(Filter::YearFilter(y)))
                .or(Err("couldn't parse year")),
            Filter::YearFilter(y) => part
                .parse()
                .and_then(|m| Ok(Filter::YearMonthFilter(y, m)))
                .or(Err("couldn't parse month")),
            Filter::YearMonthFilter(_, _) => Err("only YYYY or YYYY-MM filters are supported"),
        }
    }
}
