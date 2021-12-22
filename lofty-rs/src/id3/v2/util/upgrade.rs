use std::collections::HashMap;

macro_rules! gen_upgrades {
    (V2 => [$($($v2_key:literal)|* => $id3v24_from_v2:literal),+]; V3 => [$($($v3_key:literal)|* => $id3v24_from_v3:literal),+]) => {
		lazy_static::lazy_static! {
			static ref V2KEYS: HashMap<&'static str, &'static str> = {
				let mut map = HashMap::new();
				$(
					$(
						map.insert($v2_key, $id3v24_from_v2);
					)+
				)+
				map
			};
		}

		lazy_static::lazy_static! {
			static ref V3KEYS: HashMap<&'static str, &'static str> = {
				let mut map = HashMap::new();
				$(
					$(
						map.insert($v3_key, $id3v24_from_v3);
					)+
				)+
				map
			};
		}

		/// Upgrade an ID3v2.2 key to an ID3v2.4 key
        pub fn upgrade_v2(key: &str) -> Option<&'static str> {
            V2KEYS.get(key).map(|s| *s)
        }

		/// Upgrade an ID3v2.3 key to an ID3v2.4 key
        pub fn upgrade_v3(key: &str) -> Option<&'static str> {
            V3KEYS.get(key).map(|s| *s)
        }
    }
}

gen_upgrades!(
	// ID3v2.2 => ID3v2.4
	V2 => [
		// Standard frames
		"BUF" => "RBUF",
		"CNT" => "PCNT",
		"COM" => "COMM",
		"CRA" => "AENC",
		"ETC" => "ETCO",
		"GEO" => "GEOB",
		"IPL" => "TIPL",
		"MCI" => "MCDI",
		"MLL" => "MLLT",
		"PIC" => "APIC",
		"POP" => "POPM",
		"REV" => "RVRB",
		"SLT" => "SYLT",
		"STC" => "SYTC",
		"TAL" => "TALB",
		"TBP" => "TBPM",
		"TCM" => "TCOM",
		"TCO" => "TCON",
		"TCP" => "TCMP",
		"TCR" => "TCOP",
		"TDY" => "TDLY",
		"TEN" => "TENC",
		"TFT" => "TFLT",
		"TKE" => "TKEY",
		"TLA" => "TLAN",
		"TLE" => "TLEN",
		"TMT" => "TMED",
		"TOA" => "TOAL",
		"TOF" => "TOFN",
		"TOL" => "TOLY",
		"TOR" => "TDOR",
		"TOT" => "TOAL",
		"TP1" => "TPE1",
		"TP2" => "TPE2",
		"TP3" => "TPE3",
		"TP4" => "TPE4",
		"TPA" => "TPOS",
		"TPB" => "TPUB",
		"TRC" => "TSRC",
		"TRD" => "TDRC",
		"TRK" => "TRCK",
		"TS2" => "TSO2",
		"TSA" => "TSOA",
		"TSC" => "TSOC",
		"TSP" => "TSOP",
		"TSS" => "TSSE",
		"TST" => "TSOT",
		"TT1" => "TIT1",
		"TT2" => "TIT2",
		"TT3" => "TIT3",
		"TXT" => "TOLY",
		"TXX" => "TXXX",
		"TYE" => "TDRC",
		"UFI" => "UFID",
		"ULT" => "USLT",
		"WAF" => "WOAF",
		"WAR" => "WOAR",
		"WAS" => "WOAS",
		"WCM" => "WCOM",
		"WCP" => "WCOP",
		"WPB" => "WPUB",
		"WXX" => "WXXX",

		// iTunes non-standard frames

		// Podcast
		"PCS" => "PCST",
		"TCT" => "TCAT",
		"TDS" => "TDES",
		"TID" => "TGID",
		"WFD" => "WFED",

		// Identifiers
		"MVI" => "MVIN",
		"MVN" => "MVNM",
		"GP1" => "GRP1",
		"TDR" => "TDRL"
	];
	// ID3v2.3 => ID3v2.4
	V3 => [
		// Standard frames
		"TORY" => "TDOR",
		"TYER" => "TDRC",
		"IPLS" => "TIPL"
	]
);
