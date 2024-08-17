use crate::entry::TimeEntry;

pub fn filter_entries(
    entries: Vec<TimeEntry>,
    filters: Vec<String>,
) -> Result<Vec<TimeEntry>, &'static str> {
    let filter = build_filter(filters)?;
    Ok(entries.into_iter().filter(|e| filter.filter(e)).collect())
}

fn build_filter(filters: Vec<String>) -> Result<Filter, &'static str> {
    match filters.len() {
        0 => Ok(Filter::None),
        1 => {
            filters[0]
                .split('-')
                .try_fold(Filter::None, |v, part| v.add_part(part))
            /*
            let mut filter = Filter::None;
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
    None,
    ByYear(i32),
    ByYearMonth(i32, u8),
}

impl Filter {
    fn filter(&self, entry: &TimeEntry) -> bool {
        match self {
            Filter::None => true,
            Filter::ByYear(year) => entry.includes_year(*year),
            Filter::ByYearMonth(year, month) => entry.includes_year_month(*year, *month),
        }
    }

    fn add_part(self, part: &str) -> Result<Self, &'static str> {
        match self {
            Filter::None => part
                .parse()
                .map(Filter::ByYear)
                .or(Err("couldn't parse year")),
            Filter::ByYear(y) => part
                .parse()
                .map(|m| Filter::ByYearMonth(y, m))
                .or(Err("couldn't parse month")),
            Filter::ByYearMonth(_, _) => Err("only YYYY or YYYY-MM filters are supported"),
        }
    }
}
