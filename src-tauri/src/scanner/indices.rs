pub const SP500: &[&str] = &[
    "AAPL", "MSFT", "AMZN", "NVDA", "GOOGL", "META", "TSLA", "BRK.B", "UNH", "JNJ",
    "XOM", "JPM", "V", "PG", "HD", "MA", "CVX", "LLY", "ABBV", "PEP",
    "MRK", "KO", "PFE", "BAC", "AVGO", "TMO", "COST", "DIS", "CSCO", "ABT",
    "WMT", "MCD", "ACN", "VZ", "NEE", "CRM", "TXN", "LIN", "PM", "ADBE",
    "BMY", "RTX", "DHR", "HON", "UNP", "NKE", "LOW", "UPS", "ORCL", "QCOM",
];

pub const NASDAQ100: &[&str] = &[
    "AAPL", "MSFT", "AMZN", "NVDA", "GOOGL", "META", "TSLA", "GOOG", "AVGO", "PEP",
    "COST", "CSCO", "TMUS", "ADBE", "TXN", "CMCSA", "AMD", "NFLX", "QCOM", "AMGN",
    "INTU", "HON", "AMAT", "SBUX", "INTC", "GILD", "MDLZ", "ADI", "ADP", "PYPL",
    "REGN", "VRTX", "MRNA", "LRCX", "PANW", "SNPS", "CDNS", "ASML", "MU", "KLAC",
];

pub const DOW_JONES: &[&str] = &[
    "AAPL", "AMGN", "AXP", "BA", "CAT", "CRM", "CSCO", "CVX", "DIS", "DOW",
    "GS", "HD", "HON", "IBM", "INTC", "JNJ", "JPM", "KO", "MCD", "MMM",
    "MRK", "MSFT", "NKE", "PG", "TRV", "UNH", "V", "VZ", "WBA", "WMT",
];

pub fn get_index_symbols(name: &str) -> Option<Vec<String>> {
    let slice = match name.to_lowercase().as_str() {
        "sp500" | "s&p500" | "s&p 500" => SP500,
        "nasdaq100" | "nasdaq-100" => NASDAQ100,
        "dow" | "dowjones" | "dow jones" => DOW_JONES,
        _ => return None,
    };
    Some(slice.iter().map(|s| s.to_string()).collect())
}

pub fn list_indices() -> Vec<(&'static str, &'static str)> {
    vec![
        ("sp500", "S&P 500"),
        ("nasdaq100", "NASDAQ-100"),
        ("dowjones", "Dow Jones 30"),
    ]
}
