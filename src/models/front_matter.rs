

use chrono::{NaiveDate, NaiveTime, Utc};
use serde::{Serialize, Deserialize, Deserializer, Serializer, de::{Error, Unexpected, Visitor}};
use uuid::Uuid;

const DEFAULT_TITLE: &str = "untitled";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrontMatter {
    #[serde(default = "default_id")]
    pub id: String,
    pub title: Option<String>,
    #[serde(deserialize_with = "from_date_string", serialize_with = "to_date_string", default = "default_date")]
    pub date: NaiveDate,
    #[serde(deserialize_with = "from_time_string", serialize_with = "to_time_string", default = "default_time")]
    pub time: NaiveTime,
}

fn default_id() -> String {
    Uuid::new_v4().to_simple().to_string()
}
fn default_date() -> NaiveDate {
    let utc = Utc::now();
    utc.naive_local().date()
}
fn default_time() -> NaiveTime {
    let utc = Utc::now();
    utc.naive_local().time()
}

impl Default for FrontMatter {
    fn default() -> Self {
        Self {
            id: default_id(),
            title: None,
            date: default_date(),
            time: default_time()
        }
    }
}

struct DateVisitor;
impl<'de> Visitor<'de> for DateVisitor {
    type Value = NaiveDate;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string representing a date in the format YYYY-MM-DD")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error, {
        match NaiveDate::parse_from_str(v, "%Y-%m-%d") {
            Ok(date) => {
                Ok(date)
            }
            Err(_e) => {
                Err(Error::invalid_value(Unexpected::Str(v), &self))
            }
        }
    }
}
fn from_date_string<'de, D>(d: D) -> Result<NaiveDate, D::Error> where D: Deserializer<'de> {
    d.deserialize_str(DateVisitor)
}
fn to_date_string<S>(date: &NaiveDate, s: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let str = date.format("%Y-%m-%d").to_string();
    s.serialize_str(&str)
}

struct TimeVisitor;
impl<'de> Visitor<'de> for TimeVisitor {
    type Value = NaiveTime;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string representing a time in the format HH:MM:SS")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error, {
        match NaiveTime::parse_from_str(v, "%H:%M:%S") {
            Ok(time) => {
                Ok(time)
            }
            Err(_e) => {
                Err(Error::invalid_value(Unexpected::Str(v), &self))
            }
        }
    }
}
fn from_time_string<'de, D>(d: D) -> Result<NaiveTime, D::Error> where D: Deserializer<'de> {
    d.deserialize_str(TimeVisitor)
}
fn to_time_string<S>(date: &NaiveTime, s: S) -> Result<S::Ok, S::Error> where S: Serializer {
    let str = date.format("%H:%M:%S").to_string();
    s.serialize_str(&str)
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, NaiveTime};

    use super::FrontMatter;

    #[test]
    fn serializes_date() {
        let id = "123e4567-e89b-12d3-a456-426614174000".to_string();
        let title = Some("test_note".to_string());
        let date = NaiveDate::from_ymd(2021, 4, 7);
        let time = NaiveTime::from_hms(23, 08, 15);
        let front_matter = FrontMatter {
            id,
            title,
            date,
            time
        };

        let serialized = serde_yaml::to_string(&front_matter).unwrap();
        println!("{}", serialized);
    }

    #[test]
    fn deserialize_date() {
        let dt = NaiveDate::from_ymd(2021, 5, 1);
        let fm = r#"title: serialized note
date: 2021-05-01"#;

        let front_matter: FrontMatter = serde_yaml::from_str(fm).unwrap();

        assert_eq!(Some("serialized note".to_string()), front_matter.title);
        assert_eq!(dt.clone(), front_matter.date);
    }
}