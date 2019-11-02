use boxcars::{ParserBuilder, HeaderProp, ParseError, Replay};
use pyo3::prelude::*;
use pyo3::{wrap_pyfunction, create_exception};
use pyo3::types::{PyBytes};

type RLDateTime = (i32, u8, u8, u8, u8, u8);

#[pyclass]
pub struct BoxyReplay {
    replay: Replay 
}

#[pymethods]
impl BoxyReplay {
    fn replay_date(&self) -> Option<RLDateTime> {
        extract_property(&self.replay.properties, "Date", get_datetime)
    }

    fn replay_map(&self) -> Option<String> {
        extract_property(&self.replay.properties, "MapName", get_string)
    }

    fn replay_id(&self) -> Option<String> {
        extract_property(&self.replay.properties, "Id", get_string)
    }

    fn team_size(&self) -> Option<i32> {
        extract_property(&self.replay.properties, "TeamSize", get_int)
    }

    fn player_stats(&self) -> Option<Vec<PlayerStats>> {
        extract_property(&self.replay.properties, "PlayerStats", get_array)
            .map(|stats|
                stats.iter()
                    .map(|x| extract_player_stats(x))
                    .collect()
            )
    }
}

#[pyclass]
pub struct PlayerStats {
   pub assists: i32,
   pub goals: i32,
   pub name: String,
   pub online_id: u64,
   pub saves: i32,
   pub score: i32,
   pub shots: i32,
   pub team: i32,
   pub bot: bool,
}

#[pymethods]
impl PlayerStats {
    #[getter]
    fn assists(&self) -> PyResult<i32> {
        Ok(self.assists)
    }

    #[getter]
    fn goals(&self) -> PyResult<i32> {
        Ok(self.goals)
    }

    #[getter]
    fn saves(&self) -> PyResult<i32> {
        Ok(self.saves)
    }

    #[getter]
    fn score(&self) -> PyResult<i32> {
        Ok(self.score)
    }

    #[getter]
    fn shots(&self) -> PyResult<i32> {
        Ok(self.shots)
    }
    
    #[getter]
    fn team(&self) -> PyResult<i32> {
        Ok(self.team)
    }
}

fn get_int(prop_val: &HeaderProp) -> Option<i32> {
    if let HeaderProp::Int(val) = prop_val {
        Some(*val)
    } else {
        None
    }
}

fn get_string(prop_val: &HeaderProp) -> Option<String> {
    if let HeaderProp::Str(val) = prop_val {
        Some(val.clone())
    } else if let HeaderProp::Name(val) = prop_val {
        Some(val.clone())
    } else {
        None
    }
}

fn get_u64(prop_val: &HeaderProp) -> Option<u64> {
    if let HeaderProp::QWord(val) = prop_val {
        Some(*val)
    } else {
        None
    }
}

fn get_bool(prop_val: &HeaderProp) -> Option<bool> {
    if let HeaderProp::Bool(val) = prop_val {
        Some(*val)
    } else {
        None
    }
}

fn get_array(prop_val: &HeaderProp) -> Option<&Vec<Vec<(String, HeaderProp)>>> {
    if let HeaderProp::Array(val) = prop_val {
        Some(val)
    } else {
        None
    }
}

fn split_date_part(dt: &str) -> Option<(i32, u8, u8)> {
    let mut pts = dt.split('-');
    let y = pts.next().and_then(|x| x.parse::<i32>().ok());
    let m = pts.next().and_then(|x| x.parse::<u8>().ok());
    let d = pts.next().and_then(|x| x.parse::<u8>().ok());
    y.and_then(|y| m.and_then(|m| d.map(|d| (y, m, d))))
}

fn get_datetime(prop_val: &HeaderProp) -> Option<RLDateTime> {
    if let HeaderProp::Str(val) = prop_val {
        let mut ws = val.split_ascii_whitespace();
        let date = ws.next().and_then(split_date_part); 
        let time = ws.next().and_then(split_date_part); 
        date.and_then(|(y, m, d)| time.map(|(hour, min, sec)| 
            ((y, m, d, hour as u8, min, sec))
        ))
    } else {
        None
    }
}

fn extract_property<'a, T>(
    properties: &'a [(String, HeaderProp)],
    name: &str,
    f: impl Fn(&'a HeaderProp) -> Option<T>
) -> Option<T> {
    properties
        .iter()
        .find(|(prop_name, _)| *prop_name == name)
        .and_then(|(_, prop_val)| f(prop_val))
}

fn extract_player_stats(
    properties: &[(String, HeaderProp)]
) -> PlayerStats {
    let assists = extract_property(properties, "Assists", get_int).unwrap_or(0);
    let goals = extract_property(properties, "Goals", get_int).unwrap_or(0);
    let saves = extract_property(properties, "Saves", get_int).unwrap_or(0);
    let score = extract_property(properties, "Score", get_int).unwrap_or(0);
    let shots = extract_property(properties, "Shots", get_int).unwrap_or(0);
    let team = extract_property(properties, "Team", get_int).unwrap_or(0);
    let name = extract_property(properties, "Name", get_string).unwrap_or_else(|| String::from(""));
    let online_id = extract_property(properties, "OnlineID", get_u64).unwrap_or(0);
    let bot = extract_property(properties, "bBot", get_bool).unwrap_or(false);

    PlayerStats {
        assists,
        goals,
        name,
        online_id,
        saves,
        score,
        shots,
        team,
        bot,
    }
}

fn lib_parse_replay_header(data: &[u8]) -> Result<Replay, ParseError> {
    ParserBuilder::new(&data[..])
        .on_error_check_crc()
        .never_parse_network_data()
        .parse()
}


create_exception!(libboxcapy2, BoxcarsError, pyo3::exceptions::Exception);

#[pyfunction]
fn parse_replay_header(data: &PyBytes) -> PyResult<BoxyReplay> {
    let res = lib_parse_replay_header(data.as_bytes())
	.map_err(|err| PyErr::new::<BoxcarsError, _>(err.to_string()))?;

    Ok(BoxyReplay {
        replay: res
    })
}

/// This module is a python module implemented in Rust.
#[pymodule]
fn libboxcapy2(_: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(parse_replay_header))?;
    Ok(())
}
