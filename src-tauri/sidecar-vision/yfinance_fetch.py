import json
import sys
import traceback

import yfinance as yf


def fetch_ohlcv(ticker: str, period: str, interval: str) -> dict:
    try:
        stock = yf.Ticker(ticker)
        df = stock.history(period=period, interval=interval)

        if df.empty:
            return {
                "success": False,
                "data": None,
                "error": f"No data returned for {ticker}",
            }

        bars = []
        for idx, row in df.iterrows():
            bars.append(
                {
                    "timestamp": int(idx.timestamp()),
                    "open": float(row["Open"]),
                    "high": float(row["High"]),
                    "low": float(row["Low"]),
                    "close": float(row["Close"]),
                    "volume": int(row["Volume"]),
                }
            )

        return {"success": True, "data": bars, "error": None}
    except Exception as e:
        return {"success": False, "data": None, "error": str(e)}


def main() -> None:
    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            request = json.loads(line)
            result = fetch_ohlcv(
                request["ticker"], request["period"], request["interval"]
            )
            print(json.dumps(result), flush=True)
        except Exception:
            error_result = {
                "success": False,
                "data": None,
                "error": traceback.format_exc(),
            }
            print(json.dumps(error_result), flush=True)


if __name__ == "__main__":
    main()
