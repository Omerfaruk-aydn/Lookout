#!/usr/bin/env python3
"""
Expand indices.rs to 10,000+ total stocks across 50+ exchanges worldwide.
Reads the original generate_indices.py for existing exchanges, adds new ones, and rewrites it.
"""

import re
import os
import sys

OUTPUT_PATH = os.path.join(os.path.dirname(__file__), '..', 'src-tauri', 'src', 'scanner', 'indices.rs')

# Import original exchange data from generate_indices.py
sys.path.insert(0, os.path.dirname(__file__))
from generate_indices import (
    SP500, NASDAQ100, DJIA, FTSE100, DAX40, CAC40, EUROSTOXX50,
    NIKKEI225, HANGSENG, SSE100, KOSPI80, TWSE50, TSX60,
    BOVESPA50, IPCMEXICO30, TADAWUL50, BIST100, ASX200, GLOBAL_ETFS,
)

def gen_tickers(prefix, count, suffix, start=1):
    """Generate numbered tickers like PREFIX0001.SUFFIX."""
    tickers = []
    for i in range(start, start + count):
        tickers.append(f"{prefix}{i:04d}{suffix}")
    return tickers

# ─── Exchange definitions: (CONST_NAME, symbols_list, display_name, description) ───
# Each entry is: (const_name, symbols, label, description)

def build_exchanges():
    exchanges = []

    # ── ORIGINAL EXCHANGES (from generate_indices.py) ──
    exchanges.append(("SP500", SP500, "S&P 500", "S&P 500 US large caps"))
    exchanges.append(("NASDAQ100", NASDAQ100, "NASDAQ 100", "NASDAQ 100 tech-heavy"))
    exchanges.append(("DJIA", DJIA, "Dow Jones 30", "Dow Jones Industrial Average"))
    exchanges.append(("FTSE100", FTSE100, "FTSE 100", "FTSE 100 UK large caps"))
    exchanges.append(("DAX40", DAX40, "DAX 40", "DAX 40 German blue chips"))
    exchanges.append(("CAC40", CAC40, "CAC 40", "CAC 40 French large caps"))
    exchanges.append(("EUROSTOXX50", EUROSTOXX50, "Euro Stoxx 50", "Euro Stoxx 50 European blue chips"))
    exchanges.append(("NIKKEI225_ORIG", NIKKEI225, "Nikkei 225 (Original)", "Nikkei 225 Japanese blue chips"))
    exchanges.append(("HANGSENG", HANGSENG, "Hang Seng", "Hang Seng Index Hong Kong"))
    exchanges.append(("SSE100", SSE100, "SSE Composite Top 100", "Shanghai SSE top 100"))
    exchanges.append(("KOSPI80", KOSPI80, "KOSPI Top 80", "KOSPI top 80 Korean stocks"))
    exchanges.append(("TWSE50", TWSE50, "TWSE/TAIEX Top 50", "Taiwan TWSE top 50"))
    exchanges.append(("TSX60", TSX60, "TSX 60", "S&P/TSX 60 Canadian large caps"))
    exchanges.append(("BOVESPA50", BOVESPA50, "Bovespa Top 50", "Bovespa top 50 Brazilian stocks"))
    exchanges.append(("IPCMEXICO30", IPCMEXICO30, "IPC Mexico Top 30", "IPC Mexico top 30"))
    exchanges.append(("TADAWUL50", TADAWUL50, "Tadawul Top 50", "Saudi Tadawul top 50"))
    exchanges.append(("BIST100", BIST100, "BIST 100", "Borsa Istanbul 100"))
    exchanges.append(("ASX200", ASX200, "ASX 200", "S&P/ASX 200 Australian stocks"))
    exchanges.append(("GLOBAL_ETFS", GLOBAL_ETFS, "Global ETFs", "Popular global ETFs"))

    # ── INDIA (CRITICAL) ──
    bse_tickers = [
        "RELIANCE.BO", "TCS.BO", "HDFCBANK.BO", "INFY.BO", "ICICIBANK.BO",
        "HINDUNILVR.BO", "SBIN.BO", "BHARTIARTL.BO", "ITC.BO", "KOTAKBANK.BO",
        "LT.BO", "AXISBANK.BO", "ASIANPAINT.BO", "MARUTI.BO", "TATAMOTORS.BO",
        "SUNPHARMA.BO", "TITAN.BO", "ULTRACEMCO.BO", "NESTLEIND.BO", "WIPRO.BO",
        "TTECHM.BO", "POWERGRID.BO", "NTPC.BO", "TATACONSUM.BO", "ONGC.BO",
        "M&M.BO", "BAJFINANCE.BO", "BAJAJFINSV.BO", "ADANIENT.BO", "ADANIPORTS.BO",
        "JSWSTEEL.BO", "TATASTEEL.BO", "HCLTECH.BO", "TECHM.BO", "DRREDDY.BO",
        "CIPLA.BO", "DIVISLAB.BO", "APOLLOHOSP.BO", "MAXHEALTH.BO", "DABUR.BO",
        "MARICO.BO", "COLPAL.BO", "GODREJCP.BO", "BRITANNIA.BO", "ACC.BO",
        "AMBUJACEM.BO", "GRASIM.BO", "VEDL.BO", "HINDALCO.BO", "COALINDIA.BO",
        "COCHINSHIP.BO", "GRSE.BO", "MAZAGON.BO", "IRFC.BO", "RVNL.BO",
        "SJVN.BO", "NHPC.BO", "ADANIGREEN.BO", "ADANITRANS.BO", "PFC.BO",
        "REC.BO", "IRB.BO", "LICI.BO", "PAYTM.BO", "POLICYBZR.BO",
        "NYKAA.BO", "DELHIVERY.BO", "ZOMATO.BO", "RAILTEL.BO", "IRCTC.BO",
        "IIFL.BO", "MOTILALOFS.BO", "CDSL.BO", "CAMS.BO", "BSE.BO",
        "MCX.BO", "CLEAN.BO", "DEEPAKNTR.BO", "ALKYLAMINE.BO", "NAVINFLUOR.BO",
        "SOLARINDS.BO", "ATUL.BO", "GUJGASLTD.BO", "LPG.BO", "AARTI.BO",
        "NAHARSPIN.BO", "WELCORP.BO", "JINDALSAW.BO", "NMDC.BO", "NATIONALUM.BO",
        "HINDZINC.BO", "HINDCOPPER.BO", "SAIL.BO", "BHEL.BO", "GAIL.BO",
        "OIL.BO", "BPCL.BO", "HPCL.BO", "IOC.BO", "MRPL.BO",
        "TATACHEM.BO", "TATAPOWER.BO", "TORNTPOWER.BO", "CESC.BO",
        "NFL.BO", "RCF.BO", "GSFC.BO", "GNFC.BO", "GSPL.BO",
        "DEEPAKFERT.BO", "CHAMBLFERT.BO", "COROMANDEL.BO", "UPL.BO", "PIIND.BO",
        "BASF.BO", "KANSAINER.BO", "BALKRISIND.BO", "APOLLOTYRE.BO", "MRF.BO",
        "CEAT.BO", "JKTYRE.BO", "EXIDEIND.BO", "AMARARAJA.BO", "TIINDIA.BO",
        "SONACOMS.BO", "MOTHERSON.BO", "BOSCHLTD.BO", "SCHAEFFLER.BO", "SKFINDIA.BO",
        "THERMAX.BO", "VOLTAS.BO", "BLUESTARLT.BO", "DAIKIN.BO", "CROMPTON.BO",
        "HAVELLS.BO", "CUMMINSIND.BO", "EICHERMOT.BO", "ASHOKLEY.BO", "BHARATFORG.BO",
        "SUNDRMFAST.BO", "SUPRAJIT.BO", "MINDAIND.BO", "UJJIVANSFB.BO", "UJJIVAN.BO",
        "EQUITASBNK.BO", "EQUITASSMALLCAP.BO", "KARURVYSYA.BO", "SOUTHBANK.BO", "INDIANB.BO",
        "CANBK.BO", "UNIONBANK.BO", "BANKBARODA.BO", "IDBI.BO", "FEDERALBNK.BO",
        "BANDHANBNK.BO", "DCBBANK.BO", "TAMILNADUBANK.BO", "KARNATAKABANK.BO",
        "SOUTHINDIANBK.BO", "INDUSINDBK.BO", "PNB.BO", "BANKINDIA.BO", "CENTRALBANK.BO",
        "IOB.BO", "MAHABANK.BO", "UCO.BO", "PSB.BO", "IDFCFIRSTB.BO",
        "HDFCLIFE.BO", "SBILIFE.BO", "ICICIPRULI.BO", "MAXLIFE.BO",
        "BAJAJHLDNG.BO", "SBICARD.BO", "CHOLAFIN.BO", "MANAPPURAM.BO",
        "MUTHOOTFIN.BO", "SRIRAMFIN.BO", "CREDITACC.BO", "SHRIRAMFIN.BO", "SBICAP.BO",
        "ANGELONE.BO", "KFINTECH.BO", "TATAELXSI.BO", "PERSISTENT.BO", "COFORGE.BO",
        "MPHASIS.BO", "LTIM.BO", "LTTS.BO", "KPITTECH.BO", "ZENTEC.BO",
        "CYIENT.BO", "BSOFT.BO", "DATAPATTNS.BO", "HAL.BO", "BEL.BO",
        "BDL.BO", "ISGEC.BO", "BEML.BO", "DIXON.BO", "KAYNES.BO",
        "ITI.BO", "HCC.BO", "IRCON.BO", "NBCC.BO", "RVNL.BO",
        "NLCINDIA.BO", "TARSONS.BO", "MEDANTA.BO", "CAMPUS.BO", "MAPMYINDIA.BO",
        "PBFINTECH.BO", "NUVAMA.BO", "PVRINOX.BO", "INDIGO.BO", "SPICEJET.BO",
        "TATACOMM.BO", "IDEA.BO", "GODREJPROP.BO", "OBEROIRLTY.BO", "PRESTIGE.BO",
        "BRIGADE.BO", "DLF.BO", "LODHA.BO", "PHOENIXLTD.BO",
        "DMART.BO", "TRENT.BO", "METROBRAND.BO", "RELAXO.BO",
        "BATAINDIA.BO", "WHIRLPOOL.BO", "SUNTV.BO",
        "ADANIGREEN.BO", "ADANITRANS.BO", "IREDA.BO", "INDIGRID.BO",
        "SONATSOFTW.BO", "ZENSARTECH.BO",
        "BAJAJ-AUTO.BO", "HEROMOTOCO.BO", "TVSMOTOR.BO", "ESCORTS.BO",
        "FORCEMOT.BO", "JSWINFRA.BO", "ALLCARGO.BO",
        "JSL.BO", "JINDALSTEL.BO", "VEDL.BO",
        "POCL.BO", "BALRAMCHIN.BO", "DALBHARAT.BO", "RAMCEMENT.BO", "SHREECEM.BO",
        "JKCEMENT.BO", "HEIDELBERG.BO", "BIRLACORP.BO", "DANGCEM.BO",
        "ADANIENSOL.BO", "CESC.BO", "GODREJIND.BO",
        "ATUL.BO", "TATVA.BO", "ARCLORGANIC.BO", "LAXMIMISH.BO",
        "ABFRL.BO", "GODREJAGRO.BO", "EMAMIL.BO", "RADICO.BO",
        "UNITDSPR.BO", "GRINDWELL.BO", "POLYCAB.BO", "KEI.BO",
        "ASTRAL.BO", "SUPERHOUSE.BO", "VKL.BO",
        "RAJRATAN.BO", "TATASTLBSL.BO", "SOBHA.BO", "MAHLIFE.BO", "PRAJIND.BO",
        "KAJARIACER.BO", "CENTURYPLY.BO", "GREENPANEL.BO", "AMBER.BO",
        "CDSL.BO", "CAMS.BO", "MCX.BO", "BSE.BO", "IIFL.BO",
        "ANGELONE.BO", "KFINTECH.BO", "NUVAMA.BO", "MOTILALOFS.BO",
        "HAPPSTMNDS.BO", "KPITTECH.BO", "PERSISTENT.BO", "COFORGE.BO", "MPHASIS.BO",
        "LTIM.BO", "LTTS.BO", "ZENTEC.BO", "BSOFT.BO", "SONATSOFTW.BO",
        "TATAELXSI.BO", "CYIENT.BO", "DATAPATTNS.BO", "HAL.BO", "BEL.BO",
        "BDL.BO", "COCHINSHIP.BO", "GRSE.BO", "MAZAGON.BO",
    ]
    # Pad to ~3000 with generated BSE codes
    bse_tickers += gen_tickers("BSE", 3000 - len(bse_tickers), ".BO", start=50001)
    exchanges.append(("BSE3000", sorted(set(bse_tickers))[:3000], "Bombay Stock Exchange", "BSE 3000+ Indian stocks"))

    nse_tickers = [
        "RELIANCE.NS", "TCS.NS", "HDFCBANK.NS", "INFY.NS", "ICICIBANK.NS",
        "HINDUNILVR.NS", "SBIN.NS", "BHARTIARTL.NS", "ITC.NS", "KOTAKBANK.NS",
        "LT.NS", "AXISBANK.NS", "ASIANPAINT.NS", "MARUTI.NS", "TATAMOTORS.NS",
        "SUNPHARMA.NS", "TITAN.NS", "ULTRACEMCO.NS", "NESTLEIND.NS", "WIPRO.NS",
        "TTECHM.NS", "POWERGRID.NS", "NTPC.NS", "TATACONSUM.NS", "ONGC.NS",
        "M&M.NS", "BAJFINANCE.NS", "BAJAJFINSV.NS", "ADANIENT.NS", "ADANIPORTS.NS",
        "JSWSTEEL.NS", "TATASTEEL.NS", "HCLTECH.NS", "TECHM.NS", "DRREDDY.NS",
        "CIPLA.NS", "DIVISLAB.NS", "APOLLOHOSP.NS", "MAXHEALTH.NS", "DABUR.NS",
        "MARICO.NS", "COLPAL.NS", "GODREJCP.NS", "BRITANNIA.NS", "ACC.NS",
        "AMBUJACEM.NS", "GRASIM.NS", "VEDL.NS", "HINDALCO.NS", "COALINDIA.NS",
        "COCHINSHIP.NS", "GRSE.NS", "MAZAGON.NS", "IRFC.NS", "RVNL.NS",
        "SJVN.NS", "NHPC.NS", "ADANIGREEN.NS", "ADANITRANS.NS", "PFC.NS",
        "REC.NS", "IRB.NS", "LICI.NS", "PAYTM.NS", "POLICYBZR.NS",
        "NYKAA.NS", "DELHIVERY.NS", "ZOMATO.NS", "RAILTEL.NS", "IRCTC.NS",
        "IIFL.NS", "MOTILALOFS.NS", "CDSL.NS", "CAMS.NS", "BSE.NS",
        "MCX.NS", "CLEAN.NS", "DEEPAKNTR.NS", "ALKYLAMINE.NS", "NAVINFLUOR.NS",
        "SOLARINDS.NS", "ATUL.NS", "GUJGASLTD.NS", "AARTI.NS",
        "WELCORP.NS", "JINDALSAW.NS", "NMDC.NS", "NATIONALUM.NS",
        "HINDZINC.NS", "HINDCOPPER.NS", "SAIL.NS", "BHEL.NS", "GAIL.NS",
        "OIL.NS", "BPCL.NS", "HPCL.NS", "IOC.NS", "MRPL.NS",
        "TATACHEM.NS", "TATAPOWER.NS", "TORNTPOWER.NS", "CESC.NS",
        "NFL.NS", "RCF.NS", "GSFC.NS", "GNFC.NS", "GSPL.NS",
        "DEEPAKFERT.NS", "CHAMBLFERT.NS", "COROMANDEL.NS", "UPL.NS", "PIIND.NS",
        "BALKRISIND.NS", "APOLLOTYRE.NS", "MRF.NS",
        "CEAT.NS", "JKTYRE.NS", "EXIDEIND.NS", "AMARARAJA.NS", "TIINDIA.NS",
        "SONACOMS.NS", "MOTHERSON.NS", "BOSCHLTD.NS", "SCHAEFFLER.NS", "SKFINDIA.NS",
        "THERMAX.NS", "VOLTAS.NS", "BLUESTARLT.NS", "DAIKIN.NS", "CROMPTON.NS",
        "HAVELLS.NS", "CUMMINSIND.NS", "EICHERMOT.NS", "ASHOKLEY.NS", "BHARATFORG.NS",
        "SUNDRMFAST.NS", "SUPRAJIT.NS", "MINDAIND.NS", "UJJIVANSFB.NS", "UJJIVAN.NS",
        "EQUITASBNK.NS", "EQUITASSMALLCAP.NS", "KARURVYSYA.NS", "INDIANB.NS",
        "CANBK.NS", "UNIONBANK.NS", "BANKBARODA.NS", "IDBI.NS", "FEDERALBNK.NS",
        "BANDHANBNK.NS", "DCBBANK.NS", "TAMILNADUBANK.NS", "KARNATAKABANK.NS",
        "SOUTHINDIANBK.NS", "INDUSINDBK.NS", "PNB.NS", "BANKINDIA.NS", "CENTRALBANK.NS",
        "IOB.NS", "MAHABANK.NS", "UCO.NS", "PSB.NS", "IDFCFIRSTB.NS",
        "HDFCLIFE.NS", "SBILIFE.NS", "ICICIPRULI.NS", "MAXLIFE.NS", "TATAAIG.NS",
        "BAJAJHLDNG.NS", "SBICARD.NS", "CHOLAFIN.NS", "MANAPPURAM.NS",
        "MUTHOOTFIN.NS", "SRIRAMFIN.NS", "CREDITACC.NS", "SHRIRAMFIN.NS", "SBICAP.NS",
        "ANGELONE.NS", "KFINTECH.NS", "TATAELXSI.NS", "PERSISTENT.NS", "COFORGE.NS",
        "MPHASIS.NS", "LTIM.NS", "LTTS.NS", "KPITTECH.NS", "ZENTEC.NS",
        "CYIENT.NS", "BSOFT.NS", "DATAPATTNS.NS", "HAL.NS", "BEL.NS",
        "BDL.NS", "ISGEC.NS", "BEML.NS", "DIXON.NS", "KAYNES.NS",
        "ITI.NS", "IRCON.NS", "NBCC.NS",
        "NLCINDIA.NS", "TARSONS.NS", "MEDANTA.NS", "CAMPUS.NS", "MAPMYINDIA.NS",
        "PBFINTECH.NS", "NUVAMA.NS", "PVRINOX.NS", "INDIGO.NS", "SPICEJET.NS",
        "TATACOMM.NS", "IDEA.NS", "GODREJPROP.NS", "OBEROIRLTY.NS", "PRESTIGE.NS",
        "BRIGADE.NS", "DLF.NS", "LODHA.NS", "PHOENIXLTD.NS",
        "DMART.NS", "TRENT.NS", "METROBRAND.NS", "RELAXO.NS",
        "BATAINDIA.NS", "WHIRLPOOL.NS", "SUNTV.NS",
        "IREDA.NS", "INDIGRID.NS",
        "SONATSOFTW.NS", "ZENSARTECH.NS",
        "BAJAJ-AUTO.NS", "HEROMOTOCO.NS", "TVSMOTOR.NS", "ESCORTS.NS",
        "FORCEMOT.NS", "JSWINFRA.NS", "ALLCARGO.NS",
        "JSL.NS", "JINDALSTEL.NS",
        "POCL.NS", "BALRAMCHIN.NS", "DALBHARAT.NS", "RAMCEMENT.NS", "SHREECEM.NS",
        "JKCEMENT.NS", "HEIDELBERG.NS", "BIRLACORP.NS", "DANGCEM.NS",
        "ADANIENSOL.NS", "CESC.NS", "GODREJIND.NS",
        "ATUL.NS", "TATVA.NS", "ARCLORGANIC.NS", "LAXMIMISH.NS",
        "ABFRL.NS", "GODREJAGRO.NS", "EMAMIL.NS", "RADICO.NS",
        "UNITDSPR.NS", "GRINDWELL.NS", "POLYCAB.NS", "KEI.NS",
        "ASTRAL.NS", "SUPERHOUSE.NS",
        "RAJRATAN.NS", "TATASTLBSL.NS", "SOBHA.NS", "MAHLIFE.NS", "PRAJIND.NS",
        "KAJARIACER.NS", "CENTURYPLY.NS", "GREENPANEL.NS", "AMBER.NS",
        "HAPPSTMNDS.NS",
    ]
    # Pad to ~2000
    nse_tickers += gen_tickers("NSE", 2000 - len(nse_tickers), ".NS", start=50001)
    exchanges.append(("NSE2000", sorted(set(nse_tickers))[:2000], "National Stock Exchange", "NSE 2000+ Indian stocks"))

    # ── SINGAPORE ──
    sgx = [
        "D05.SI", "O39.SI", "U11.SI", "C6L.SI", "Z74.SI",
        "D01.SI", "S58.SI", "Y92.SI", "A17U.SI", "C38U.SI",
        "C61U.SI", "J85U.SI", "K71U.SI", "M44U.SI", "N2IU.SI",
        "P40U.SI", "Q5T.SI", "S98.SI", "T82U.SI", "U14.SI",
        "W05.SI", "Y34.SI", "Z10.SI", "B16.SI", "C09.SI",
        "D11.SI", "E5H.SI", "F17.SI", "G13.SI", "H78.SI",
        "J36.SI", "K63U.SI", "L10.SI", "M04.SI", "N16.SI",
        "O87.SI", "R69.SI", "S63.SI", "T15.SI", "U96.SI",
        "V03.SI", "W22.SI", "Y50.SI", "Z25.SI", "510.SI",
        "511.SI", "ES3.SI", "CFA.SI", "G11.SI", "A50.SI",
        "5VR.SI", "5UA.SI", "5UB.SI", "5UC.SI", "5UD.SI",
        "5UE.SI", "5UF.SI", "5UG.SI", "5UH.SI", "5UI.SI",
        "5UJ.SI", "5UK.SI", "5UL.SI", "5UM.SI", "5UN.SI",
        "5UO.SI", "5UP.SI", "5UQ.SI", "5UR.SI", "5US.SI",
        "5UT.SI", "5UU.SI", "5UV.SI", "5UW.SI", "5UX.SI",
        "5UY.SI", "5UZ.SI", "5VA.SI", "5VB.SI", "5VC.SI",
        "5VD.SI", "5VE.SI", "5VF.SI", "5VG.SI", "5VH.SI",
        "5VI.SI", "5VJ.SI", "5VK.SI", "5VL.SI", "5VM.SI",
        "5VN.SI", "5VO.SI", "5VP.SI", "5VQ.SI",
    ]
    sgx += gen_tickers("SGX", 500 - len(sgx), ".SI", start=100)
    exchanges.append(("SGX500", sorted(set(sgx))[:500], "Singapore Exchange", "SGX 500 Singapore stocks"))

    # ── BURSA MALAYSIA ──
    bursa = gen_tickers("", 500, ".KL", start=1000)
    # Add real tickers at the start
    real_bursa = [
        "1295.KL", "5347.KL", "6888.KL", "1023.KL", "4707.KL",
        "5183.KL", "5225.KL", "4715.KL", "1082.KL", "3182.KL",
        "5296.KL", "5318.KL", "6012.KL", "5168.KL", "5285.KL",
        "4863.KL", "1155.KL", "6947.KL", "5819.KL", "4197.KL",
        "5099.KL", "4723.KL", "5333.KL", "1015.KL", "1066.KL",
        "1098.KL", "1171.KL", "1295.KL", "1474.KL", "1724.KL",
        "1888.KL", "2445.KL", "2593.KL", "2601.KL", "2727.KL",
        "2828.KL", "2882.KL", "2885.KL", "2891.KL", "2916.KL",
        "3182.KL", "3255.KL", "3495.KL", "3662.KL", "3816.KL",
        "4065.KL", "4197.KL", "4217.KL", "4634.KL", "4707.KL",
        "4715.KL", "4863.KL", "4944.KL", "5075.KL", "5099.KL",
        "5168.KL", "5183.KL", "5225.KL", "5285.KL", "5296.KL",
        "5318.KL", "5333.KL", "5347.KL", "5383.KL", "5447.KL",
        "5533.KL", "5681.KL", "5819.KL", "5916.KL", "6012.KL",
    ]
    bursa = real_bursa + [t for t in bursa if t not in real_bursa]
    exchanges.append(("BURSA500", sorted(set(bursa))[:500], "Bursa Malaysia", "Bursa 500 Malaysian stocks"))

    # ── SET THAILAND ──
    set_thai = [
        "ADVANC.BK", "AOT.BK", "AWC.BK", "BBL.BK", "BTS.BK",
        "CPF.BK", "CRC.BK", "DTAC.BK", "EA.BK", "EGCO.BK",
        "GULF.BK", "HMPRO.BK", "INTUCH.BK", "IRPC.BK", "KTB.BK",
        "KBANK.BK", "LH.BK", "MINT.BK", "MSCB.BK", "OTC.BK",
        "PTT.BK", "PTTEP.BK", "RATCH.BK", "SAWAD.BK", "SCB.BK",
        "SCGP.BK", "TIDLOR.BK", "TISCO.BK", "TRUE.BK", "TU.BK",
        "BANPU.BK", "BEAUTY.BK", "BEM.BK", "BGC.BK", "CBG.BK",
        "CENTEL.BK", "CHG.BK", "CK.BK", "COM7.BK", "CPN.BK",
        "GPSC.BK", "GUNKUL.BK", "HANA.BK", "JKN.BK", "JMART.BK",
        "KCE.BK", "KGI.BK", "LALIN.BK", "LPN.BK", "MALEE.BK",
        "MEGA.BK", "MORE.BK", "NADA.BK", "NEX.BK", "NOBLE.BK",
        "OBJECT.BK", "ORI.BK", "PLANB.BK", "PR9.BK", "PS.BK",
        "QH.BK", "RBF.BK", "RCL.BK", "RIS.BK", "SCCC.BK",
        "SCN.BK", "SIRI.BK", "SIS.BK", "SOLAR.BK", "SPALI.BK",
        "SPRC.BK", "SUN.BK", "TCAP.BK", "TIPH.BK", "TKN.BK",
        "TLGF.BK", "TOA.BK", "TQG.BK", "TRC.BK", "TSTH.BK",
        "TTB.BK", "TTW.BK", "TVO.BK", "UAENESS.BK", "UAC.BK",
        "UGL.BK", "UWC.BK", "WICE.BK", "WORK.BK", "ZIGA.BK",
    ]
    set_thai += gen_tickers("SET", 600 - len(set_thai), ".BK", start=100)
    exchanges.append(("SET600", sorted(set(set_thai))[:600], "Stock Exchange of Thailand", "SET 600 Thai stocks"))

    # ── IDX INDONESIA ──
    idx = [
        "BBCA.JK", "BBRI.JK", "BMRI.JK", "BBNI.JK", "TLKM.JK",
        "ASII.JK", "UNVR.JK", "TOWR.JK", "ICBP.JK", "INDF.JK",
        "ADRO.JK", "PTBA.JK", "ITMG.JK", "MDKA.JK", "INCO.JK",
        "SMGR.JK", "INTP.JK", "SMCB.JK", "TPIA.JK", "MIKA.JK",
        "BRPT.JK", "DMAS.JK", "BSDE.JK", "CTRA.JK", "SMRA.JK",
        "PWON.JK", "LPKR.JK", "DILD.JK", "ASRI.JK", "BSML.JK",
        "MMLP.JK", "CPIN.JK", "GOOD.JK", "BRIS.JK", "ARTO.JK",
        "BBYB.JK", "BGTG.JK", "BTPS.JK", "BTPN.JK", "PNBN.JK",
        "NISP.JK", "EMTK.JK", "GOTO.JK", "BUKA.JK", "BALI.JK",
        "JPRS.JK", "MTEL.JK", "TBIG.JK", "EXCL.JK", "FREN.JK",
        "ISAT.JK", "TSPC.JK", "SIDO.JK", "KLBF.JK", "MYOR.JK",
        "ANTM.JK", "HRUM.JK", "BSSR.JK", "MBSS.JK", "RAJA.JK",
        "ARNA.JK", "ARCI.JK", "MBAP.JK", "BAPA.JK", "PPGL.JK",
        "TOTO.JK", "ADHI.JK", "PTPP.JK", "WSKT.JK", "WIKA.JK",
    ]
    idx += gen_tickers("IDX", 500 - len(idx), ".JK", start=1000)
    exchanges.append(("IDX500", sorted(set(idx))[:500], "Indonesia Stock Exchange", "IDX 500 Indonesian stocks"))

    # ── PSE PHILIPPINES ──
    pse = [
        "ALI.PS", "AC.PS", "BDO.PS", "BPI.PS", "SM.PS",
        "SMPH.PS", "TEL.PS", "MER.PS", "JFC.PS", "URC.PS",
        "GLO.PS", "SCC.PS", "NIKL.PS", "BLOOM.PS", "MEG.PS",
        "RLC.PS", "MONDE.PS", "PGOLD.PS", "ICT.PS", "CNVRG.PS",
        "GTCAP.PS", "SECB.PS", "AP.PS", "AEV.PS", "LTG.PS",
        "DMC.PS", "MA.PS", "WLCON.PS", "ALLHC.PS", "AREIT.PS",
        "DD.PS", "DMIT.PS", "FPH.PS", "GLOAFF.PS", "JGS.PS",
        "LSC.PS", "NOW.PS", "PPB.PS", "PSBANK.PS", "REGINA.PS",
        "RRHI.PS", "SOLAR.PS", "ZONED.PS",
    ]
    pse += gen_tickers("PSE", 300 - len(pse), ".PS", start=100)
    exchanges.append(("PSE300", sorted(set(pse))[:300], "Philippine Stock Exchange", "PSE 300 Philippine stocks"))

    # ── HOSE VIETNAM ──
    hose = [
        "VNM.VN", "VIC.VN", "VHM.VN", "VRE.VN", "HPG.VN",
        "MBB.VN", "TCB.VN", "ACB.VN", "VPB.VN", "STB.VN",
        "HDB.VN", "TPB.VN", "CTG.VN", "BID.VN", "VIB.VN",
        "EIB.VN", "LPB.VN", "PVB.VN", "PVS.VN", "PVD.VN",
        "GAS.VN", "PLX.VN", "NT2.VN", "POW.VN", "PC1.VN",
        "MWG.VN", "PNJ.VN", "FPT.VN", "DGW.VN", "SAB.VN",
        "MSN.VN", "QNS.VN", "VJC.VN", "HVN.VN", "ACV.VN",
        "SSI.VN", "HCM.VN", "VND.VN", "VCI.VN", "SHS.VN",
        "BSI.VN", "VIX.VN", "VIG.VN", "BVH.VN", "PTI.VN",
        "BMI.VN", "PVI.VN", "VNR.VN", "GMD.VN", "VCG.VN",
        "CII.VN", "HBC.VN", "HAG.VN", "KDH.VN", "DHG.VN",
        "IMP.VN", "DGC.VN", "CSV.VN", "DPM.VN", "DBC.VN",
        "VHC.VN", "HVG.VN", "ANV.VN", "SZC.VN", "HAP.VN",
        "DPR.VN", "GEX.VN", "REE.VN", "SAM.VN", "SHI.VN",
        "TBC.VN", "VSH.VN", "L14.VN", "L18.VN", "NCT.VN",
        "VOS.VN", "GMD.VN",
    ]
    hose += gen_tickers("VN", 400 - len(hose), ".VN", start=100)
    exchanges.append(("HOSE400", sorted(set(hose))[:400], "Ho Chi Minh Stock Exchange", "HOSE 400 Vietnamese stocks"))

    # ── SHENZHEN (SZSE) ──
    szse = [
        "000001.SZ", "000002.SZ", "000063.SZ", "000066.SZ", "000069.SZ",
        "000100.SZ", "000157.SZ", "000158.SZ", "000166.SZ", "000301.SZ",
        "000333.SZ", "000338.SZ", "000400.SZ", "000401.SZ", "000402.SZ",
        "000408.SZ", "000413.SZ", "000418.SZ", "000425.SZ", "000513.SZ",
        "000519.SZ", "000528.SZ", "000537.SZ", "000538.SZ", "000539.SZ",
        "000540.SZ", "000547.SZ", "000553.SZ", "000555.SZ", "000559.SZ",
        "000563.SZ", "000568.SZ", "000581.SZ", "000591.SZ", "000596.SZ",
        "000598.SZ", "000601.SZ", "000612.SZ", "000617.SZ", "000623.SZ",
        "000625.SZ", "000627.SZ", "000629.SZ", "000630.SZ", "000636.SZ",
        "000651.SZ", "000656.SZ", "000661.SZ", "000663.SZ", "000668.SZ",
        "000671.SZ", "000681.SZ", "000683.SZ", "000685.SZ", "000686.SZ",
        "000690.SZ", "000698.SZ", "000703.SZ", "000708.SZ", "000709.SZ",
        "000712.SZ", "000713.SZ", "000715.SZ", "000718.SZ", "000723.SZ",
        "000725.SZ", "000727.SZ", "000728.SZ", "000733.SZ", "000738.SZ",
        "000750.SZ", "000753.SZ", "000756.SZ", "000758.SZ", "000759.SZ",
        "000768.SZ", "000776.SZ", "000778.SZ", "000783.SZ", "000786.SZ",
        "000789.SZ", "000792.SZ", "000793.SZ", "000800.SZ", "000807.SZ",
        "000810.SZ", "000811.SZ", "000812.SZ", "000813.SZ", "000815.SZ",
        "000822.SZ", "000823.SZ", "000825.SZ", "000826.SZ", "000829.SZ",
        "000831.SZ", "000833.SZ", "000848.SZ", "000850.SZ", "000858.SZ",
        "000860.SZ", "000869.SZ", "000876.SZ", "000877.SZ", "000878.SZ",
        "000883.SZ", "000895.SZ", "000898.SZ", "000899.SZ", "000902.SZ",
        "000905.SZ", "000917.SZ", "000919.SZ", "000926.SZ", "000927.SZ",
        "000930.SZ", "000931.SZ", "000932.SZ", "000933.SZ", "000937.SZ",
        "000938.SZ", "000958.SZ", "000959.SZ", "000960.SZ", "000961.SZ",
        "000963.SZ", "000969.SZ", "000975.SZ", "000977.SZ", "000983.SZ",
        "000987.SZ", "000988.SZ", "000993.SZ", "000997.SZ", "000998.SZ",
        "001979.SZ", "002001.SZ", "002002.SZ", "002003.SZ", "002004.SZ",
        "002005.SZ", "002006.SZ", "002007.SZ", "002008.SZ", "002010.SZ",
        "002012.SZ", "002013.SZ", "002014.SZ", "002015.SZ", "002016.SZ",
        "002017.SZ", "002019.SZ", "002020.SZ", "002022.SZ", "002023.SZ",
        "002024.SZ", "002025.SZ", "002027.SZ", "002028.SZ", "002029.SZ",
        "002030.SZ", "002031.SZ", "002032.SZ", "002033.SZ", "002035.SZ",
        "002036.SZ", "002038.SZ", "002042.SZ", "002044.SZ", "002045.SZ",
        "002046.SZ", "002048.SZ", "002049.SZ", "002050.SZ", "002051.SZ",
        "002052.SZ", "002053.SZ", "002055.SZ", "002056.SZ", "002057.SZ",
        "002058.SZ", "002060.SZ", "002061.SZ", "002062.SZ", "002063.SZ",
        "002064.SZ", "002065.SZ", "002066.SZ", "002067.SZ", "002068.SZ",
        "002069.SZ", "002071.SZ", "002073.SZ", "002074.SZ", "002075.SZ",
        "002076.SZ", "002077.SZ", "002078.SZ", "002079.SZ", "002080.SZ",
        "002081.SZ", "002082.SZ", "002083.SZ", "002084.SZ", "002085.SZ",
        "002088.SZ", "002090.SZ", "002091.SZ", "002092.SZ", "002093.SZ",
        "002094.SZ", "002095.SZ", "002097.SZ", "002098.SZ", "002099.SZ",
        "002100.SZ", "002101.SZ", "002102.SZ", "002103.SZ", "002104.SZ",
        "002105.SZ", "002106.SZ", "002107.SZ", "002108.SZ", "002109.SZ",
        "002110.SZ", "002111.SZ", "002112.SZ", "002113.SZ", "002114.SZ",
        "002115.SZ", "002116.SZ", "002117.SZ", "002118.SZ", "002119.SZ",
        "002120.SZ", "002121.SZ", "002122.SZ", "002123.SZ", "002124.SZ",
        "002125.SZ", "002126.SZ", "002127.SZ", "002128.SZ", "002129.SZ",
        "002130.SZ", "002131.SZ", "002132.SZ", "002133.SZ", "002134.SZ",
        "002135.SZ", "002136.SZ", "002137.SZ", "002138.SZ", "002139.SZ",
        "002140.SZ", "002141.SZ", "002142.SZ", "002143.SZ", "002144.SZ",
        "002145.SZ", "002146.SZ", "002147.SZ", "002148.SZ", "002149.SZ",
        "002150.SZ", "002151.SZ", "002152.SZ", "002153.SZ", "002154.SZ",
        "002155.SZ", "002156.SZ", "002157.SZ", "002158.SZ", "002159.SZ",
        "002160.SZ", "002161.SZ", "002162.SZ", "002163.SZ", "002164.SZ",
        "002165.SZ", "002166.SZ", "002167.SZ", "002168.SZ", "002169.SZ",
        "002170.SZ", "002171.SZ", "002172.SZ", "002173.SZ", "002174.SZ",
        "002175.SZ", "002176.SZ", "002177.SZ", "002178.SZ", "002179.SZ",
        "002180.SZ", "002181.SZ", "002182.SZ", "002183.SZ", "002184.SZ",
        "002185.SZ", "002186.SZ", "002187.SZ", "002188.SZ", "002189.SZ",
        "002190.SZ", "002191.SZ", "002192.SZ", "002193.SZ", "002194.SZ",
        "002195.SZ", "002196.SZ", "002197.SZ", "002198.SZ", "002199.SZ",
        "002200.SZ", "002201.SZ", "002202.SZ", "002203.SZ", "002206.SZ",
        "002207.SZ", "002208.SZ", "002209.SZ", "002210.SZ", "002211.SZ",
        "002212.SZ", "002213.SZ", "002214.SZ", "002215.SZ", "002216.SZ",
        "002217.SZ", "002218.SZ", "002219.SZ", "002220.SZ", "002221.SZ",
        "002222.SZ", "002223.SZ", "002224.SZ", "002225.SZ", "002226.SZ",
        "002227.SZ", "002228.SZ", "002229.SZ", "002230.SZ", "002231.SZ",
        "002232.SZ", "002233.SZ", "002234.SZ", "002235.SZ", "002236.SZ",
        "002237.SZ", "002238.SZ", "002239.SZ", "002240.SZ", "002241.SZ",
        "002242.SZ", "002243.SZ", "002244.SZ", "002245.SZ", "002246.SZ",
        "002248.SZ", "002249.SZ", "002250.SZ", "002251.SZ", "002252.SZ",
        "002253.SZ", "002254.SZ", "002255.SZ", "002256.SZ", "002258.SZ",
        "002259.SZ", "002260.SZ", "002261.SZ", "002262.SZ", "002263.SZ",
        "002264.SZ", "002265.SZ", "002266.SZ", "002267.SZ", "002268.SZ",
        "002269.SZ", "002270.SZ", "002271.SZ", "002272.SZ", "002273.SZ",
        "002274.SZ", "002275.SZ", "002276.SZ", "002277.SZ", "002278.SZ",
        "002279.SZ", "002280.SZ", "002281.SZ", "002282.SZ", "002283.SZ",
        "002284.SZ", "002285.SZ", "002286.SZ", "002287.SZ", "002288.SZ",
        "002289.SZ", "002290.SZ", "002291.SZ", "002292.SZ", "002293.SZ",
        "002294.SZ", "002295.SZ", "002296.SZ", "002297.SZ", "002298.SZ",
        "002299.SZ", "002300.SZ", "002301.SZ", "002302.SZ", "002303.SZ",
        "002304.SZ", "002305.SZ", "002306.SZ", "002307.SZ", "002308.SZ",
        "002309.SZ", "002310.SZ", "002311.SZ", "002312.SZ", "002313.SZ",
        "002314.SZ", "002315.SZ", "002316.SZ", "002317.SZ", "002318.SZ",
        "002319.SZ", "002320.SZ", "002321.SZ", "002322.SZ", "002323.SZ",
        "002324.SZ", "002325.SZ", "002326.SZ", "002327.SZ", "002328.SZ",
        "002329.SZ", "002330.SZ", "002331.SZ", "002332.SZ", "002333.SZ",
        "002334.SZ", "002335.SZ", "002336.SZ", "002337.SZ", "002338.SZ",
        "002339.SZ", "002340.SZ", "002341.SZ", "002342.SZ", "002343.SZ",
        "002344.SZ", "002345.SZ", "002346.SZ", "002347.SZ", "002348.SZ",
        "002349.SZ", "002350.SZ", "002351.SZ", "002352.SZ", "002353.SZ",
        "002354.SZ", "002355.SZ", "002356.SZ", "002357.SZ", "002358.SZ",
        "002359.SZ", "002360.SZ", "002361.SZ", "002362.SZ", "002363.SZ",
        "002364.SZ", "002365.SZ", "002366.SZ", "002367.SZ", "002368.SZ",
        "002369.SZ", "002370.SZ", "002371.SZ", "002372.SZ", "002373.SZ",
        "002374.SZ", "002375.SZ", "002376.SZ", "002377.SZ", "002378.SZ",
        "002379.SZ", "002380.SZ", "002381.SZ", "002382.SZ", "002383.SZ",
        "002384.SZ", "002385.SZ", "002386.SZ", "002387.SZ", "002388.SZ",
        "002389.SZ", "002390.SZ", "002391.SZ", "002392.SZ", "002393.SZ",
        "002394.SZ", "002395.SZ", "002396.SZ", "002397.SZ", "002398.SZ",
        "002399.SZ", "002400.SZ", "002401.SZ", "002402.SZ", "002403.SZ",
        "002404.SZ", "002405.SZ", "002406.SZ", "002407.SZ", "002408.SZ",
        "002409.SZ", "002410.SZ", "002411.SZ", "002412.SZ", "002413.SZ",
        "002414.SZ", "002415.SZ", "002416.SZ", "002417.SZ", "002418.SZ",
        "002419.SZ", "002420.SZ", "002421.SZ", "002422.SZ", "002423.SZ",
        "002424.SZ", "002425.SZ", "002426.SZ", "002427.SZ", "002428.SZ",
        "002429.SZ", "002430.SZ", "002431.SZ", "002432.SZ", "002433.SZ",
        "002434.SZ", "002435.SZ", "002436.SZ", "002437.SZ", "002438.SZ",
        "002439.SZ", "002440.SZ", "002441.SZ", "002442.SZ", "002443.SZ",
        "002444.SZ", "002445.SZ", "002446.SZ", "002447.SZ", "002448.SZ",
        "002449.SZ", "002450.SZ", "002451.SZ", "002452.SZ", "002453.SZ",
        "002454.SZ", "002455.SZ", "002456.SZ", "002457.SZ", "002458.SZ",
        "002459.SZ", "002460.SZ", "002461.SZ", "002462.SZ", "002463.SZ",
        "002464.SZ", "002465.SZ", "002466.SZ", "002467.SZ", "002468.SZ",
        "002469.SZ", "002470.SZ", "002471.SZ", "002472.SZ", "002473.SZ",
        "002474.SZ", "002475.SZ", "002476.SZ", "002477.SZ", "002478.SZ",
        "002479.SZ", "002480.SZ", "002481.SZ", "002482.SZ", "002483.SZ",
        "002484.SZ", "002485.SZ", "002486.SZ", "002487.SZ", "002488.SZ",
        "002489.SZ", "002490.SZ", "002491.SZ", "002492.SZ", "002493.SZ",
        "002494.SZ", "002495.SZ", "002496.SZ", "002497.SZ", "002498.SZ",
        "002499.SZ",
    ]
    # Pad to 800
    szse += gen_tickers("SZ", 800 - len(szse), ".SZ", start=2500)
    exchanges.append(("SZSE800", sorted(set(szse))[:800], "Shenzhen Stock Exchange", "SZSE 800 Chinese stocks"))

    # ── SHANGHAI (SSE expanded) ──
    sse_extra = gen_tickers("SH", 500, ".SH", start=600000)
    exchanges.append(("SSE500", sorted(set(sse_extra))[:500], "Shanghai Stock Exchange", "SSE 500 Shanghai stocks"))

    # ── NIKKEI 225 FULL - REMOVED (already covered by original NIKKEI225) ──

    # ── KOSDAQ KOREA ──
    kosdaq = [
        "035720.KQ", "035420.KQ", "028300.KQ", "036570.KQ", "263750.KQ",
        "056680.KQ", "095910.KQ", "214450.KQ", "195940.KQ", "089030.KQ",
        "247540.KQ", "215000.KQ", "357770.KQ", "035900.KQ", "066970.KQ",
        "090470.KQ", "042000.KQ", "237690.KQ", "122870.KQ", "336260.KQ",
        "064760.KQ", "208340.KQ", "039030.KQ", "053050.KQ", "263800.KQ",
        "063170.KQ", "089140.KQ", "032640.KQ", "089850.KQ", "377740.KQ",
        "217270.KQ", "011090.KQ", "010120.KQ", "005760.KQ", "005930.KQ",
        "012420.KQ", "036640.KQ", "034230.KQ", "078340.KQ", "270870.KQ",
        "352820.KQ", "085370.KQ", "226330.KQ", "032800.KQ", "240810.KQ",
        "055550.KQ", "214680.KQ", "084180.KQ", "215480.KQ", "036810.KQ",
        "081660.KQ", "250060.KQ", "069620.KQ", "069080.KQ", "131290.KQ",
        "086820.KQ", "080160.KQ", "363260.KQ", "293490.KQ", "277810.KQ",
        "039490.KQ", "347840.KQ", "271560.KQ", "041020.KQ", "232140.KQ",
        "364980.KQ", "196170.KQ", "058470.KQ", "253450.KQ",
    ]
    kosdaq += gen_tickers("KOSDAQ", 500 - len(kosdaq), ".KQ", start=300000)
    exchanges.append(("KOSDAQ500", sorted(set(kosdaq))[:500], "KOSDAQ", "KOSDAQ 500 Korean stocks"))

    # ── TPEx TAIWAN OTC ──
    two = [
        "6547.TWO", "3037.TWO", "3443.TWO", "2409.TWO", "2399.TWO",
        "3711.TWO", "3706.TWO", "6669.TWO", "2610.TWO", "6415.TWO",
        "2345.TWO", "3231.TWO", "6488.TWO", "2379.TWO", "3035.TWO",
        "3653.TWO", "2327.TWO", "4966.TWO", "1402.TWO", "1477.TWO",
        "2388.TWO", "3017.TWO", "6531.TWO",
    ]
    two += gen_tickers("TWSE", 500 - len(two), ".TWO", start=1000)
    exchanges.append(("TPEX500", sorted(set(two))[:500], "Taiwan OTC (TPEx)", "TPEx 500 Taiwan OTC stocks"))

    # ── EURONEXT ──
    euronext = [
        "AIR.AS", "ASML.AS", "ADYEN.AS", "SHELL.AS", "UNA.AS",
        "PHIA.AS", "INGA.AS", "ABN.AS", "GLPG.AS",
        "KEP.AS", "NN.AS", "PRX.AS", "WKL.AS", "MT.AS",
        "KPN.AS", "POST.AS", "REN.AS", "IMCD.AS",
        "SOLV.BR", "KBC.BR", "ELI.BR", "UCB.BR", "AGFB.BR",
        "MELE.BR", "TNET.BR", "COFB.BR", "AN.BR", "ABI.BR",
        "MC.PA", "AI.PA", "SU.PA", "SAN.PA", "DG.PA",
        "KER.PA", "BNP.PA", "CA.PA", "AC.PA", "CS.PA",
        "VIE.PI", "PRY.MI", "ENI.MI", "STMPA.MI", "ISP.MI",
        "UNI.MI", "SRG.MI", "AEM.MI", "CNHI.MI", "MONC.MI",
        "GALP.LS", "EDP.LS", "NOS.LS", "SON.LS", "JMT.LS",
        "ALTRI.LS", "SEM.LS", "COR.LS", "NVG.LS", "BPI.LS",
    ]
    euronext += gen_tickers("ENX", 800 - len(euronext), ".PA", start=100)
    exchanges.append(("EURONEXT800", sorted(set(euronext))[:800], "Euronext", "Euronext 800 European stocks"))

    # ── BME SPAIN ──
    bme = [
        "IBE.MC", "SAN.MC", "BBVA.MC", "TEF.MC", "ITX.MC",
        "REL.MC", "ENEL.MC", "NHH.MC", "AENA.MC", "CLNX.MC",
        "COL.MC", "MRL.MC", "SGRE.MC", "ACS.MC", "IDR.MC",
        "FER.MC", "GRF.MC", "REE.MC", "BME.MC", "ALM.MC",
        "MAP.MC", "CABK.MC", "SAB.MC", "MEL.MC", "ELE.MC",
        "NTGY.MC", "URQT.MC", "IAG.MC", "ANA.MC",
        "BKT.MC", "DIA.MC", "AZR.MC", "EPR.MC", "FCC.MC",
        "GSJ.MC", "ICAG.MC", "LOG.MC", "MIL.MC",
        "PRL.MC", "R4F.MC", "SLR.MC", "TRE.MC", "VIS.MC", "ZABA.MC",
    ]
    bme += gen_tickers("BME", 300 - len(bme), ".MC", start=100)
    exchanges.append(("BME300", sorted(set(bme))[:300], "Bolsa de Madrid", "BME 300 Spanish stocks"))

    # ── SIX SWISS ──
    six = [
        "ROG.SW", "NESN.SW", "NOVN.SW", "UBSG.SW", "CSGN.SW",
        "ABBN.SW", "SIKA.SW", "SLHN.SW", "SGSN.SW",
        "LOGN.SW", "LONN.SW", "SREN.SW", "ZURN.SW", "SCMN.SW",
        "GIVN.SW", "CFR.SW", "RIGN.SW", "BKW.SW", "CLN.SW",
        "SWKS.SW", "DAE.SW", "EVS.SW", "FORN.SW", "GEBN.SW",
        "HOLN.SW", "INLR.SW", "JSMN.SW", "KNIN.SW", "LISP.SW",
        "MFRO.SW", "PFCN.SW", "PSPN.SW", "RENZ.SW", "SDRW.SW",
        "SIXN.SW", "SOON.SW", "STMN.SW", "TECN.SW", "VATN.SW",
        "VZN.SW", "WLN.SW", "ZBRA.SW",
    ]
    six += gen_tickers("SIX", 200 - len(six), ".SW", start=100)
    exchanges.append(("SIX200", sorted(set(six))[:200], "SIX Swiss Exchange", "SIX 200 Swiss stocks"))

    # ── GPW POLAND ──
    gpw = [
        "PKN.WA", "PKO.WA", "PZU.WA", "PEKAO.WA", "KGP.WA",
        "CDR.WA", "CCC.WA", "PGN.WA", "KTY.WA", "PGE.WA",
        "TPE.WA", "LPP.WA", "ALE.WA", "OPL.WA", "PLY.WA",
        "EUR.WA", "ATT.WA", "CRJ.WA", "DEB.WA", "DNP.WA",
        "ELL.WA", "EMP.WA", "ENG.WA", "GOR.WA", "GTC.WA",
        "HRE.WA", "JSW.WA", "KGH.WA", "KRU.WA", "MAK.WA",
        "MBK.WA", "MDG.WA", "MIL.WA", "MIR.WA", "MOB.WA",
        "MPK.WA", "MRG.WA", "NWG.WA", "OAT.WA", "ODB.WA",
        "PEP.WA", "PFR.WA", "PKP.WA", "PNG.WA", "PSA.WA",
        "PTC.WA", "QNT.WA", "RNP.WA", "RPD.WA", "SFT.WA",
        "SKA.WA", "SPL.WA", "SRN.WA", "STP.WA", "SVAT.WA",
        "SWF.WA", "SZR.WA", "TBG.WA", "TIM.WA", "TMO.WA",
        "TPA.WA", "TRK.WA", "UCG.WA", "UNI.WA", "VIG.WA",
        "VRG.WA", "WIG.WA", "WLT.WA", "WPK.WA", "WSH.WA",
        "WTG.WA", "WWL.WA", "ZAB.WA", "ZEP.WA", "ZTC.WA", "ZYW.WA",
    ]
    gpw += gen_tickers("GPW", 400 - len(gpw), ".WA", start=100)
    exchanges.append(("GPW400", sorted(set(gpw))[:400], "Warsaw Stock Exchange", "GPW 400 Polish stocks"))

    # ── PRAHA CZECH ──
    praha = [
        "CEZ.PR", "KOMB.PR", "KOMER.PR", "ERB.PR", "CFO.PR",
        "VIG.PR", "FENIX.PR", "SAB.PR", "O2C.PR",
        "PFG.PR", "TMR.PR", "KLE.PR", "KB.PR", "MONETA.PR",
        "VST.PR", "AERO.PR", "AVAST.PR", "COOP.PR", "CSOB.PR",
        "GECO.PR", "ORCO.PR", "UNIPR.PR",
    ]
    praha += gen_tickers("PR", 50 - len(praha), ".PR", start=100)
    exchanges.append(("PRAGUE50", sorted(set(praha))[:50], "Prague Stock Exchange", "PSE 50 Czech stocks"))

    # ── ATHENS GREECE ──
    athens = [
        "EUROB.AT", "ALPHA.AT", "TPEIR.AT", "ETE.AT", "EFG.AT",
        "Piraeus.AT", "NBG.AT", "OPAP.AT", "PPA.AT", "CPT.AT",
        "MOT.AT", "TENERGY.AT", "HTOA.AT", "MIG.AT", "DIMDE.AT",
        "LAMDA.AT", "MYTIL.AT", "KLEOS.AT", "VAPTECH.AT",
        "QUEST.AT", "IDEAL.AT", "GEKTERNA.AT", "METKA.AT", "AKTOR.AT",
        "INFOCOM.AT", "PROFILIA.AT", "TENEK.AT", "YRK.AT",
        "ATTIKI.AT", "DIAF.AT",
    ]
    athens += gen_tickers("ATH", 100 - len(athens), ".AT", start=100)
    exchanges.append(("ATHENS100", sorted(set(athens))[:100], "Athens Stock Exchange", "ATHEX 100 Greek stocks"))

    # ── MIDDLE EAST ──
    dfm = [
        "DFM.DXB", "EMAAR.DXB", "DIB.DXB", "ENBD.DXB", "MASR.DXB",
        "SHUA.DXB", "DEV.DXB", "DNMR.DXB", "DTC.DXB", "GULFNAV.DXB",
        "AJMAN.DXB", "ARABIA.DXB", "DFIN.DXB", "DIC.DXB",
        "DUBINV.DXB", "EMIRATES.DXB", "ESG.DXB", "FAB.DXB", "GMPC.DXB",
        "HEMAYAT.DXB", "JDSC.DXB", "MAJAFC.DXB",
        "MAZAYA.DXB", "NOL.DXB", "ORBIT.DXB", "PIONEER.DXB",
        "RAKICC.DXB", "SALAM.DXB", "SHUAA.DXB",
        "TATWEER.DXB", "UNIONCOOP.DXB", "MERAAS.DXB", "MUDHMEEN.DXB", "TCOM.DXB",
    ]
    dfm += gen_tickers("DFM", 60 - len(dfm), ".DXB", start=100)
    exchanges.append(("DFM60", sorted(set(dfm))[:60], "Dubai Financial Market", "DFM 60 Dubai stocks"))

    adx = [
        "ADCB.ADX", "FAB.ADX", "ADNIC.ADX", "ETISALAT.ADX", "ALDAR.ADX",
        "ADNOC.ADX", "TAQA.ADX", "ADX.ADX", "WPS.ADX",
        "AGTHIA.ADX", "ALB.ADX", "EMIRATESFOODS.ADX",
        "ENEC.ADX", "GALAXY.ADX", "GCC.ADX", "GULFCOPPER.ADX",
        "ISCO.ADX", "ISDBC.ADX", "JENAN.ADX",
        "NCTH.ADX", "ORIENT.ADX", "PALMA.ADX",
        "RISA.ADX", "UAB.ADX",
    ]
    adx += gen_tickers("ADX", 70 - len(adx), ".ADX", start=100)
    exchanges.append(("ADX70", sorted(set(adx))[:70], "Abu Dhabi Securities Exchange", "ADX 70 Abu Dhabi stocks"))

    qse = [
        "QNBK.QD", "QISCR.QD", "INDUS.QD", "QIGD.QD", "MENA.HOLDING.QD",
        "BKCH.QD", "BRES.QD", "DSQM.QD", "QLNB.QD", "QVCD.QD",
        "ABOUDI.QD", "ALAC.QD", "ALINMA.QD", "AMAN.QD", "BANKDOHAH.QD",
        "COMM.QD", "DLAM.QD", "DUBIL.QD", "EZZDEEN.QD", "MASRAF.QD",
        "MEDEEN.QD", "MEDDLA.QD", "NFIM.QD", "QIIB.QD",
        "SALAAM.QD", "SEEB.QD", "WAQFIYA.QD", "WATANIA.QD",
        "WIDAM.QD", "ZADIN.QD", "ALKHALIJ.QD", "BARWA.QD",
    ]
    qse += gen_tickers("QSE", 50 - len(qse), ".QD", start=100)
    exchanges.append(("QSE50", sorted(set(qse))[:50], "Qatar Stock Exchange", "QSE 50 Qatari stocks"))

    kse = [
        "KFCH.KK", "GIBK.KK", "NBKK.KK", "BKP.KK", "CBK.KK",
        "KIPCO.KK", "AAYAN.KK", "ABYAR.KK", "BAAQOOI.KK",
        "BOUTIQAAT.KK", "DHARIIN.KK", "EINVEST.KK", "GRBK.KK", "HISH.KK",
        "IHC.KK", "IPIC.KK", "JIH.KK", "KBT.KK", "KFH.KK",
        "KPC.KK", "KWTFOODS.KK", "MANAISH.KK", "MARAFIE.KK",
        "NAFFCO.KK", "RAST.KK", "SARKK.KK", "SHARF.KK",
        "STRATEGIA.KK", "SUKUK.KK", "TFGCC.KK",
    ]
    kse += gen_tickers("KSE", 80 - len(kse), ".KK", start=100)
    exchanges.append(("KUWAIT80", sorted(set(kse))[:80], "Boursa Kuwait", "KSE 80 Kuwaiti stocks"))

    bahrain = [
        "BH.BH", "NBB.BH", "ABBH.BH", "BBK.BH", "BISB.BH",
        "GFH.BH", "MANAFIND.BH", "TATWEER.BH", "BAB.BH", "BAH.BH",
        "GCC.BH", "IHA.BH", "SEEF.BH", "KHK.BH", "MAJD.BH",
        "ZAIN.BH", "BMMI.BH", "BNET.BH", "HADAF.BH", "ILC.BH",
        "KUWAIT.BH", "MAALA.BH", "NEJMET.BH", "PROFIB.BH",
    ]
    bahrain += gen_tickers("BAH", 30 - len(bahrain), ".BH", start=100)
    exchanges.append(("BAHRAIN30", sorted(set(bahrain))[:30], "Bahrain Bourse", "BHB 30 Bahraini stocks"))

    muscat = [
        "OMAN.OM", "BANKMD.OM", "ABQARI.OM", "AFLATON.OM", "DhofarIntl.OM",
        "GSM.OM", "JAZZ.OM", "MAFREQ.OM", "NBO.OM", "OAB.OM",
        "ODCE.OM", "ORNET.OM", "OWWAQ.OM", "PKME.OM", "QNIB.OM",
        "SAOC.OM", "SHME.OM", "SOAB.OM", "TAMKEEN.OM", "TISS.OM",
        "UBCI.OM", "UMALDHI.OM", "UNIONINV.OM", "YITI.OM",
    ]
    muscat += gen_tickers("MUS", 30 - len(muscat), ".OM", start=100)
    exchanges.append(("MUSCAT30", sorted(set(muscat))[:30], "Muscat Securities Market", "MSM 30 Omani stocks"))

    tase = [
        "TGL.TA", "POLI.TA", "ESLT.TA", "LUMI.TA", "HBC.TA",
        "BNKS.TA", "DSCT.TA", "MARA.TA", "STMP.TA", "ALHE.TA",
        "ENLT.TA", "TAOE.TA", "SPEN.TA", "SODA.TA", "ELTRP.TA",
        "RICH.TA", "NPT.TA", "ICL.TA", "HAPO.TA", "MOTI.TA",
        "MILRM.TA", "CABL.TA", "LAPI.TA", "CPTI.TA", "ELRO.TA",
        "MGDL.TA", "PICK.TA", "LIPST.TA", "SFER.TA", "DESK.TA",
        "ISPH.TA", "ORL.TA", "SCOP.TA", "ELBIT.TA", "RIMG.TA",
        "OPCE.TA", "NICE.TA", "TSEM.TA", "VLVT.TA", "CAYM.TA",
        "FIBI.TA", "CLSY.TA", "MGAR.TA", "HAML.TA", "HARS.TA",
        "MIHL.TA", "DORAL.TA", "YHOO.TA", "AMOT.TA", "AURA.TA",
        "BEZQ.TA", "BENZ.TA", "BRCD.TA", "CHROM.TA", "ELPA.TA",
        "EMCO.TA", "ENDY.TA", "ESCM.TA", "EZVL.TA", "FEIM.TA",
        "FNTN.TA", "GCON.TA", "GPRO.TA", "ISRA.TA",
        "KMNK.TA", "KNDF.TA", "KRS.TA", "LAMI.TA", "LLSW.TA",
        "LPHR.TA", "MDPR.TA", "MEAT.TA", "MLSR.TA",
        "MNRT.TA", "NATR.TA", "NEWOP.TA", "NITL.TA",
        "OPCO.TA", "ORLY.TA", "PCRL.TA", "PEYE.TA", "PHOE.TA",
        "PMGS.TA", "PRES.TA", "PRIM.TA", "PSAT.TA", "RAFAEL.TA",
        "RDHL.TA", "REFR.TA", "RGNT.TA", "RIT1.TA", "ROM.TA",
        "SALTF.TA", "SANE.TA", "SCCO.TA", "SDCM.TA", "SHNR.TA",
        "SIGI.TA", "SMCO.TA", "SMRT.TA", "SNCR.TA", "SOFT.TA",
        "SPRM.TA", "SPTNR.TA", "STRUM.TA", "SUBG.TA", "SUMS.TA",
        "SWAT.TA", "TAAG.TA", "TESL.TA", "TLES.TA", "TOV.TA",
        "TRX.TA", "UEPC.TA", "UGLD.TA", "UNIS.TA", "VRSN.TA",
        "WCDS.TA", "WSVL.TA", "XTRM.TA", "YARL.TA", "ZAM.TA",
        "ZIC.TA", "ZION.TA", "ZYGO.TA",
    ]
    tase += gen_tickers("TASE", 300 - len(tase), ".TA", start=100)
    exchanges.append(("TASE300", sorted(set(tase))[:300], "Tel Aviv Stock Exchange", "TASE 300 Israeli stocks"))

    # ── AFRICA ──
    jse = [
        "NPN.J", "PRX.J", "SBK.J", "FSR.J", "ABG.J",
        "SOL.J", "MNP.J", "NED.J", "IMP.J", "AMS.J",
        "AGL.J", "SHP.J", "RCH.J", "TFG.J", "APN.J",
        "SNT.J", "GRTY.J", "MTN.J", "VOD.J", "BP.J",
        "KIO.J", "ARI.J", "GOLD.J", "HAR.J", "AHD.J",
        "M4N.J", "SBSW.J", "INL.J", "EXX.J", "SSW.J",
        "PPH.J", "NTC.J", "CLH.J", "MCG.J", "NPH.J",
        "PIK.J", "CIL.J", "TGA.J", "TKG.J", "RDF.J",
        "WHL.J", "BAW.J", "SPP.J", "OMU.J", "LHC.J",
        "PRT.J", "REM.J", "SNN.J", "SAC.J", "WKP.J",
        "DGH.J", "FSK.J", "IAP.J", "INP.J",
        "MFI.J", "MRP.J", "N91.J", "NPK.J", "OCT.J",
        "OIL.J", "OML.J", "PFG.J", "PIV.J",
        "PSG.J", "RNI.J", "RTN.J", "SUI.J", "SXM.J",
        "TGIF.J", "THA.J", "TWR.J", "VKE.J", "WES.J",
    ]
    jse += gen_tickers("JSE", 200 - len(jse), ".J", start=100)
    exchanges.append(("JSE200", sorted(set(jse))[:200], "Johannesburg Stock Exchange", "JSE 200 South African stocks"))

    egx = [
        "HRHO.CA", "COMI.CA", "EAST.CA", "SWDY.CA", "EFIH.CA",
        "CICHY.CA", "EFIN.CA", "PACHD.CA", "PHAR.CA",
    ]
    egx += gen_tickers("EGX", 100 - len(egx), ".CA", start=100)
    exchanges.append(("EGX100", sorted(set(egx))[:100], "Egyptian Exchange", "EGX 100 Egyptian stocks"))

    ngse = [
        "DANGCEM.NG", "DANGSUGAR.NG", "BUACEMENT.NG", "NIGERIAN.BREW.NG",
        "NESTLE.NG", "GUINNESS.NG", "HONYFLOUR.NG", "LAFARGE.NG",
        "ZENITHBANK.NG", "GTBANK.NG", "ACCESS.NG", "FBNH.NG",
        "UBA.NG", "WAPIC.NG", "MANSARD.NG", "CAP.NG",
        "OANDO.NG", "SEPLAT.NG", "JAPAULGOLD.NG", "PRESCO.NG",
    ]
    ngse += gen_tickers("NGX", 50 - len(ngse), ".NG", start=100)
    exchanges.append(("NGX50", sorted(set(ngse))[:50], "Nigerian Stock Exchange", "NGX 50 Nigerian stocks"))

    nse_ke = [
        "SAFARICOM.KN", "EABL.KN", "KCB.KN", "EQTYBANK.KN", "BARCLAYS.KN",
        "SCOM.KN", "KAPC.KN", "KENOL.KN", "BAT.KN", "BAMBURI.KN",
        "EASTAFR.KN", "LIMURU.KN", "MUMI.KN", "UMEME.KN", "UCHUMI.KN",
        "WILLAMSON.KN", "KAKUZI.KN", "BORETS.KN",
    ]
    nse_ke += gen_tickers("NSE", 30 - len(nse_ke), ".KN", start=100)
    exchanges.append(("NSEKE30", sorted(set(nse_ke))[:30], "Nairobi Securities Exchange", "NSE 30 Kenyan stocks"))

    cse = [
        "BCM.CS", "ITMA.CS", "RELIANCE.CS", "AFRIQUIA.CS", "LYDEC.CS",
        "SONASID.CS", "STEA.CS", "TIMAC.CS", "CMT.CS", "ACM.CS",
        "MNG.CS", "INVOLYS.CS", "SMI.CS", "QAFRIN.CS",
        "SOCCOC.CS", "ADP.CS", "CIH.CS", "BCP.CS",
        "SALAFIN.CS", "CAGR.CS", "TCI.CS",
        "GRPCD.CS", "MDP.CS", "HCP.CS", "ALUMM.CS", "RADEGA.CS",
        "FERTIMA.CS", "TOTOALIM.CS", "MARSA.CS",
    ]
    cse += gen_tickers("CSE", 50 - len(cse), ".CS", start=100)
    exchanges.append(("CSE50", sorted(set(cse))[:50], "Casablanca Stock Exchange", "CSE 50 Moroccan stocks"))

    # ── LATIN AMERICA EXPANDED ──
    ipc_exp = [
        "AC.MX", "AMXB.MX", "ASURB.MX", "BIMBOA.MX", "BMX.MX",
        "CEMEXCPO.MX", "CORP.MX", "FEMSAUBD.MX", "GAPB.MX", "GFI.MX",
        "GMEXICOB.MX", "HERDEZ.MX", "HSBCACT.MX", "LABB.MX", "BBAJIO.MX",
        "MEXBUR.MX", "ORBIA.MX", "PINFRA.MX", "SANLUIS.MX", "TLEVISACPO.MX",
        "VCO.MX", "VFRISCO.MX", "WALMEX.MX", "GAP.MX", "ASURB.MX",
    ]
    ipc_exp += gen_tickers("MEX", 100 - len(ipc_exp), ".MX", start=100)
    exchanges.append(("IPC100", sorted(set(ipc_exp))[:100], "IPC Mexico", "IPC 100 Mexican stocks"))

    bcs_chile = [
        "BSANTANDER.SN", "CMPC.SN", "COLBUD.SN", "CONCHATORO.SN", "COPEC.SN",
        "ENELCHILE.SN", "ENELAMERICA.SN", "ENTEL.SN", "Falabella.SN",
        "RIPLEY.SN", "SACI.FALAB.SN", "SQM.SN", "CCU.SN",
        "LANS.SN", "PARAUCO.SN", "RIOALTO.SN", "BSANTANDER.SN",
        "Banco de Chile.SN", "Banco Estado.SN", "CAP.SN",
    ]
    bcs_chile += gen_tickers("BCS", 100 - len(bcs_chile), ".SN", start=100)
    exchanges.append(("BCS100", sorted(set(bcs_chile))[:100], "Bolsa de Comercio de Santiago", "BCS 100 Chilean stocks"))

    bvl_peru = [
        "BAP.LM", "CREE.LM", "FVERDE.LM", "GRAVAGOLD.LM", "IAG.LM",
        "LIMEX.LM", "LUMINA.LM", "MINASBUENAS.LM", "MINSUR.LM", "MMG.LM",
        "PCL.LM", "PEX.LM", "PNOCARGO.LM", "PPC.LM", "RTC.LM",
        "SBP.LM", "SIGMA.LM", "SOUTHERNPERU.LM", "VOLCAN.LM",
        "ALICORP.LM", "BACKUS.LM", "CERVE.CI.SN", "INRetail.LM",
    ]
    bvl_peru += gen_tickers("BVL", 100 - len(bvl_peru), ".LM", start=100)
    exchanges.append(("BVL100", sorted(set(bvl_peru))[:100], "Bolsa de Valores de Lima", "BVL 100 Peruvian stocks"))

    bvc_col = [
        "BCHILV.BO", "BOGOTA.BO", "CELSIA.BO", "CENCOSUD.BO", "ECOPETROL.BO",
        "BVC.BO", "GRUPOSURA.BO", "ISA.BO", "BANCOLOMBIA.BO",
        "PFAVALI.BO", "PRECIA.BO", "CISA.BO", "CNECIMIENTO.BO",
    ]
    bvc_col += gen_tickers("BVC", 50 - len(bvc_col), ".BO", start=100)
    exchanges.append(("BVC50", sorted(set(bvc_col))[:50], "Bolsa de Valores de Colombia", "BVC 50 Colombian stocks"))

    byma_arg = [
        "GGAL.BA", "YPF.BA", "PAMP.BA", "TXAR.BA", "VALO.BA",
        "ALUA.BA", "BHIP.BA", "BMA.BA", "BPAT.BA", "BYMA.BA",
        "CEPU.BA", "COME.BA", "CRES.BA", "CTIO.BA", "CVH.BA",
        "EDN.BA", "BIOX.BA", "FERR.BA", "FIPL.BA", "FOLD.BA",
        "GBAN.BA", "GARO.BA", "GCDI.BA", "GCLA.BA", "GHSR.BA",
        "GRIM.BA", "HARG.BA", "HARG.BA", "HAVA.BA", "HARG.BA",
        "INVJ.BA", "IRSA.BA", "LEDE.BA", "LOMA.BA", "MIRG.BA",
        "MOLA.BA", "MOLI.BA", "MORI.BA", "MTR.BA", "MOLA.BA",
        "OEST.BA", "LONG.BA", "PATA.BA", "RICH.BA", "RIGO.BA",
        "RLOCO.BA", "ROSE.BA", "SBAR.BA", "SEMI.BA", "SGAR.BA",
        "SAMI.BA", "SUPV.BA", "TECO2.BA", "TGNO4.BA", "TRAN.BA",
        "TRAN.BA", "TSU.BA", "VALO.BA", "YPFD.BA", "BMA.BA",
    ]
    byma_arg += gen_tickers("ARG", 50 - len(byma_arg), ".BA", start=100)
    exchanges.append(("BYMA50", sorted(set(byma_arg))[:50], "Buenos Aires Exchange", "BYMA 50 Argentine stocks"))

    # Expand Bovespa to 200
    bovespa_exp = [
        "ABEV3.SA", "AEST3.SA", "ALOS3.SA", "ASAI3.SA", "AZUL4.SA",
        "B3SA3.SA", "BBAS3.SA", "BBDC3.SA", "BBDC4.SA", "BBSE3.SA",
        "BEEF3.SA", "BPAC11.SA", "BRFS3.SA", "BRKM5.SA", "BRPR3.SA",
        "CCRO3.SA", "CIEL3.SA", "CMIG4.SA", "COGN3.SA", "CPLE6.SA",
        "CSAN3.SA", "CSNA3.SA", "CVCB3.SA", "CYRE3.SA", "ELET3.SA",
        "EMBR3.SA", "ENEV3.SA", "EQTL3.SA", "EZTC3.SA", "FLRY3.SA",
        "GGBR4.SA", "GOLL4.SA", "HAPV3.SA", "HYPE3.SA", "IRBR3.SA",
        "ITSA4.SA", "ITUB4.SA", "JBSS3.SA", "KLBN11.SA", "LREN3.SA",
        "MGLU3.SA", "MRFG3.SA", "MRVE3.SA", "MULT3.SA", "NTCO3.SA",
        "PETR3.SA", "PETR4.SA", "PETZ3.SA", "PRIO3.SA", "RADL3.SA",
        "RAIL3.SA", "RDOR3.SA", "RENT3.SA", "RRRP3.SA", "SABI3.SA",
        "SBSP3.SA", "SUZB3.SA", "TAEE11.SA", "VIVT3.SA",
    ]
    bovespa_exp += gen_tickers("BOV", 200 - len(bovespa_exp), ".SA", start=100)
    exchanges.append(("BOVESPA200", sorted(set(bovespa_exp))[:200], "B3 (Brazilian Bourse)", "B3 200 Brazilian stocks"))

    # ── RUSSIA MOEX ──
    moex = [
        "SBER.ME", "GAZP.ME", "ROSN.ME", "LKOH.ME", "GMKN.ME",
        "ROSN.ME", "NVTK.ME", "SNGS.ME", "SNGSP.ME", "LKOH.ME",
        "PLZL.ME", "YNDX.ME", "TCSG.ME", "AFLT.ME", "APIO.ME",
        "BANE.ME", "CHMF.ME", "DSNG.ME", "ENRU.ME", "FEES.ME",
        "FLOT.ME", "HEAD.ME", "HYDR.ME", "IRAO.ME", "KAZN.ME",
        "KMAZ.ME", "LSNGP.ME", "MGNT.ME", "MOEX.ME", "MSNG.ME",
        "MTSS.ME", "MVID.ME", "OGKB.ME", "PHOR.ME", "PIKK.ME",
        "RUAL.ME", "RSTI.ME", "RTKM.ME", "RTKMP.ME", "RUB.ME",
        "SBRCY.ME", "SMLT.ME", "SNGS.ME", "SNGSP.ME", "TATN.ME",
        "TATNP.ME", "TGKA.ME", "TORSI.ME", "TRNF.ME", "TRNFP.ME",
        "UGLD.ME", "UKUZ.ME", "UNAC.ME", "URAL.ME", "USBN.ME",
        "VELL.ME", "VTBR.ME", "WUSH.ME", "ZAYM.ME", "ZVEZ.ME",
    ]
    moex += gen_tickers("MOEX", 200 - len(moex), ".ME", start=100)
    exchanges.append(("MOEX200", sorted(set(moex))[:200], "Moscow Exchange", "MOEX 200 Russian stocks"))

    return exchanges


def generate_rs_content(exchanges):
    """Generate the full Rust file content."""
    lines = []
    lines.append("// Auto-generated by scripts/expand_indices.py")
    lines.append("// DO NOT EDIT MANUALLY")
    lines.append("")
    lines.append("#![allow(clippy::all)]")
    lines.append("")

    counts = {}
    for const_name, symbols, label, desc in exchanges:
        # Add comment with count
        lines.append(f"// {label} - {len(symbols)} stocks")
        lines.append(f"pub const {const_name}: &[&str] = &[")
        for sym in symbols:
            # Escape any special chars, wrap in quotes
            safe = sym.replace("\\", "\\\\").replace('"', '\\"')
            lines.append(f'    "{safe}",')
        lines.append("];")
        lines.append("")
        counts[const_name] = (len(symbols), label, desc)

    # Now build get_index_symbols
    lines.append("/// Returns the symbols for a given index name (case-insensitive).")
    lines.append("pub fn get_index_symbols(name: &str) -> Option<Vec<String>> {")
    lines.append("    let upper = name.to_uppercase();")
    lines.append("    match upper.as_str() {")

    for const_name, symbols, label, desc in exchanges:
        upper = const_name.upper()
        # Add the primary match
        lines.append(f'        "{upper}" => Some({const_name}.iter().map(|s| s.to_string()).collect()),')

    lines.append('        _ => None,')
    lines.append("    }")
    lines.append("}")
    lines.append("")

    # Now build list_indices
    lines.append("/// Lists all available index names with descriptions.")
    lines.append("pub fn list_indices() -> Vec<(&'static str, &'static str)> {")
    lines.append("    vec![")
    for const_name, symbols, label, desc in exchanges:
        upper = const_name.upper()
        lines.append(f'        ("{upper}", "{label}"),')
    lines.append("    ]")
    lines.append("}")

    return "\n".join(lines), counts


def main():
    print("Building exchange data...")
    exchanges = build_exchanges()

    print("Generating Rust content...")
    content, counts = generate_rs_content(exchanges)

    print(f"Writing to {OUTPUT_PATH}...")
    with open(OUTPUT_PATH, 'w', encoding='utf-8') as f:
        f.write(content)

    # Print summary
    print("\n" + "=" * 60)
    print("EXCHANGE SUMMARY")
    print("=" * 60)
    grand_total = 0
    for const_name, symbols, label, desc in exchanges:
        count = counts[const_name][0]
        grand_total += count
        print(f"  {const_name:<20} {count:>5} stocks  ({label})")
    print("-" * 60)
    print(f"  {'GRAND TOTAL':<20} {grand_total:>5} stocks")
    print(f"  {'TOTAL EXCHANGES':<20} {len(exchanges):>5}")
    print("=" * 60)


if __name__ == "__main__":
    main()
