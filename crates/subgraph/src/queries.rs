pub const TOTALS_QUERY: &str = "query Totals {\n  totals {\n    tokens\n    orders\n    traders\n    settlements\n    volumeUsd\n    volumeEth\n    feesUsd\n    feesEth\n  }\n}";

pub const LAST_DAYS_VOLUME_QUERY: &str = "query LastDaysVolume($days: Int!) {\n  dailyTotals(orderBy: timestamp, orderDirection: desc, first: $days) {\n    timestamp\n    volumeUsd\n  }\n}";

pub const LAST_HOURS_VOLUME_QUERY: &str = "query LastHoursVolume($hours: Int!) {\n  hourlyTotals(orderBy: timestamp, orderDirection: desc, first: $hours) {\n    timestamp\n    volumeUsd\n  }\n}";
