use std::str::FromStr;

use anyhow::{anyhow, bail, Result};

pub trait ParseKeyValueAt {
    fn parse_key_value_at<K, V>(&self, line_num: usize) -> Result<(K, V)>
    where
        K: FromStr,
        <K as std::str::FromStr>::Err: std::error::Error + Send + Sync + 'static,
        V: FromStr,
        <V as std::str::FromStr>::Err: std::error::Error + Send + Sync + 'static;
}

impl ParseKeyValueAt for Vec<String> {
    fn parse_key_value_at<K, V>(&self, line_num: usize) -> Result<(K, V)>
    where
        K: FromStr,
        <K as FromStr>::Err: std::error::Error + Send + Sync + 'static,
        V: FromStr,
        <V as FromStr>::Err: std::error::Error + Send + Sync + 'static,
    {
        let l = self
            .get(line_num)
            .ok_or_else(|| anyhow!("Line number does not exist"))?;

        let split: Vec<_> = l.split(':').collect();

        if split.len() != 2 {
            bail!("Line does not contain K: V")
        }

        let k = split[0].parse::<K>()?;
        let v = split[1][1..].parse::<V>()?;

        Ok((k, v))
    }
}

pub fn path_to_routes(path: &[usize]) -> Vec<Vec<usize>> {
    let mut paths = Vec::new();
    for &i in path {
        if i == 0 {
            paths.push(Vec::new());
        } else {
            let last: &mut Vec<usize> = paths.last_mut().unwrap();
            last.push(i);
        }
    }

    paths
}
