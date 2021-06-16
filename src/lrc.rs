#[derive(Debug)]
pub struct Lyric {
    pub metadata: std::collections::HashMap<Box<str>, Box<str>>,
    pub content: Box<[LyricsType]>,
}
impl Lyric {
    pub fn parse(file: String) -> Result<Self, ()> {
        let mut metadata_parse_completed = false;
        let mut metadata = std::collections::HashMap::new();
        let mut content = Vec::new();
        // Some subtitle is more than one line
        let file = if cfg!(feature = "multiline-lrc") {
            let mut restructured_file = String::new();
            for i in file.split('\n') {
                if i.contains('[') && i.contains(']') {
                    restructured_file.push('\n');
                    restructured_file.push_str(i);
                } else {
                    log::warn!(
                        "Line {} does not contain time element. Concating to previous line...",
                        i
                    );
                    restructured_file.push_str(i);
                }
            }
            restructured_file
        } else {
            file
        };
        for i in file.split('\n') {
            let i = i.trim();
            if i.is_empty() {
                continue;
            }
            // This implementation will break if the metadata contains '.' and ":"
            if !metadata_parse_completed
                && !(i[..i.find(']').unwrap()].contains('.')
                    && i[..i.find(']').unwrap()].contains(':'))
            {
                let mut metadata_segment =
                    i[i.find('[').unwrap() + 1..i.find(']').unwrap()].split(':');
                let key = Box::from(metadata_segment.nth(0).ok_or(())?);
                // There might be ':' in values
                let value = metadata_segment.collect::<Vec<&str>>().join(":");
                log::info!("Metadata added: key - {}; value - {}", key, value);
                metadata.insert(key, value.into_boxed_str());
            } else {
                metadata_parse_completed = true;
                content.push(LyricsType::parse_line(&mut i.to_string())?);
            }
        }
        Ok(Self {
            metadata,
            content: content.into_boxed_slice(),
        })
    }
}
#[derive(Debug)]
pub enum LyricsType {
    Standard(std::time::Duration, Box<str>),
    Enhanced(std::time::Duration, Box<[(std::time::Duration, Box<str>)]>),
}
impl LyricsType {
    fn parse_standard(line: &mut String) -> Result<Self, ()> {
        //[00:12.00]Line 1 lyrics
        // !line.starts_with('[') | !line.contains(']')
        // First, parse the time
        let time = parse_time(
            line.get(line.find('[').unwrap() + 1..line.find(']').unwrap())
                .unwrap(),
        )?;
        Ok(Self::Standard(
            time,
            line.drain(line.find(']').unwrap() + 1..)
                .collect::<String>()
                .into_boxed_str(),
        ))
    }
    fn parse_enhanced(line: &mut Self) -> Result<Self, ()> {
        if let Self::Standard(_, i) = line {
            if !(i.contains('<') && i.contains('>')) {
                return Err(());
            }
        }
        let (raw_duration, raw_string) = if let Self::Standard(i, j) = line {
            (i, j)
        } else {
            if let Self::Enhanced(_, _) = line {
                return Err(());
            } else {
                return Err(());
            }
        };
        let mut parsed_string = Vec::new();
        let (mut offset, mut duration) = (0, None);
        let raw_string = raw_string.trim();
        for x in raw_string.chars().into_iter().enumerate() {
            if x.1 == '<' {
                //The previous one is a lyric
                if !raw_string[0..x.0].trim().is_empty() {
                    parsed_string.push((duration.ok_or(())?, Box::from(&raw_string[offset..x.0])));
                }
                offset = x.0 + 1;
            } else if x.1 == '>' {
                println!("{:?}", &raw_string[offset..x.0]);
                //The previous one is a duration
                duration = Some(parse_time(&raw_string[offset..x.0])?);
                offset = x.0 + 1;
            }
        }
        let duration = std::mem::replace(raw_duration, std::time::Duration::default());
        Ok(Self::Enhanced(duration, parsed_string.into_boxed_slice()))
    }
    pub fn parse_line(line: &mut String) -> Result<Self, ()> {
        let mut parsed = Self::parse_standard(line)?;
        Ok(Self::parse_enhanced(&mut parsed).unwrap_or(parsed))
    }
}
fn parse_time(string: &str) -> Result<std::time::Duration, ()> {
    //mm:ss.xx or mm:ss.xxx
    if !(string.contains(':')) | !(string.contains('.')) {
        return Err(());
    }
    let (x, y) = (string.find(':').unwrap(), string.find('.').unwrap());
    let minute = string.get(0..x).ok_or(())?.parse::<u32>().map_err(|_| ())?;
    let second = string
        .get(x + 1..y)
        .ok_or(())?
        .parse::<u32>()
        .map_err(|_| ())?;
    let micros = &format!("0.{}", string.get(y + 1..).ok_or(())?)
        .parse::<f64>()
        .map_err(|_| ())?;
    let sum_milis = minute as u64 * 60 * 1000 + second as u64 * 1000 + (micros * 1000.0) as u64;
    Ok(std::time::Duration::from_millis(sum_milis))
}
