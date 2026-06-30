// ── United States ──────────────────────────────────────────────
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

// ── Europe ────────────────────────────────────────────────────
pub const FTSE100: &[&str] = &[
    "SHEL.L", "AZN.L", "HSBA.L", "ULVR.L", "GSK.L", "BP.L", "VOD.L", "BARC.L",
    "BATS.L", "LSEG.L", "NG.L", "RIO.L", "GLEN.L", "AAL.L", "REL.L",
    "DGE.L", "SVT.L", "CPRT.L", "LAND.L", "NXT.L",
];

pub const DAX40: &[&str] = &[
    "SAP.DE", "SIE.DE", "ALV.DE", "BAS.DE", "BMW.DE", "MBG.DE", "IFX.DE",
    "SRT3.DE", "HEN3.DE", "DTE.DE", "EOAN.DE", "VNA.DE", "ZAL.DE",
    "1COV.DE", "DBK.DE", "MUV2.DE", "HNR1.DE", "RHM.DE", "CON.DE", "PAH3.DE",
];

pub const CAC40: &[&str] = &[
    "AI.PA", "MC.PA", "TTE.PA", "SAN.PA", "BNP.PA", "RMS.PA", "OR.PA", "CS.PA",
    "VIV.PA", "SAF.PA", "EL.PA", "DG.PA", "ENGI.PA", "GLE.PA", "RI.PA",
    "KER.PA", "STLAP.PA", "FR.PA", "VIE.PA", "SW.PA",
];

pub const EURO_STOXX50: &[&str] = &[
    "SAP.DE", "ASML.AS", "MC.PA", "SIE.DE", "SHEL.L", "AZN.L", "TTE.PA",
    "SAN.MC", "ABI.BR", "ADS.DE", "AIR.PA", "ALV.DE", "ASML.AS", "BAS.DE",
    "BMW.DE", "BNP.PA", "ENEL.MI", "GLEN.L", "IBE.MC", "INFN.MI",
];

// ── Asia ──────────────────────────────────────────────────────
pub const NIKKEI225: &[&str] = &[
    "7203.T", "6758.T", "8306.T", "9984.T", "6501.T", "8035.T", "7974.T",
    "6861.T", "8411.T", "9433.T", "6954.T", "6367.T", "7751.T", "5020.T",
    "6178.T", "8316.T", "7267.T", "4063.T", "6988.T", "8001.T",
];

pub const HANG_SENG: &[&str] = &[
    "0005.HK", "0941.HK", "1398.HK", "0883.HK", "2318.HK", "0001.HK",
    "0027.HK", "0066.HK", "1299.HK", "2388.HK", "0388.HK", "0002.HK",
    "0003.HK", "0016.HK", "0011.HK", "0012.HK", "0006.HK", "0017.HK",
];

pub const SSE_COMPOSITE: &[&str] = &[
    "600519.SS", "601318.SS", "600036.SS", "601166.SS", "600900.SS",
    "601398.SS", "600276.SS", "000858.SZ", "601888.SS", "600030.SS",
    "601012.SS", "600887.SS", "601668.SS", "600309.SS", "601601.SS",
];

pub const KOSPI: &[&str] = &[
    "005930.KS", "000660.KS", "035420.KS", "051910.KS", "028260.KS",
    "006400.KS", "012330.KS", "055550.KS", "032640.KS", "000270.KS",
    "068270.KS", "017670.KS", "090430.KS", "326400.KS", "009830.KS",
];

pub const TWSE: &[&str] = &[
    "2330.TW", "2317.TW", "2454.TW", "2308.TW", "2881.TW",
    "2882.TW", "2891.TW", "2303.TW", "3711.TW", "2002.TW",
    "1301.TW", "1303.TW", "2412.TW", "2886.TW", "5880.TW",
];

// ── Americas (non-US) ─────────────────────────────────────────
pub const TSX60: &[&str] = &[
    "RY.TO", "TD.TO", "BNS.TO", "BMO.TO", "CM.TO", "MFC.TO",
    "ENB.TO", "TRP.TO", "CNQ.TO", "SU.TO", "IMO.TO", "CVE.TO",
    "ABX.TO", "NTR.TO", "FNV.TO", "AEM.TO", "WPM.TO", "COST.TO",
    "ATD.TO", "WCN.TO",
];

pub const BOVESPA: &[&str] = &[
    "PETR4.SA", "VALE3.SA", "ITUB4.SA", "BBDC4.SA", "BBAS3.SA",
    "ABEV3.SA", "WEGE3.SA", "RENT3.SA", "SUZB3.SA", "JBSS3.SA",
    "MGLU3.SA", "PRIO3.SA", "RADL3.SA", "HAPV3.SA", "CSAN3.SA",
];

pub const IPC_MEXICO: &[&str] = &[
    "AMXB.MX", "CEMEXCPO.MX", "WALMEX.MX", "FEMSAUBD.MX", "GFNORTEO.MX",
    "GAPB.MX", "GMEXICOB.MX", "ASURB.MX", "BIMBOA.MX", "ELEKTRA.MX",
];

// ── Middle East & Africa ──────────────────────────────────────
pub const TADAWUL: &[&str] = &[
    "2222.SR", "1120.SR", "7010.SR", "2010.SR", "1180.SR",
    "4200.SR", "1211.SR", "2290.SR", "4001.SR", "1304.SR",
];

pub const BIST100: &[&str] = &[
    "THYAO.IS", "GARAN.IS", "ASELS.IS", "EREGL.IS", "KCHOL.IS",
    "SISE.IS", "TUPRS.IS", "FROTO.IS", "TCELL.IS", "AKBNK.IS",
    "YKBNK.IS", "HALKB.IS", "SAHOL.IS", "PGSUS.IS", "TOASO.IS",
];

// ── Oceania ───────────────────────────────────────────────────
pub const ASX200: &[&str] = &[
    "BHP.AX", "CBA.AX", "CSL.AX", "NAB.AX", "WBC.AX",
    "ANZ.AX", "WDS.AX", "FMG.AX", "MQG.AX", "TLS.AX",
    "WES.AX", "WOW.AX", "RIO.AX", "STO.AX", "COL.AX",
];

// ── Global Major ETFs ─────────────────────────────────────────
pub const GLOBAL_ETFS: &[&str] = &[
    "SPY", "QQQ", "DIA", "IWM", "EEM", "EFA", "VGK", "FXI", "EWJ",
    "GLD", "SLV", "USO", "UNG", "TLT", "HYG", "LQD", "VIX",
];

// ── Lookup ────────────────────────────────────────────────────
pub fn get_index_symbols(name: &str) -> Option<Vec<String>> {
    let slice = match name.to_lowercase().as_str() {
        // US
        "sp500" | "s&p500" | "s&p 500" => SP500,
        "nasdaq100" | "nasdaq-100" => NASDAQ100,
        "dow" | "dowjones" | "dow jones" => DOW_JONES,
        // Europe
        "ftse100" | "ftse-100" | "ftse 100" => FTSE100,
        "dax40" | "dax-40" | "dax" => DAX40,
        "cac40" | "cac-40" | "cac" => CAC40,
        "eurostoxx50" | "euro-stoxx-50" | "eurostoxx" => EURO_STOXX50,
        // Asia
        "nikkei225" | "nikkei-225" | "nikkei" => NIKKEI225,
        "hangseng" | "hang-seng" => HANG_SENG,
        "sse" | "shanghai" | "sse-composite" => SSE_COMPOSITE,
        "kospi" => KOSPI,
        "twse" | "taiex" => TWSE,
        // Americas
        "tsx60" | "tsx" | "tsx-60" => TSX60,
        "bovespa" | "bovespa50" => BOVESPA,
        "ipc" | "ipc-mexico" => IPC_MEXICO,
        // Middle East & Africa
        "tadawul" | "saudi" => TADAWUL,
        "bist100" | "bist" | "borsa-istanbul" => BIST100,
        // Oceania
        "asx200" | "asx" | "asx-200" => ASX200,
        // Global
        "etfs" | "global-etfs" => GLOBAL_ETFS,
        _ => return None,
    };
    Some(slice.iter().map(|s| s.to_string()).collect())
}

pub fn list_indices() -> Vec<(&'static str, &'static str)> {
    vec![
        // US
        ("sp500", "S&P 500 (US)"),
        ("nasdaq100", "NASDAQ-100 (US)"),
        ("dowjones", "Dow Jones 30 (US)"),
        // Europe
        ("ftse100", "FTSE 100 (UK)"),
        ("dax40", "DAX 40 (Germany)"),
        ("cac40", "CAC 40 (France)"),
        ("eurostoxx50", "Euro Stoxx 50 (EU)"),
        // Asia
        ("nikkei225", "Nikkei 225 (Japan)"),
        ("hangseng", "Hang Seng (Hong Kong)"),
        ("sse", "SSE Composite (China)"),
        ("kospi", "KOSPI (South Korea)"),
        ("twse", "TAIEX (Taiwan)"),
        // Americas
        ("tsx60", "S&P/TSX 60 (Canada)"),
        ("bovespa", "Bovespa 50 (Brazil)"),
        ("ipc", "IPC (Mexico)"),
        // Middle East & Africa
        ("tadawul", "Tadawul (Saudi Arabia)"),
        ("bist100", "BIST 100 (Turkey)"),
        // Oceania
        ("asx200", "ASX 200 (Australia)"),
        // Global
        ("etfs", "Global Major ETFs"),
    ]
}
